//! Webhook integration system
//!
//! This module provides webhook functionality for external system integration.

use crate::core::models::RequestContext;
use crate::utils::error::{GatewayError, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, info};
use uuid::Uuid;

/// Webhook event types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WebhookEventType {
    /// Request started
    RequestStarted,
    /// Request completed successfully
    RequestCompleted,
    /// Request failed
    RequestFailed,
    /// Rate limit exceeded
    RateLimitExceeded,
    /// Cost threshold exceeded
    CostThresholdExceeded,
    /// Provider health changed
    ProviderHealthChanged,
    /// Cache hit/miss
    CacheEvent,
    /// Batch completed
    BatchCompleted,
    /// Batch failed
    BatchFailed,
    /// User created
    UserCreated,
    /// User updated
    UserUpdated,
    /// API key created
    ApiKeyCreated,
    /// API key revoked
    ApiKeyRevoked,
    /// Budget threshold reached
    BudgetThresholdReached,
    /// Security alert
    SecurityAlert,
    /// Custom event
    Custom(String),
}

/// Webhook payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    /// Event type
    pub event_type: WebhookEventType,
    /// Event timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Request context
    pub request_context: Option<RequestContext>,
    /// Event data
    pub data: serde_json::Value,
    /// Event metadata
    pub metadata: HashMap<String, String>,
}

/// Webhook configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    /// Webhook URL
    pub url: String,
    /// Events to subscribe to
    pub events: Vec<WebhookEventType>,
    /// HTTP headers to include
    pub headers: HashMap<String, String>,
    /// Webhook secret for signature verification
    pub secret: Option<String>,
    /// Timeout for webhook requests
    pub timeout_seconds: u64,
    /// Maximum retry attempts
    pub max_retries: u32,
    /// Retry delay in seconds
    pub retry_delay_seconds: u64,
    /// Whether webhook is enabled
    pub enabled: bool,
}

impl Default for WebhookConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            events: vec![],
            headers: HashMap::new(),
            secret: None,
            timeout_seconds: 30,
            max_retries: 3,
            retry_delay_seconds: 5,
            enabled: true,
        }
    }
}

/// Webhook delivery status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WebhookDeliveryStatus {
    /// Pending delivery
    Pending,
    /// Successfully delivered
    Delivered,
    /// Failed to deliver
    Failed,
    /// Retrying delivery
    Retrying,
}

/// Webhook delivery record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookDelivery {
    /// Delivery ID
    pub id: String,
    /// Webhook configuration ID
    pub webhook_id: String,
    /// Event payload
    pub payload: WebhookPayload,
    /// Delivery status
    pub status: WebhookDeliveryStatus,
    /// HTTP response status code
    pub response_status: Option<u16>,
    /// Response body
    pub response_body: Option<String>,
    /// Number of attempts
    pub attempts: u32,
    /// Created timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last attempt timestamp
    pub last_attempt_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Next retry timestamp
    pub next_retry_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Webhook manager
pub struct WebhookManager {
    /// HTTP client for webhook requests
    client: Client,
    /// Consolidated webhook data - single lock for all related state
    data: Arc<RwLock<WebhookData>>,
}

/// Consolidated webhook data - single lock for all webhook-related state
#[derive(Debug, Default)]
struct WebhookData {
    /// Registered webhooks
    webhooks: HashMap<String, WebhookConfig>,
    /// Delivery queue
    delivery_queue: Vec<WebhookDelivery>,
    /// Webhook statistics
    stats: WebhookStats,
}

/// Webhook statistics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct WebhookStats {
    /// Total events sent
    pub total_events: u64,
    /// Successful deliveries
    pub successful_deliveries: u64,
    /// Failed deliveries
    pub failed_deliveries: u64,
    /// Average delivery time in milliseconds
    pub avg_delivery_time_ms: f64,
    /// Events by type
    pub events_by_type: HashMap<String, u64>,
}

