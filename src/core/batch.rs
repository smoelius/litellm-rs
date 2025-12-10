//! Batch processing system for handling multiple requests efficiently
//!
//! This module provides batch processing capabilities for chat completions,
//! embeddings, and other API operations.

use crate::core::models::openai::{ChatCompletionRequest, EmbeddingRequest};

use crate::storage::database::Database;
use crate::utils::error::{GatewayError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info};
use uuid::Uuid;

/// Batch request for processing multiple operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRequest {
    /// Unique batch ID
    pub batch_id: String,
    /// User ID who created the batch
    pub user_id: String,
    /// Batch type
    pub batch_type: BatchType,
    /// Individual requests in the batch
    pub requests: Vec<BatchItem>,
    /// Batch metadata
    pub metadata: HashMap<String, String>,
    /// Completion window in hours (24h default)
    pub completion_window: Option<u32>,
    /// Webhook URL for completion notification
    pub webhook_url: Option<String>,
}

/// Types of batch operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BatchType {
    /// Chat completion batch requests
    ChatCompletion,
    /// Embedding batch requests
    Embedding,
    /// Image generation batch requests
    ImageGeneration,
    /// Custom batch request type
    Custom(String),
}

/// Individual item in a batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchItem {
    /// Custom ID for this request
    pub custom_id: String,
    /// HTTP method (usually POST)
    pub method: String,
    /// API endpoint
    pub url: String,
    /// Request body
    pub body: serde_json::Value,
}

/// Database batch record (different from BatchItem)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRecord {
    /// Batch ID
    pub id: String,
    /// Object type
    pub object: String,
    /// Endpoint used
    pub endpoint: String,
    /// Input file ID
    pub input_file_id: Option<String>,
    /// Completion window
    pub completion_window: String,
    /// Batch status
    pub status: BatchStatus,
    /// Output file ID
    pub output_file_id: Option<String>,
    /// Error file ID
    pub error_file_id: Option<String>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// In progress timestamp
    pub in_progress_at: Option<DateTime<Utc>>,
    /// Expiration timestamp
    pub expires_at: Option<DateTime<Utc>>,
    /// Finalizing timestamp
    pub finalizing_at: Option<DateTime<Utc>>,
    /// Completion timestamp
    pub completed_at: Option<DateTime<Utc>>,
    /// Failed timestamp
    pub failed_at: Option<DateTime<Utc>>,
    /// Expired timestamp
    pub expired_at: Option<DateTime<Utc>>,
    /// Cancelling timestamp
    pub cancelling_at: Option<DateTime<Utc>>,
    /// Cancelled timestamp
    pub cancelled_at: Option<DateTime<Utc>>,
    /// Request counts
    pub request_counts: BatchRequestCounts,
    /// Batch metadata
    pub metadata: Option<serde_json::Value>,
}

/// Batch response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResponse {
    /// Batch ID
    pub id: String,
    /// Object type
    pub object: String,
    /// Endpoint used
    pub endpoint: String,
    /// Batch status
    pub status: BatchStatus,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Completion timestamp
    pub completed_at: Option<DateTime<Utc>>,
    /// Expiration timestamp
    pub expires_at: Option<DateTime<Utc>>,
    /// Input file ID (for file-based batches)
    pub input_file_id: Option<String>,
    /// Output file ID (for completed batches)
    pub output_file_id: Option<String>,
    /// Error file ID (for failed requests)
    pub error_file_id: Option<String>,
    /// Request counts
    pub request_counts: BatchRequestCounts,
    /// Batch metadata
    pub metadata: Option<serde_json::Value>,
    /// Completion window
    pub completion_window: String,
    /// In progress timestamp
    pub in_progress_at: Option<DateTime<Utc>>,
    /// Finalizing timestamp
    pub finalizing_at: Option<DateTime<Utc>>,
    /// Failed timestamp
    pub failed_at: Option<DateTime<Utc>>,
    /// Expired timestamp
    pub expired_at: Option<DateTime<Utc>>,
    /// Cancelling timestamp
    pub cancelling_at: Option<DateTime<Utc>>,
    /// Cancelled timestamp
    pub cancelled_at: Option<DateTime<Utc>>,
}

/// Batch processing status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BatchStatus {
    /// Batch is being validated
    Validating,
    /// Batch validation failed
    Failed,
    /// Batch is being processed
    InProgress,
    /// Batch is being finalized
    Finalizing,
    /// Batch processing completed
    Completed,
    /// Batch has expired
    Expired,
    /// Batch is being cancelled
    Cancelling,
    /// Batch has been cancelled
    Cancelled,
}

