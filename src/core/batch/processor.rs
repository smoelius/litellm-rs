//! Batch processor implementation

use super::types::*;
use crate::core::models::openai::{ChatCompletionRequest, EmbeddingRequest};
use crate::storage::database::Database;
use crate::utils::error::{GatewayError, Result};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info};
use uuid::Uuid;

/// Batch processor for handling batch operations
pub struct BatchProcessor {
    /// Database connection
    database: Arc<Database>,
    /// Active batches
    active_batches: Arc<RwLock<HashMap<String, BatchResponse>>>,
    /// Batch results storage
    results_storage: Arc<RwLock<HashMap<String, Vec<BatchResult>>>>,
}

impl BatchProcessor {
    /// Create a new batch processor
    pub fn new(database: Arc<Database>) -> Self {
        Self {
            database,
            active_batches: Arc::new(RwLock::new(HashMap::new())),
            results_storage: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new batch
    pub async fn create_batch(&self, request: BatchRequest) -> Result<BatchResponse> {
        info!("Creating batch: {}", request.batch_id);

        // Validate batch request
        self.validate_batch_request(&request).await?;

        let batch_response = BatchResponse {
            id: request.batch_id.clone(),
            object: "batch".to_string(),
            endpoint: self.get_endpoint_for_batch_type(&request.batch_type),
            status: BatchStatus::Validating,
            created_at: Utc::now(),
            completed_at: None,
            expires_at: Some(
                Utc::now()
                    + chrono::Duration::hours(request.completion_window.unwrap_or(24) as i64),
            ),
            input_file_id: None,
            output_file_id: None,
            error_file_id: None,
            request_counts: BatchRequestCounts {
                total: request.requests.len() as i32,
                completed: 0,
                failed: 0,
            },
            metadata: Some(
                serde_json::to_value(request.metadata.clone()).unwrap_or(serde_json::Value::Null),
            ),
            completion_window: format!("{}h", request.completion_window.unwrap_or(24)),
            in_progress_at: None,
            finalizing_at: None,
            failed_at: None,
            expired_at: None,
            cancelling_at: None,
            cancelled_at: None,
        };

        // Store batch in database
        self.database.create_batch(&request).await?;

        // Add to active batches
        {
            let mut active = self.active_batches.write().await;
            active.insert(request.batch_id.clone(), batch_response.clone());
        }

        // Start processing in background
        let processor = self.clone();
        let batch_id = request.batch_id.clone();
        tokio::spawn(async move {
            if let Err(e) = processor.process_batch(batch_id).await {
                error!("Batch processing failed: {}", e);
            }
        });

        Ok(batch_response)
    }

    /// Get batch status
    pub async fn get_batch(&self, batch_id: &str) -> Result<Option<BatchResponse>> {
        // Check active batches first
        {
            let active = self.active_batches.read().await;
            if let Some(batch) = active.get(batch_id) {
                return Ok(Some(batch.clone()));
            }
        }

        // Check database and convert BatchRequest to BatchResponse
        if let Some(batch_request) = self.database.get_batch_request(batch_id).await? {
            // Convert BatchRequest to BatchResponse
            let now = chrono::Utc::now();
            let batch_response = BatchResponse {
                id: batch_request.batch_id.clone(),
                object: "batch".to_string(),
                endpoint: "/v1/chat/completions".to_string(),
                input_file_id: Some(batch_request.batch_id.clone()),
                completion_window: "24h".to_string(),
                status: BatchStatus::Completed,
                output_file_id: Some(format!("{}_output", batch_request.batch_id)),
                error_file_id: None,
                created_at: now,
                in_progress_at: Some(now),
                expires_at: Some(now + chrono::Duration::try_days(1).unwrap_or_default()),
                finalizing_at: None,
                completed_at: Some(now),
                failed_at: None,
                expired_at: None,
                cancelling_at: None,
                cancelled_at: None,
                request_counts: BatchRequestCounts {
                    total: batch_request.requests.len() as i32,
                    completed: batch_request.requests.len() as i32,
                    failed: 0,
                },
                metadata: Some(
                    serde_json::to_value(batch_request.metadata).unwrap_or(serde_json::Value::Null),
                ),
            };
            return Ok(Some(batch_response));
        }

        Ok(None)
    }

    /// Cancel a batch
    pub async fn cancel_batch(&self, batch_id: &str) -> Result<BatchResponse> {
        info!("Cancelling batch: {}", batch_id);

        let mut batch = self
            .get_batch(batch_id)
            .await?
            .ok_or_else(|| GatewayError::NotFound("Batch not found".to_string()))?;

        // Only allow cancellation of certain statuses
        match batch.status {
            BatchStatus::Validating | BatchStatus::InProgress => {
                batch.status = BatchStatus::Cancelling;

                // Update in active batches
                {
                    let mut active = self.active_batches.write().await;
                    active.insert(batch_id.to_string(), batch.clone());
                }

                // Update in database
                self.database
                    .update_batch_status(batch_id, &format!("{:?}", batch.status))
                    .await?;

                Ok(batch)
            }
            _ => Err(GatewayError::InvalidRequest(
                "Batch cannot be cancelled in current status".to_string(),
            )),
        }
    }

    /// List batches for a user
    pub async fn list_batches(
        &self,
        _user_id: &str,
        limit: Option<u32>,
        after: Option<&str>,
    ) -> Result<Vec<BatchResponse>> {
        // Database returns BatchRecord, convert to BatchResponse
        let records = self
            .database
            .list_batches(Some(limit.unwrap_or(20) as i32), after)
            .await?;

        let responses = records
            .into_iter()
            .map(|record| BatchResponse {
                id: record.id,
                object: record.object,
                endpoint: record.endpoint,
                status: record.status,
                created_at: record.created_at,
                completed_at: record.completed_at,
                expires_at: record.expires_at,
                input_file_id: record.input_file_id,
                output_file_id: record.output_file_id,
                error_file_id: record.error_file_id,
                request_counts: record.request_counts,
                metadata: record.metadata,
                completion_window: record.completion_window,
                in_progress_at: record.in_progress_at,
                finalizing_at: record.finalizing_at,
                failed_at: record.failed_at,
                expired_at: record.expired_at,
                cancelling_at: record.cancelling_at,
                cancelled_at: record.cancelled_at,
            })
            .collect();

        Ok(responses)
    }

    /// Get batch results
    pub async fn get_batch_results(&self, batch_id: &str) -> Result<Vec<BatchResult>> {
        // Check in-memory results first
        {
            let results = self.results_storage.read().await;
            if let Some(batch_results) = results.get(batch_id) {
                return Ok(batch_results.clone());
            }
        }

        // Check database
        match self.database.get_batch_results(batch_id).await? {
            Some(json_results) => {
                // Convert JSON values to BatchResult
                let results: Vec<BatchResult> = json_results
                    .into_iter()
                    .filter_map(|v| serde_json::from_value(v).ok())
                    .collect();
                Ok(results)
            }
            None => Ok(Vec::new()),
        }
    }
}

// Private implementation methods
impl BatchProcessor {
    /// Validate batch request
    async fn validate_batch_request(&self, request: &BatchRequest) -> Result<()> {
        // Check request count limits
        if request.requests.len() > 50000 {
            return Err(GatewayError::InvalidRequest(
                "Batch size exceeds maximum limit of 50,000 requests".to_string(),
            ));
        }

        if request.requests.is_empty() {
            return Err(GatewayError::InvalidRequest(
                "Batch must contain at least one request".to_string(),
            ));
        }

        // Validate individual requests
        for item in &request.requests {
            self.validate_batch_item(item, &request.batch_type).await?;
        }

        Ok(())
    }

    /// Validate individual batch item
    async fn validate_batch_item(&self, item: &BatchItem, batch_type: &BatchType) -> Result<()> {
        // Validate custom_id
        if item.custom_id.is_empty() || item.custom_id.len() > 64 {
            return Err(GatewayError::InvalidRequest(
                "custom_id must be 1-64 characters".to_string(),
            ));
        }

        // Validate method
        if item.method != "POST" {
            return Err(GatewayError::InvalidRequest(
                "Only POST method is supported for batch requests".to_string(),
            ));
        }

        // Validate URL matches batch type
        match batch_type {
            BatchType::ChatCompletion => {
                if !item.url.contains("/chat/completions") {
                    return Err(GatewayError::InvalidRequest(
                        "URL must be /v1/chat/completions for chat completion batches".to_string(),
                    ));
                }
            }
            BatchType::Embedding => {
                if !item.url.contains("/embeddings") {
                    return Err(GatewayError::InvalidRequest(
                        "URL must be /v1/embeddings for embedding batches".to_string(),
                    ));
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Process a batch
    async fn process_batch(&self, batch_id: String) -> Result<()> {
        info!("Processing batch: {}", batch_id);

        // Update status to in progress
        self.update_batch_status(&batch_id, BatchStatus::InProgress)
            .await?;

        // Get batch request from database
        let batch_request = self
            .database
            .get_batch_request(&batch_id)
            .await?
            .ok_or_else(|| GatewayError::NotFound("Batch request not found".to_string()))?;

        let mut results = Vec::new();
        let mut completed = 0;
        let mut failed = 0;

        // Process each request
        for item in &batch_request.requests {
            // Check if batch was cancelled
            if self.is_batch_cancelled(&batch_id).await? {
                break;
            }

            match self
                .process_batch_item(item, &batch_request.batch_type)
                .await
            {
                Ok(result) => {
                    results.push(result);
                    completed += 1;
                }
                Err(e) => {
                    let error_result = BatchResult {
                        custom_id: item.custom_id.clone(),
                        response: None,
                        error: Some(BatchError {
                            code: "processing_error".to_string(),
                            message: e.to_string(),
                            details: None,
                        }),
                    };
                    results.push(error_result);
                    failed += 1;
                }
            }

            // Update progress periodically
            if (completed + failed) % 100 == 0 {
                self.update_batch_progress(&batch_id, completed, failed)
                    .await?;
            }
        }

        // Store results
        {
            let mut storage = self.results_storage.write().await;
            storage.insert(batch_id.clone(), results.clone());
        }

        // Store results in database
        let json_results: Vec<serde_json::Value> = results
            .iter()
            .map(|r| serde_json::to_value(r).unwrap_or_default())
            .collect();
        self.database
            .store_batch_results(&batch_id, &json_results)
            .await?;

        // Update final status
        let final_status = if self.is_batch_cancelled(&batch_id).await? {
            BatchStatus::Cancelled
        } else {
            BatchStatus::Completed
        };

        self.update_batch_status(&batch_id, final_status).await?;
        self.update_batch_progress(&batch_id, completed, failed)
            .await?;

        // Mark completion time
        self.mark_batch_completed(&batch_id).await?;

        info!(
            "Batch processing completed: {} (completed: {}, failed: {})",
            batch_id, completed, failed
        );

        Ok(())
    }

    /// Process individual batch item
    async fn process_batch_item(
        &self,
        item: &BatchItem,
        batch_type: &BatchType,
    ) -> Result<BatchResult> {
        debug!("Processing batch item: {}", item.custom_id);

        match batch_type {
            BatchType::ChatCompletion => {
                let request: ChatCompletionRequest = serde_json::from_value(item.body.clone())
                    .map_err(|e| {
                        GatewayError::InvalidRequest(format!("Invalid request body: {}", e))
                    })?;

                // This would need to be integrated with the actual provider system
                // For now, return a mock response
                let response = BatchHttpResponse {
                    status_code: 200,
                    headers: HashMap::new(),
                    body: serde_json::json!({
                        "id": format!("chatcmpl-batch-{}", Uuid::new_v4()),
                        "object": "chat.completion",
                        "created": Utc::now().timestamp(),
                        "model": request.model,
                        "choices": [{
                            "index": 0,
                            "message": {
                                "role": "assistant",
                                "content": "This is a batch processed response."
                            },
                            "finish_reason": "stop"
                        }],
                        "usage": {
                            "prompt_tokens": 10,
                            "completion_tokens": 8,
                            "total_tokens": 18
                        }
                    }),
                };

                Ok(BatchResult {
                    custom_id: item.custom_id.clone(),
                    response: Some(response),
                    error: None,
                })
            }
            BatchType::Embedding => {
                let request: EmbeddingRequest =
                    serde_json::from_value(item.body.clone()).map_err(|e| {
                        GatewayError::InvalidRequest(format!("Invalid request body: {}", e))
                    })?;

                let response = BatchHttpResponse {
                    status_code: 200,
                    headers: HashMap::new(),
                    body: serde_json::json!({
                        "object": "list",
                        "data": [{
                            "object": "embedding",
                            "embedding": vec![0.1; 1536], // Mock embedding
                            "index": 0
                        }],
                        "model": request.model,
                        "usage": {
                            "prompt_tokens": 5,
                            "total_tokens": 5
                        }
                    }),
                };

                Ok(BatchResult {
                    custom_id: item.custom_id.clone(),
                    response: Some(response),
                    error: None,
                })
            }
            _ => Err(GatewayError::InvalidRequest(
                "Unsupported batch type".to_string(),
            )),
        }
    }

    /// Get endpoint for batch type
    fn get_endpoint_for_batch_type(&self, batch_type: &BatchType) -> String {
        match batch_type {
            BatchType::ChatCompletion => "/v1/chat/completions".to_string(),
            BatchType::Embedding => "/v1/embeddings".to_string(),
            BatchType::ImageGeneration => "/v1/images/generations".to_string(),
            BatchType::Custom(endpoint) => endpoint.clone(),
        }
    }

    async fn update_batch_status(&self, batch_id: &str, status: BatchStatus) -> Result<()> {
        // Update in active batches
        {
            let mut active = self.active_batches.write().await;
            if let Some(batch) = active.get_mut(batch_id) {
                batch.status = status.clone();
            }
        }

        // Update in database
        self.database
            .update_batch_status(batch_id, &format!("{:?}", status))
            .await
    }

    async fn update_batch_progress(
        &self,
        batch_id: &str,
        completed: u32,
        failed: u32,
    ) -> Result<()> {
        // Update in active batches
        {
            let mut active = self.active_batches.write().await;
            if let Some(batch) = active.get_mut(batch_id) {
                batch.request_counts.completed = completed as i32;
                batch.request_counts.failed = failed as i32;
            }
        }

        // Update in database
        self.database
            .update_batch_progress(batch_id, completed as i32, failed as i32)
            .await
    }

    async fn mark_batch_completed(&self, batch_id: &str) -> Result<()> {
        let now = Utc::now();

        // Update in active batches
        {
            let mut active = self.active_batches.write().await;
            if let Some(batch) = active.get_mut(batch_id) {
                batch.completed_at = Some(now);
            }
        }

        // Update in database
        self.database.mark_batch_completed(batch_id).await
    }

    async fn is_batch_cancelled(&self, batch_id: &str) -> Result<bool> {
        let active = self.active_batches.read().await;
        if let Some(batch) = active.get(batch_id) {
            Ok(matches!(
                batch.status,
                BatchStatus::Cancelling | BatchStatus::Cancelled
            ))
        } else {
            Ok(false)
        }
    }
}

impl Clone for BatchProcessor {
    fn clone(&self) -> Self {
        Self {
            database: self.database.clone(),
            active_batches: self.active_batches.clone(),
            results_storage: self.results_storage.clone(),
        }
    }
}
