//! Metrics models for the Gateway
//!
//! This module defines metrics and monitoring data structures.

use super::Metadata;
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

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenUsage {
    /// Input tokens
    pub input_tokens: u32,
    /// Output tokens
    pub output_tokens: u32,
    /// Total tokens
    pub total_tokens: u32,
    /// Cached tokens
    pub cached_tokens: Option<u32>,
    /// Reasoning tokens
    pub reasoning_tokens: Option<u32>,
    /// Audio tokens
    pub audio_tokens: Option<u32>,
}

/// Cost information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CostInfo {
    /// Input cost
    pub input_cost: f64,
    /// Output cost
    pub output_cost: f64,
    /// Total cost
    pub total_cost: f64,
    /// Currency
    pub currency: String,
    /// Cost per token rates
    pub rates: CostRates,
}

/// Cost rates
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CostRates {
    /// Input cost per token
    pub input_cost_per_token: f64,
    /// Output cost per token
    pub output_cost_per_token: f64,
    /// Cost per request
    pub cost_per_request: Option<f64>,
}

/// Error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorInfo {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
    /// Error type
    pub error_type: String,
    /// Provider error code
    pub provider_code: Option<String>,
    /// Stack trace
    pub stack_trace: Option<String>,
}

/// Cache metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CacheMetrics {
    /// Cache hit
    pub hit: bool,
    /// Cache type
    pub cache_type: Option<String>,
    /// Cache key
    pub cache_key: Option<String>,
    /// Similarity score (for semantic cache)
    pub similarity_score: Option<f32>,
    /// Cache latency in milliseconds
    pub cache_latency_ms: Option<u64>,
}

/// Provider metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderMetrics {
    /// Metrics metadata
    #[serde(flatten)]
    pub metadata: Metadata,
    /// Provider name
    pub provider: String,
    /// Time period start
    pub period_start: chrono::DateTime<chrono::Utc>,
    /// Time period end
    pub period_end: chrono::DateTime<chrono::Utc>,
    /// Total requests
    pub total_requests: u64,
    /// Successful requests
    pub successful_requests: u64,
    /// Failed requests
    pub failed_requests: u64,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    /// P50 response time
    pub p50_response_time_ms: f64,
    /// P95 response time
    pub p95_response_time_ms: f64,
    /// P99 response time
    pub p99_response_time_ms: f64,
    /// Total tokens processed
    pub total_tokens: u64,
    /// Total cost
    pub total_cost: f64,
    /// Error breakdown
    pub error_breakdown: HashMap<String, u64>,
    /// Model breakdown
    pub model_breakdown: HashMap<String, ModelMetrics>,
}

/// Model-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetrics {
    /// Model name
    pub model: String,
    /// Request count
    pub requests: u64,
    /// Success count
    pub successes: u64,
    /// Total tokens
    pub tokens: u64,
    /// Total cost
    pub cost: f64,
    /// Average response time
    pub avg_response_time_ms: f64,
}

/// System metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    /// Metrics metadata
    #[serde(flatten)]
    pub metadata: Metadata,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// CPU usage percentage
    pub cpu_usage: f64,
    /// Memory usage in bytes
    pub memory_usage: u64,
    /// Memory usage percentage
    pub memory_usage_percent: f64,
    /// Disk usage in bytes
    pub disk_usage: u64,
    /// Disk usage percentage
    pub disk_usage_percent: f64,
    /// Network I/O
    pub network_io: NetworkIO,
    /// Active connections
    pub active_connections: u32,
    /// Queue sizes
    pub queue_sizes: HashMap<String, u32>,
    /// Thread pool stats
    pub thread_pool: ThreadPoolStats,
}

/// Network I/O metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NetworkIO {
    /// Bytes received
    pub bytes_received: u64,
    /// Bytes sent
    pub bytes_sent: u64,
    /// Packets received
    pub packets_received: u64,
    /// Packets sent
    pub packets_sent: u64,
}

/// Thread pool statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThreadPoolStats {
    /// Active threads
    pub active_threads: u32,
    /// Total threads
    pub total_threads: u32,
    /// Queued tasks
    pub queued_tasks: u32,
    /// Completed tasks
    pub completed_tasks: u64,
}

