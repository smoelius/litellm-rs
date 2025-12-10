//! Types
//!
//! Defines common data structures and enums used in the system

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// Request
///
/// Handle
#[derive(Debug, Clone)]
pub struct RequestContext {
    /// Request
    pub request_id: String,

    /// User ID
    pub user_id: Option<String>,

    /// Client IP
    pub client_ip: Option<String>,

    /// User agent
    pub user_agent: Option<String>,

    /// Custom headers
    pub headers: HashMap<String, String>,

    /// Start time
    pub start_time: SystemTime,

    /// Extra metadata
    pub metadata: HashMap<String, serde_json::Value>,

    /// Trace ID (for distributed tracing)
    pub trace_id: Option<String>,

    /// Span ID
    pub span_id: Option<String>,
}

/// Provider configuration for router
/// Re-export from config module for backward compatibility
pub use crate::config::models::provider::ProviderConfig;

impl Default for RequestContext {
    fn default() -> Self {
        Self {
            request_id: uuid::Uuid::new_v4().to_string(),
            user_id: None,
            client_ip: None,
            user_agent: None,
            headers: HashMap::new(),
            start_time: SystemTime::now(),
            metadata: HashMap::new(),
            trace_id: None,
            span_id: None,
        }
    }
}

impl RequestContext {
    /// Created at
    pub fn new() -> Self {
        Self::default()
    }

    /// Settings
    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// Settings
    pub fn with_client_ip(mut self, client_ip: impl Into<String>) -> Self {
        self.client_ip = Some(client_ip.into());
        self
    }

    /// Settings
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    /// Add header
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// Settings
    pub fn with_trace_id(mut self, trace_id: impl Into<String>) -> Self {
        self.trace_id = Some(trace_id.into());
        self
    }

    /// Get
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed().unwrap_or_default()
    }
}

/// Provider capability enum
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderCapability {
    /// Chat completion
    ChatCompletion,
    /// Streaming chat completion
    ChatCompletionStream,
    /// Embeddings generation
    Embeddings,
    /// Image generation
    ImageGeneration,
    /// Image editing
    ImageEdit,
    /// Image variation
    ImageVariation,
    /// Audio transcription
    AudioTranscription,
    /// Audio translation
    AudioTranslation,
    /// Text to speech
    TextToSpeech,
    /// Tool calling
    ToolCalling,
    /// Function calling (backward compatibility)
    FunctionCalling,
    /// Code execution
    CodeExecution,
    /// File upload
    FileUpload,
    /// Fine-tuning
    FineTuning,
    /// Handle
    BatchProcessing,
    /// Real-time API
    RealtimeApi,
}

/// Model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Model
    pub id: String,

    /// Model
    pub name: String,

    /// Provider
    pub provider: String,

    /// Maximum context length
    pub max_context_length: u32,

    /// Maximum output length
    pub max_output_length: Option<u32>,

    /// Supports streaming
    pub supports_streaming: bool,

    /// Supports tool calling
    pub supports_tools: bool,

    /// Supports multimodal
    pub supports_multimodal: bool,

    /// Input price (per 1K tokens)
    pub input_cost_per_1k_tokens: Option<f64>,

    /// Output price (per 1K tokens)
    pub output_cost_per_1k_tokens: Option<f64>,

    /// Currency unit
    pub currency: String,

    /// Supported features
    pub capabilities: Vec<ProviderCapability>,

    /// Created at
    pub created_at: Option<SystemTime>,

    /// Updated at
    pub updated_at: Option<SystemTime>,

    /// Extra metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Default for ModelInfo {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            provider: String::new(),
            max_context_length: 4096,
            max_output_length: None,
            supports_streaming: false,
            supports_tools: false,
            supports_multimodal: false,
            input_cost_per_1k_tokens: None,
            output_cost_per_1k_tokens: None,
            currency: "USD".to_string(),
            capabilities: Vec::new(),
            created_at: None,
            updated_at: None,
            metadata: HashMap::new(),
        }
    }
}

/// Health status
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    /// Healthy
    Healthy,
    /// Unhealthy
    Unhealthy,
    /// Unknown status
    #[default]
    Unknown,
    /// Degraded service
    Degraded,
}

/// Check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    /// Status
    pub status: HealthStatus,

    /// Check
    pub checked_at: SystemTime,

    /// Latency (milliseconds)
    pub latency_ms: Option<u64>,

    /// Error
    pub error: Option<String>,

    /// Extra details
    pub details: HashMap<String, serde_json::Value>,
}