impl WebhookManager {
    /// Create a new webhook manager
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| GatewayError::Network(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            client,
            data: Arc::new(RwLock::new(WebhookData::default())),
        })
    }

    /// Create a new webhook manager with default settings, panics on failure
    /// Use `new()` for fallible construction
    pub fn new_or_default() -> Self {
        Self::new().unwrap_or_else(|e| {
            tracing::error!(
                "Failed to create WebhookManager: {}, using minimal client",
                e
            );
            // Create a minimal client as fallback
            Self {
                client: Client::new(),
                data: Arc::new(RwLock::new(WebhookData::default())),
            }
        })
    }

    /// Register a webhook
    pub async fn register_webhook(&self, id: String, config: WebhookConfig) -> Result<()> {
        info!("Registering webhook: {} -> {}", id, config.url);

        // Validate webhook URL
        if config.url.is_empty() {
            return Err(GatewayError::Validation(
                "Webhook URL cannot be empty".to_string(),
            ));
        }

        if !config.url.starts_with("http://") && !config.url.starts_with("https://") {
            return Err(GatewayError::Validation(
                "Webhook URL must be HTTP or HTTPS".to_string(),
            ));
        }

        let mut data = self.data.write().await;
        data.webhooks.insert(id, config);

        Ok(())
    }

    /// Unregister a webhook
    pub async fn unregister_webhook(&self, id: &str) -> Result<()> {
        info!("Unregistering webhook: {}", id);

        let mut data = self.data.write().await;
        data.webhooks.remove(id);

        Ok(())
    }

    /// Send webhook event
    pub async fn send_event(
        &self,
        event_type: WebhookEventType,
        event_data: serde_json::Value,
        context: Option<RequestContext>,
    ) -> Result<()> {
        let payload = WebhookPayload {
            event_type: event_type.clone(),
            timestamp: chrono::Utc::now(),
            request_context: context,
            data: event_data,
            metadata: HashMap::new(),
        };

        let mut data = self.data.write().await;
        let mut deliveries = Vec::new();

        // Find webhooks subscribed to this event type
        for (webhook_id, config) in data.webhooks.iter() {
            if config.enabled && config.events.contains(&event_type) {
                let delivery = WebhookDelivery {
                    id: Uuid::new_v4().to_string(),
                    webhook_id: webhook_id.clone(),
                    payload: payload.clone(),
                    status: WebhookDeliveryStatus::Pending,
                    response_status: None,
                    response_body: None,
                    attempts: 0,
                    created_at: chrono::Utc::now(),
                    last_attempt_at: None,
                    next_retry_at: Some(chrono::Utc::now()),
                };
                deliveries.push(delivery);
            }
        }

        // Add to delivery queue and update statistics
        let delivery_count = deliveries.len();
        if !deliveries.is_empty() {
            data.delivery_queue.extend(deliveries);
            data.stats.total_events += 1;
            *data
                .stats
                .events_by_type
                .entry(format!("{:?}", event_type))
                .or_insert(0) += 1;
        }

        debug!(
            "Queued {} webhook deliveries for event: {:?}",
            delivery_count, event_type
        );
        Ok(())
    }

    /// Process webhook delivery queue
    pub async fn process_delivery_queue(&self) -> Result<()> {
        // First, collect deliveries to process and their configs
        let deliveries_to_process: Vec<(usize, WebhookDelivery, WebhookConfig)> = {
            let data = self.data.read().await;
            data.delivery_queue
                .iter()
                .enumerate()
                .filter(|(_, delivery)| {
                    delivery.status == WebhookDeliveryStatus::Pending
                        || (delivery.status == WebhookDeliveryStatus::Retrying
                            && delivery
                                .next_retry_at
                                .is_some_and(|t| t <= chrono::Utc::now()))
                })
                .filter_map(|(idx, delivery)| {
                    data.webhooks
                        .get(&delivery.webhook_id)
                        .map(|config| (idx, delivery.clone(), config.clone()))
                })
                .collect()
        };

        // Process each delivery (without holding the lock)
        let mut results: Vec<(usize, WebhookDeliveryStatus, Option<String>)> = Vec::new();

        for (idx, mut delivery, config) in deliveries_to_process {
            let result = self.deliver_webhook_internal(&mut delivery, &config).await;

            match result {
                Ok(_) => {
                    results.push((idx, WebhookDeliveryStatus::Delivered, None));
                }
                Err(e) => {
                    delivery.attempts += 1;
                    if delivery.attempts >= config.max_retries {
                        results.push((idx, WebhookDeliveryStatus::Failed, Some(e.to_string())));
                    } else {
                        let next_retry = chrono::Utc::now()
                            + chrono::Duration::seconds(config.retry_delay_seconds as i64);
                        results.push((
                            idx,
                            WebhookDeliveryStatus::Retrying,
                            Some(next_retry.to_rfc3339()),
                        ));
                    }
                }
            }
        }

        // Apply results with a single lock acquisition
        {
            let mut data = self.data.write().await;
            for (idx, status, info) in results {
                if let Some(delivery) = data.delivery_queue.get_mut(idx) {
                    delivery.status = status.clone();
                    delivery.last_attempt_at = Some(chrono::Utc::now());

                    match status {
                        WebhookDeliveryStatus::Delivered => {
                            data.stats.successful_deliveries += 1;
                        }
                        WebhookDeliveryStatus::Failed => {
                            data.stats.failed_deliveries += 1;
                            if let Some(err) = info {
                                error!("Webhook delivery failed permanently: {}", err);
                            }
                        }
                        WebhookDeliveryStatus::Retrying => {
                            if let Some(next_retry_str) = info {
                                delivery.next_retry_at =
                                    chrono::DateTime::parse_from_rfc3339(&next_retry_str)
                                        .ok()
                                        .map(|dt| dt.with_timezone(&chrono::Utc));
                            }
                            delivery.attempts += 1;
                        }
                        _ => {}
                    }
                }
            }

            // Remove completed deliveries (keep failed ones for debugging)
            data.delivery_queue
                .retain(|d| d.status != WebhookDeliveryStatus::Delivered);
        }

        Ok(())
    }

    /// Deliver a single webhook (internal version with config)
    async fn deliver_webhook_internal(
        &self,
        delivery: &mut WebhookDelivery,
        config: &WebhookConfig,
    ) -> Result<()> {
        let start_time = std::time::Instant::now();

        // Prepare request
        let mut request = self
            .client
            .post(&config.url)
            .timeout(Duration::from_secs(config.timeout_seconds))
            .header("Content-Type", "application/json")
            .header("User-Agent", "LiteLLM-Gateway/1.0");

        // Add custom headers
        for (key, value) in &config.headers {
            request = request.header(key, value);
        }

        // Add signature if secret is configured
        if let Some(secret) = &config.secret {
            let signature = self.generate_signature(&delivery.payload, secret)?;
            request = request.header("X-Webhook-Signature", signature);
        }

        // Send request
        let response = request
            .json(&delivery.payload)
            .send()
            .await
            .map_err(|e| GatewayError::Network(e.to_string()))?;

        let status_code = response.status().as_u16();
        let response_body = response.text().await.unwrap_or_default();

        delivery.response_status = Some(status_code);
        delivery.response_body = Some(response_body.clone());

        // Update delivery time statistics
        let delivery_time = start_time.elapsed().as_millis() as f64;
        {
            let mut data = self.data.write().await;
            data.stats.avg_delivery_time_ms = (data.stats.avg_delivery_time_ms
                * (data.stats.successful_deliveries as f64)
                + delivery_time)
                / (data.stats.successful_deliveries + 1) as f64;
        }

        if (200..300).contains(&status_code) {
            debug!(
                "Webhook delivered successfully: {} -> {}",
                delivery.webhook_id, config.url
            );
            Ok(())
        } else {
            Err(GatewayError::External(format!(
                "Webhook returned status {}: {}",
                status_code, response_body
            )))
        }
    }

    /// Deliver a single webhook
    async fn deliver_webhook(&self, delivery: &mut WebhookDelivery) -> Result<()> {
        let config = self.get_webhook_config(&delivery.webhook_id).await?;
        self.deliver_webhook_internal(delivery, &config).await
    }

    /// Get webhook configuration
    async fn get_webhook_config(&self, webhook_id: &str) -> Result<WebhookConfig> {
        let data = self.data.read().await;
        data.webhooks
            .get(webhook_id)
            .cloned()
            .ok_or_else(|| GatewayError::NotFound(format!("Webhook not found: {}", webhook_id)))
    }

    /// Generate webhook signature
    fn generate_signature(&self, payload: &WebhookPayload, secret: &str) -> Result<String> {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        type HmacSha256 = Hmac<Sha256>;

        let payload_json = serde_json::to_string(payload).map_err(GatewayError::Serialization)?;

        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .map_err(|e| GatewayError::Crypto(e.to_string()))?;

        mac.update(payload_json.as_bytes());
        let result = mac.finalize();

        Ok(format!("sha256={}", hex::encode(result.into_bytes())))
    }

    /// Get webhook statistics
    pub async fn get_stats(&self) -> WebhookStats {
        self.data.read().await.stats.clone()
    }

    /// List all registered webhooks
    pub async fn list_webhooks(&self) -> HashMap<String, WebhookConfig> {
        self.data.read().await.webhooks.clone()
    }

    /// Get delivery history
    pub async fn get_delivery_history(&self, limit: Option<usize>) -> Vec<WebhookDelivery> {
        let data = self.data.read().await;
        let limit = limit.unwrap_or(100).min(data.delivery_queue.len());
        // Pre-allocate with exact capacity and collect from reverse iterator
        let mut result = Vec::with_capacity(limit);
        result.extend(data.delivery_queue.iter().rev().take(limit).cloned());
        result
    }

    /// Start background delivery processor
    pub async fn start_delivery_processor(&self) -> Result<()> {
        let manager = self.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));

            loop {
                interval.tick().await;

                if let Err(e) = manager.process_delivery_queue().await {
                    error!("Error processing webhook delivery queue: {}", e);
                }
            }
        });

        info!("Started webhook delivery processor");
        Ok(())
    }
}

