//! Request metrics models

use super::super::Metadata;
use super::{CacheMetrics, CostInfo, ErrorInfo, TokenUsage};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Request metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMetrics {
    /// Metrics metadata
    #[serde(flatten)]
    pub metadata: Metadata,
    /// Request ID
    pub request_id: String,
    /// User ID
    pub user_id: Option<Uuid>,
    /// Team ID
    pub team_id: Option<Uuid>,
    /// API Key ID
    pub api_key_id: Option<Uuid>,
    /// Model used
    pub model: String,
    /// Provider used
    pub provider: String,
    /// Request type
    pub request_type: String,
    /// Request status
    pub status: RequestStatus,
    /// HTTP status code
    pub status_code: u16,
    /// Request timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Response time in milliseconds
    pub response_time_ms: u64,
    /// Queue time in milliseconds
    pub queue_time_ms: u64,
    /// Provider response time in milliseconds
    pub provider_time_ms: u64,
    /// Token usage
    pub token_usage: TokenUsage,
    /// Cost information
    pub cost: CostInfo,
    /// Error information
    pub error: Option<ErrorInfo>,
    /// Cache information
    pub cache: CacheMetrics,
    /// Additional metadata
    pub extra: HashMap<String, serde_json::Value>,
}

/// Request status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestStatus {
    /// Request completed successfully
    Success,
    /// Request failed with error
    Error,
    /// Request timed out
    Timeout,
    /// Request hit rate limit
    RateLimit,
    /// Request exceeded quota
    QuotaExceeded,
    /// Request was cancelled
    Cancelled,
}

impl RequestMetrics {
    /// Create new request metrics
    pub fn new(request_id: String, model: String, provider: String, request_type: String) -> Self {
        Self {
            metadata: Metadata::new(),
            request_id,
            user_id: None,
            team_id: None,
            api_key_id: None,
            model,
            provider,
            request_type,
            status: RequestStatus::Success,
            status_code: 200,
            timestamp: chrono::Utc::now(),
            response_time_ms: 0,
            queue_time_ms: 0,
            provider_time_ms: 0,
            token_usage: TokenUsage::default(),
            cost: CostInfo::default(),
            error: None,
            cache: CacheMetrics::default(),
            extra: HashMap::new(),
        }
    }

    /// Set user context
    pub fn with_user(mut self, user_id: Uuid, team_id: Option<Uuid>) -> Self {
        self.user_id = Some(user_id);
        self.team_id = team_id;
        self
    }

    /// Set API key context
    pub fn with_api_key(mut self, api_key_id: Uuid) -> Self {
        self.api_key_id = Some(api_key_id);
        self
    }

    /// Set timing information
    pub fn with_timing(
        mut self,
        response_time_ms: u64,
        queue_time_ms: u64,
        provider_time_ms: u64,
    ) -> Self {
        self.response_time_ms = response_time_ms;
        self.queue_time_ms = queue_time_ms;
        self.provider_time_ms = provider_time_ms;
        self
    }

    /// Set token usage
    pub fn with_tokens(mut self, input_tokens: u32, output_tokens: u32) -> Self {
        self.token_usage.input_tokens = input_tokens;
        self.token_usage.output_tokens = output_tokens;
        self.token_usage.total_tokens = input_tokens + output_tokens;
        self
    }

    /// Set cost information
    pub fn with_cost(mut self, input_cost: f64, output_cost: f64, currency: String) -> Self {
        self.cost.input_cost = input_cost;
        self.cost.output_cost = output_cost;
        self.cost.total_cost = input_cost + output_cost;
        self.cost.currency = currency;
        self
    }

    /// Set error information
    pub fn with_error(mut self, error: ErrorInfo) -> Self {
        self.status = RequestStatus::Error;
        self.error = Some(error);
        self
    }

    /// Set cache information
    pub fn with_cache(mut self, cache: CacheMetrics) -> Self {
        self.cache = cache;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_metrics_creation() {
        let metrics = RequestMetrics::new(
            "req-123".to_string(),
            "gpt-4".to_string(),
            "openai".to_string(),
            "chat_completion".to_string(),
        );

        assert_eq!(metrics.request_id, "req-123");
        assert_eq!(metrics.model, "gpt-4");
        assert_eq!(metrics.provider, "openai");
        assert!(matches!(metrics.status, RequestStatus::Success));
    }
}