impl Default for HealthCheckResult {
    fn default() -> Self {
        Self {
            status: HealthStatus::Unknown,
            checked_at: SystemTime::now(),
            latency_ms: None,
            error: None,
            details: HashMap::new(),
        }
    }
}

/// Metric data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricDataPoint {
    /// Metric name
    pub name: String,

    /// Metric value
    pub value: f64,

    /// Labels
    pub labels: HashMap<String, String>,

    /// Timestamp
    pub timestamp: SystemTime,
}

/// Metric type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MetricType {
    /// Counter
    Counter,
    /// Gauge
    Gauge,
    /// Histogram
    Histogram,
    /// Summary
    Summary,
}

/// Metric definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricDefinition {
    /// Metric name
    pub name: String,

    /// Metric type
    pub metric_type: MetricType,

    /// Description
    pub description: String,

    /// Unit
    pub unit: Option<String>,

    /// Labels
    pub labels: Vec<String>,
}

/// Cache key type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheKey {
    /// Cache type
    pub cache_type: String,

    /// Key value
    pub key: String,

    /// Extra identifiers
    pub identifiers: HashMap<String, String>,
}

impl std::hash::Hash for CacheKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.cache_type.hash(state);
        self.key.hash(state);
        // Sort the HashMap keys for consistent hashing
        let mut sorted_keys: Vec<_> = self.identifiers.keys().collect();
        sorted_keys.sort();
        for k in sorted_keys {
            k.hash(state);
            self.identifiers.get(k).hash(state);
        }
    }
}

impl CacheKey {
    pub fn new(cache_type: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            cache_type: cache_type.into(),
            key: key.into(),
            identifiers: HashMap::new(),
        }
    }

    pub fn with_identifier(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.identifiers.insert(key.into(), value.into());
        self
    }
}

impl std::fmt::Display for CacheKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.cache_type, self.key)?;
        for (k, v) in &self.identifiers {
            write!(f, ":{}={}", k, v)?;
        }
        Ok(())
    }
}

/// API version
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApiVersion {
    /// v1 version
    #[default]
    V1,
    /// v2 version (future extension)
    V2,
}

impl std::fmt::Display for ApiVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::V1 => write!(f, "v1"),
            Self::V2 => write!(f, "v2"),
        }
    }
}

/// Service status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStatus {
    /// Service name
    pub name: String,

    /// version
    pub version: String,

    /// Status
    pub status: HealthStatus,

    /// Start time
    pub uptime: SystemTime,

    /// Connection
    pub active_connections: u32,

    /// Handle
    pub requests_processed: u64,

    /// Error
    pub errors: u64,

    /// Response
    pub avg_response_time_ms: f64,

    /// Memory usage (bytes)
    pub memory_usage_bytes: u64,

    /// CPU usage rate (percentage)
    pub cpu_usage_percent: f64,
}

/// Pagination parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    /// Page number (starts from 1)
    pub page: u32,

    /// Page size
    pub per_page: u32,

    /// Total count
    pub total: Option<u64>,

    /// Total pages
    pub total_pages: Option<u32>,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            page: 1,
            per_page: 20,
            total: None,
            total_pages: None,
        }
    }
}

impl Pagination {
    /// Calculate offset
    pub fn offset(&self) -> u32 {
        (self.page - 1) * self.per_page
    }

    /// Settings
    pub fn with_total(mut self, total: u64) -> Self {
        self.total = Some(total);
        self.total_pages = Some(((total as f64) / (self.per_page as f64)).ceil() as u32);
        self
    }
}

/// Sort parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortOrder {
    /// Sort field
    pub field: String,

    /// Sort direction
    pub direction: SortDirection,
}

/// Sort direction
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortDirection {
    /// Ascending
    #[default]
    Asc,
    /// Descending
    Desc,
}

/// Filter criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Filter {
    /// Field name
    pub field: String,

    /// Operator
    pub operator: FilterOperator,

    /// Value
    pub value: serde_json::Value,
}

/// Filter operator
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterOperator {
    /// Equals
    Eq,
    /// Not equals
    Ne,
    /// Greater than
    Gt,
    /// Greater than or equal
    Gte,
    /// Less than
    Lt,
    /// Less than or equal
    Lte,
    /// Contains
    Contains,
    /// Not contains
    NotContains,
    /// In list
    In,
    /// Not in list
    NotIn,
    /// Starts with
    StartsWith,
    /// Ends with
    EndsWith,
    /// Regex match
    Regex,
    /// Is null
    IsNull,
    /// Is not null
    IsNotNull,
}