impl Clone for WebhookManager {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            data: self.data.clone(),
        }
    }
}

impl Default for WebhookManager {
    fn default() -> Self {
        Self::new_or_default()
    }
}

/// Webhook event builders
pub mod events {
    use super::*;

    /// Build request started event
    pub fn request_started(
        model: &str,
        provider: &str,
        context: RequestContext,
    ) -> (WebhookEventType, serde_json::Value) {
        (
            WebhookEventType::RequestStarted,
            serde_json::json!({
                "model": model,
                "provider": provider,
                "request_id": context.request_id,
                "user_id": context.user_id,
                "timestamp": chrono::Utc::now()
            }),
        )
    }

    /// Build request completed event
    pub fn request_completed(
        model: &str,
        provider: &str,
        tokens_used: u32,
        cost: f64,
        latency_ms: u64,
        context: RequestContext,
    ) -> (WebhookEventType, serde_json::Value) {
        (
            WebhookEventType::RequestCompleted,
            serde_json::json!({
                "model": model,
                "provider": provider,
                "tokens_used": tokens_used,
                "cost": cost,
                "latency_ms": latency_ms,
                "request_id": context.request_id,
                "user_id": context.user_id,
                "timestamp": chrono::Utc::now()
            }),
        )
    }

    /// Build request failed event
    pub fn request_failed(
        model: &str,
        provider: &str,
        error: &str,
        context: RequestContext,
    ) -> (WebhookEventType, serde_json::Value) {
        (
            WebhookEventType::RequestFailed,
            serde_json::json!({
                "model": model,
                "provider": provider,
                "error": error,
                "request_id": context.request_id,
                "user_id": context.user_id,
                "timestamp": chrono::Utc::now()
            }),
        )
    }