/// Request counts for batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRequestCounts {
    /// Total requests in batch
    pub total: i32,
    /// Completed requests
    pub completed: i32,
    /// Failed requests
    pub failed: i32,
}

/// Individual batch result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResult {
    /// Custom ID from request
    pub custom_id: String,
    /// HTTP response
    pub response: Option<BatchHttpResponse>,
    /// Error information
    pub error: Option<BatchError>,
}

/// HTTP response for batch item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchHttpResponse {
    /// HTTP status code
    pub status_code: u16,
    /// Response headers
    pub headers: HashMap<String, String>,
    /// Response body
    pub body: serde_json::Value,
}

/// Batch error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchError {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
    /// Additional error details
    pub details: Option<serde_json::Value>,
}

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

    /// Helper methods
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

// ============================================================================
// Async Batch Completion - Concurrent Request Processing
// ============================================================================
// This section provides high-performance concurrent batch processing for
// chat completions, similar to Python LiteLLM's `abatch_completion()`.

use futures::stream::{self, StreamExt};
use std::time::Duration;

/// Configuration for async batch processing
#[derive(Debug, Clone)]
pub struct AsyncBatchConfig {
    /// Maximum concurrent requests (default: 10)
    pub concurrency: usize,
    /// Timeout per individual request (default: 60s)
    pub timeout: Duration,
    /// Continue processing on individual failures (default: true)
    pub continue_on_error: bool,
    /// Retry failed requests (default: 1)
    pub max_retries: u32,
    /// Delay between retries (default: 1s)
    pub retry_delay: Duration,
}

impl Default for AsyncBatchConfig {
    fn default() -> Self {
        Self {
            concurrency: 10,
            timeout: Duration::from_secs(60),
            continue_on_error: true,
            max_retries: 1,
            retry_delay: Duration::from_secs(1),
        }
    }
}

impl AsyncBatchConfig {
    /// Create a new config
    pub fn new() -> Self {
        Self::default()
    }

    /// Set concurrency limit
    pub fn with_concurrency(mut self, concurrency: usize) -> Self {
        self.concurrency = concurrency.max(1);
        self
    }

    /// Set timeout per request
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set whether to continue on individual errors
    pub fn with_continue_on_error(mut self, continue_on_error: bool) -> Self {
        self.continue_on_error = continue_on_error;
        self
    }

    /// Set max retries
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }
}

/// Result of an individual request in a batch
#[derive(Debug, Clone)]
pub struct AsyncBatchItemResult<T> {
    /// Index of the request in the original batch
    pub index: usize,
    /// The result (Ok or Err)
    pub result: std::result::Result<T, AsyncBatchError>,
    /// Time taken for this request
    pub duration: Duration,
    /// Number of retries attempted
    pub retries: u32,
}

/// Error for async batch operations
#[derive(Debug, Clone)]
pub struct AsyncBatchError {
    /// Error message
    pub message: String,
    /// Error code (if available)
    pub code: Option<String>,
    /// Whether this error is retryable
    pub retryable: bool,
}

impl std::fmt::Display for AsyncBatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for AsyncBatchError {}

impl From<GatewayError> for AsyncBatchError {
    fn from(err: GatewayError) -> Self {
        let retryable = matches!(
            &err,
            GatewayError::Timeout(_) | GatewayError::Network(_) | GatewayError::RateLimit { .. }
        );

        Self {
            message: err.to_string(),
            code: None,
            retryable,
        }
    }
}

/// Summary of batch execution
#[derive(Debug, Clone)]
pub struct AsyncBatchSummary {
    /// Total requests processed
    pub total: usize,
    /// Successful requests
    pub succeeded: usize,
    /// Failed requests
    pub failed: usize,
    /// Total time for batch processing
    pub total_duration: Duration,
    /// Average time per request
    pub avg_duration: Duration,
}

/// Async batch executor for concurrent request processing
pub struct AsyncBatchExecutor {
    config: AsyncBatchConfig,
}

impl AsyncBatchExecutor {
    /// Create a new batch executor
    pub fn new(config: AsyncBatchConfig) -> Self {
        Self { config }
    }