/// Usage analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageAnalytics {
    /// Analytics metadata
    #[serde(flatten)]
    pub metadata: Metadata,
    /// Time period
    pub period: TimePeriod,
    /// User ID (if user-specific)
    pub user_id: Option<Uuid>,
    /// Team ID (if team-specific)
    pub team_id: Option<Uuid>,
    /// Total requests
    pub total_requests: u64,
    /// Total tokens
    pub total_tokens: u64,
    /// Total cost
    pub total_cost: f64,
    /// Model usage breakdown
    pub model_usage: HashMap<String, ModelUsage>,
    /// Provider usage breakdown
    pub provider_usage: HashMap<String, ProviderUsage>,
    /// Daily breakdown
    pub daily_breakdown: Vec<DailyUsage>,
    /// Top endpoints
    pub top_endpoints: Vec<EndpointUsage>,
}

/// Time period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimePeriod {
    /// Period start
    pub start: chrono::DateTime<chrono::Utc>,
    /// Period end
    pub end: chrono::DateTime<chrono::Utc>,
    /// Period type
    pub period_type: PeriodType,
}

/// Period type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PeriodType {
    /// Hourly period
    Hour,
    /// Daily period
    Day,
    /// Weekly period
    Week,
    /// Monthly period
    Month,
    /// Yearly period
    Year,
    /// Custom period
    Custom,
}

/// Model usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelUsage {
    /// Model name
    pub model: String,
    /// Request count
    pub requests: u64,
    /// Token count
    pub tokens: u64,
    /// Cost
    pub cost: f64,
    /// Success rate
    pub success_rate: f64,
    /// Average response time
    pub avg_response_time_ms: f64,
}

/// Provider usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderUsage {
    /// Provider name
    pub provider: String,
    /// Request count
    pub requests: u64,
    /// Token count
    pub tokens: u64,
    /// Cost
    pub cost: f64,
    /// Success rate
    pub success_rate: f64,
    /// Average response time
    pub avg_response_time_ms: f64,
}

/// Daily usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyUsage {
    /// Date
    pub date: chrono::NaiveDate,
    /// Request count
    pub requests: u64,
    /// Token count
    pub tokens: u64,
    /// Cost
    pub cost: f64,
    /// Unique users
    pub unique_users: u32,
}

/// Endpoint usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointUsage {
    /// Endpoint path
    pub endpoint: String,
    /// Request count
    pub requests: u64,
    /// Success rate
    pub success_rate: f64,
    /// Average response time
    pub avg_response_time_ms: f64,
}

/// Alert configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    /// Alert metadata
    #[serde(flatten)]
    pub metadata: Metadata,
    /// Alert name
    pub name: String,
    /// Alert description
    pub description: Option<String>,
    /// Alert condition
    pub condition: AlertCondition,
    /// Alert threshold
    pub threshold: f64,
    /// Alert severity
    pub severity: AlertSeverity,
    /// Alert channels
    pub channels: Vec<String>,
    /// Alert enabled
    pub enabled: bool,
    /// Cooldown period in seconds
    pub cooldown_seconds: u64,
}

/// Alert condition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertCondition {
    /// High error rate condition
    ErrorRateHigh,
    /// Slow response time condition
    ResponseTimeSlow,
    /// High request volume condition
    RequestVolumeHigh,
    /// High cost condition
    CostHigh,
    /// Provider down condition
    ProviderDown,
    /// Quota exceeded condition
    QuotaExceeded,
    /// Custom alert condition
    Custom(String),
}

/// Alert severity
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertSeverity {
    /// Informational alert
    Info,
    /// Warning alert
    Warning,
    /// Error alert
    Error,
    /// Critical alert
    Critical,
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

impl TokenUsage {
    /// Create new token usage
    pub fn new(input_tokens: u32, output_tokens: u32) -> Self {
        Self {
            input_tokens,
            output_tokens,
            total_tokens: input_tokens + output_tokens,
            cached_tokens: None,
            reasoning_tokens: None,
            audio_tokens: None,
        }
    }
}

impl CostInfo {
    /// Create new cost info
    pub fn new(input_cost: f64, output_cost: f64, currency: String) -> Self {
        Self {
            input_cost,
            output_cost,
            total_cost: input_cost + output_cost,
            currency,
            rates: CostRates::default(),
        }
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

    #[test]
    fn test_token_usage() {
        let usage = TokenUsage::new(100, 50);
        assert_eq!(usage.input_tokens, 100);
        assert_eq!(usage.output_tokens, 50);
        assert_eq!(usage.total_tokens, 150);
    }

    #[test]
    fn test_cost_calculation() {
        let cost = CostInfo::new(0.01, 0.02, "USD".to_string());
        assert_eq!(cost.input_cost, 0.01);
        assert_eq!(cost.output_cost, 0.02);
        assert_eq!(cost.total_cost, 0.03);
        assert_eq!(cost.currency, "USD");
    }
}