    /// Build cost threshold exceeded event
    pub fn cost_threshold_exceeded(
        user_id: &str,
        current_cost: f64,
        threshold: f64,
        period: &str,
    ) -> (WebhookEventType, serde_json::Value) {
        (
            WebhookEventType::CostThresholdExceeded,
            serde_json::json!({
                "user_id": user_id,
                "current_cost": current_cost,
                "threshold": threshold,
                "period": period,
                "timestamp": chrono::Utc::now()
            }),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_webhook_manager_creation() {
        let manager = WebhookManager::new().unwrap();
        let webhooks = manager.list_webhooks().await;
        assert!(webhooks.is_empty());
    }

    #[tokio::test]
    async fn test_webhook_registration() {
        let manager = WebhookManager::new().unwrap();

        let config = WebhookConfig {
            url: "https://example.com/webhook".to_string(),
            events: vec![WebhookEventType::RequestCompleted],
            ..Default::default()
        };

        manager
            .register_webhook("test".to_string(), config)
            .await
            .unwrap();

        let webhooks = manager.list_webhooks().await;
        assert_eq!(webhooks.len(), 1);
        assert!(webhooks.contains_key("test"));
    }

    #[test]
    fn test_webhook_event_types() {
        let event = WebhookEventType::RequestStarted;
        assert_eq!(event, WebhookEventType::RequestStarted);

        let custom_event = WebhookEventType::Custom("my_event".to_string());
        assert_eq!(
            custom_event,
            WebhookEventType::Custom("my_event".to_string())
        );
    }

    #[test]
    fn test_webhook_payload_serialization() {
        let payload = WebhookPayload {
            event_type: WebhookEventType::RequestCompleted,
            timestamp: chrono::Utc::now(),
            request_context: None,
            data: serde_json::json!({"test": "data"}),
            metadata: HashMap::new(),
        };

        let serialized = serde_json::to_string(&payload).unwrap();
        assert!(serialized.contains("RequestCompleted"));
        assert!(serialized.contains("test"));
    }
}