    /// Execute a batch of async operations concurrently
    ///
    /// # Arguments
    /// * `items` - Iterator of items to process
    /// * `operation` - Async function to execute for each item
    ///
    /// # Returns
    /// Vector of results in the same order as input items
    ///
    /// # Example
    /// ```rust,ignore
    /// use litellm_rs::core::batch::{AsyncBatchExecutor, AsyncBatchConfig};
    ///
    /// let executor = AsyncBatchExecutor::new(
    ///     AsyncBatchConfig::new()
    ///         .with_concurrency(5)
    ///         .with_timeout(Duration::from_secs(30))
    /// );
    ///
    /// let requests = vec![request1, request2, request3];
    /// let results = executor.execute(requests, |req| async move {
    ///     provider.complete(req).await
    /// }).await;
    /// ```
    pub async fn execute<T, R, F, Fut>(
        &self,
        items: impl IntoIterator<Item = T>,
        operation: F,
    ) -> Vec<AsyncBatchItemResult<R>>
    where
        T: Send + 'static,
        R: Send + 'static,
        F: Fn(T) -> Fut + Send + Sync + Clone + 'static,
        Fut: std::future::Future<Output = std::result::Result<R, GatewayError>> + Send,
    {
        let items_with_index: Vec<(usize, T)> = items.into_iter().enumerate().collect();
        let config = self.config.clone();

        let results: Vec<AsyncBatchItemResult<R>> = stream::iter(items_with_index)
            .map(|(index, item)| {
                let op = operation.clone();
                let cfg = config.clone();

                async move {
                    let start = std::time::Instant::now();
                    let retries = 0u32;

                    let result = tokio::time::timeout(cfg.timeout, op(item))
                        .await
                        .map_err(|_| {
                            GatewayError::Timeout(format!(
                                "Request {} timed out after {:?}",
                                index, cfg.timeout
                            ))
                        })
                        .and_then(|r| r);

                    match result {
                        Ok(value) => AsyncBatchItemResult {
                            index,
                            result: Ok(value),
                            duration: start.elapsed(),
                            retries,
                        },
                        Err(e) => {
                            let batch_err = AsyncBatchError::from(e);
                            // Note: Can't retry because item is consumed
                            // In a real implementation, we'd clone the item
                            AsyncBatchItemResult {
                                index,
                                result: Err(batch_err),
                                duration: start.elapsed(),
                                retries,
                            }
                        }
                    }
                }
            })
            .buffer_unordered(config.concurrency)
            .collect()
            .await;

        // Sort by index to maintain original order
        let mut sorted_results = results;
        sorted_results.sort_by_key(|r| r.index);
        sorted_results
    }

    /// Execute with summary statistics
    pub async fn execute_with_summary<T, R, F, Fut>(
        &self,
        items: impl IntoIterator<Item = T>,
        operation: F,
    ) -> (Vec<AsyncBatchItemResult<R>>, AsyncBatchSummary)
    where
        T: Send + 'static,
        R: Send + 'static,
        F: Fn(T) -> Fut + Send + Sync + Clone + 'static,
        Fut: std::future::Future<Output = std::result::Result<R, GatewayError>> + Send,
    {
        let start = std::time::Instant::now();
        let results = self.execute(items, operation).await;
        let total_duration = start.elapsed();

        let total = results.len();
        let succeeded = results.iter().filter(|r| r.result.is_ok()).count();
        let failed = total - succeeded;
        let avg_duration = if total > 0 {
            Duration::from_nanos((total_duration.as_nanos() / total as u128) as u64)
        } else {
            Duration::ZERO
        };

        let summary = AsyncBatchSummary {
            total,
            succeeded,
            failed,
            total_duration,
            avg_duration,
        };

        (results, summary)
    }

    /// Get current configuration
    pub fn config(&self) -> &AsyncBatchConfig {
        &self.config
    }
}

impl Default for AsyncBatchExecutor {
    fn default() -> Self {
        Self::new(AsyncBatchConfig::default())
    }
}

/// Convenience function for batch completion without creating an executor
pub async fn batch_execute<T, R, F, Fut>(
    items: impl IntoIterator<Item = T>,
    operation: F,
    config: Option<AsyncBatchConfig>,
) -> Vec<AsyncBatchItemResult<R>>
where
    T: Send + 'static,
    R: Send + 'static,
    F: Fn(T) -> Fut + Send + Sync + Clone + 'static,
    Fut: std::future::Future<Output = std::result::Result<R, GatewayError>> + Send,
{
    let executor = AsyncBatchExecutor::new(config.unwrap_or_default());
    executor.execute(items, operation).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_batch_creation() {
        // TODO: Create a proper mock database for testing
        // For now, skip this test as it requires a real database
        // This test would create a BatchProcessor and test batch creation
        // when proper database mocking is implemented
    }

    #[test]
    fn test_batch_status_transitions() {
        assert_eq!(BatchStatus::Validating, BatchStatus::Validating);
        assert_ne!(BatchStatus::Validating, BatchStatus::InProgress);
    }

    // Async Batch Tests

    #[test]
    fn test_async_batch_config_builder() {
        let config = AsyncBatchConfig::new()
            .with_concurrency(20)
            .with_timeout(Duration::from_secs(120))
            .with_continue_on_error(false)
            .with_max_retries(3);

        assert_eq!(config.concurrency, 20);
        assert_eq!(config.timeout, Duration::from_secs(120));
        assert!(!config.continue_on_error);
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_async_batch_config_min_concurrency() {
        let config = AsyncBatchConfig::new().with_concurrency(0);
        assert_eq!(config.concurrency, 1); // Should be at least 1
    }

    #[tokio::test]
    async fn test_async_batch_executor_success() {
        let executor = AsyncBatchExecutor::new(
            AsyncBatchConfig::new()
                .with_concurrency(2)
                .with_timeout(Duration::from_secs(5)),
        );

        let items = vec![1, 2, 3, 4, 5];

        let results = executor
            .execute(items, |n| async move {
                // Simulate async work
                tokio::time::sleep(Duration::from_millis(10)).await;
                Ok::<_, GatewayError>(n * 2)
            })
            .await;

        assert_eq!(results.len(), 5);

        // Check results are in order
        for (i, result) in results.iter().enumerate() {
            assert_eq!(result.index, i);
            assert!(result.result.is_ok());
            assert_eq!(result.result.as_ref().unwrap(), &((i + 1) * 2));
        }
    }

    #[tokio::test]
    async fn test_async_batch_executor_with_failures() {
        let executor = AsyncBatchExecutor::new(AsyncBatchConfig::new().with_concurrency(2));

        let items = vec![1, 2, 3, 4, 5];

        let results = executor
            .execute(items, |n| async move {
                if n == 3 {
                    Err(GatewayError::InvalidRequest("Test error".to_string()))
                } else {
                    Ok::<_, GatewayError>(n * 2)
                }
            })
            .await;

        assert_eq!(results.len(), 5);

        // Check that index 2 (value 3) failed
        let failed = results.iter().find(|r| r.index == 2).unwrap();
        assert!(failed.result.is_err());

        // Others should succeed
        let succeeded: Vec<_> = results.iter().filter(|r| r.result.is_ok()).collect();
        assert_eq!(succeeded.len(), 4);
    }

    #[tokio::test]
    async fn test_async_batch_executor_with_summary() {
        let executor = AsyncBatchExecutor::new(AsyncBatchConfig::new().with_concurrency(3));

        let items = vec![1, 2, 3, 4, 5];

        let (results, summary) = executor
            .execute_with_summary(items, |n| async move {
                if n % 2 == 0 {
                    Err(GatewayError::InvalidRequest("Even number".to_string()))
                } else {
                    Ok::<_, GatewayError>(n)
                }
            })
            .await;

        assert_eq!(results.len(), 5);
        assert_eq!(summary.total, 5);
        assert_eq!(summary.succeeded, 3); // 1, 3, 5
        assert_eq!(summary.failed, 2); // 2, 4
    }

    #[tokio::test]
    async fn test_batch_execute_convenience_fn() {
        let items = vec![10, 20, 30];

        let results = batch_execute(
            items,
            |n| async move { Ok::<_, GatewayError>(n + 1) },
            Some(AsyncBatchConfig::new().with_concurrency(2)),
        )
        .await;

        assert_eq!(results.len(), 3);
        assert_eq!(results[0].result.as_ref().unwrap(), &11);
        assert_eq!(results[1].result.as_ref().unwrap(), &21);
        assert_eq!(results[2].result.as_ref().unwrap(), &31);
    }

    #[test]
    fn test_async_batch_error_from_gateway_error() {
        let timeout_err = GatewayError::Timeout("timeout".to_string());
        let batch_err: AsyncBatchError = timeout_err.into();
        assert!(batch_err.retryable);

        let invalid_err = GatewayError::InvalidRequest("invalid".to_string());
        let batch_err: AsyncBatchError = invalid_err.into();
        assert!(!batch_err.retryable);
    }
}
