//! Configuration
//!
//! Configuration

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use url::Url;

/// Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteLLMConfig {
    /// Configuration
    pub server: ServerConfig,

    /// Configuration
    pub providers: Vec<ProviderConfigEntry>,

    /// Configuration
    pub routing: RoutingConfig,

    /// Configuration
    pub middleware: MiddlewareConfig,

    /// Configuration
    pub observability: ObservabilityConfig,
}

/// Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Listen address
    pub host: String,

    /// Listen port
    pub port: u16,

    /// Worker thread count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workers: Option<usize>,

    /// Connection
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_connections: Option<usize>,

    /// Request
    #[serde(with = "duration_serde")]
    pub timeout: Duration,

    /// Configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls: Option<TlsConfig>,

    /// Enabled features
    #[serde(default)]
    pub features: Vec<String>,
}

/// Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Certificate file path
    pub cert_file: String,

    /// Private key file path
    pub key_file: String,

    /// Enabled HTTP/2
    #[serde(default)]
    pub http2: bool,
}

/// Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfigEntry {
    /// Provider name (unique identifier)
    pub name: String,

    /// Provider type
    pub provider_type: String,

    /// Enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Routing weight (0.0-1.0)
    #[serde(default = "default_weight")]
    pub weight: f64,

    /// Configuration
    pub config: serde_json::Value,

    /// Labels (for routing and filtering)
    #[serde(default)]
    pub tags: HashMap<String, String>,

    /// Configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health_check: Option<HealthCheckConfig>,

    /// Configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry: Option<RetryConfig>,

    /// Configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limit: Option<RateLimitConfig>,
}

/// Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIProviderConfig {
    /// API key
    pub api_key: String,

    /// API base URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_base: Option<String>,

    /// Organization ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization: Option<String>,

    /// Request
    #[serde(default = "default_timeout_seconds")]
    pub timeout_seconds: u64,

    /// maximumNumber of retries
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// Model
    #[serde(default)]
    pub models: Vec<String>,

    /// Custom headers
    #[serde(default)]
    pub headers: HashMap<String, String>,
}

impl crate::core::traits::ProviderConfig for OpenAIProviderConfig {
    fn validate(&self) -> Result<(), String> {
        if self.api_key.is_empty() {
            return Err("API key is required".to_string());
        }

        if let Some(base_url) = &self.api_base {
            if Url::parse(base_url).is_err() {
                return Err("Invalid API base URL".to_string());
            }
        }

        if self.timeout_seconds == 0 {
            return Err("Timeout must be greater than 0".to_string());
        }

        Ok(())
    }

    fn api_key(&self) -> Option<&str> {
        Some(&self.api_key)
    }

    fn api_base(&self) -> Option<&str> {
        self.api_base.as_deref()
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(self.timeout_seconds)
    }

    fn max_retries(&self) -> u32 {
        self.max_retries
    }
}

/// Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingConfig {
    /// Routing strategy
    pub strategy: RoutingStrategyConfig,

    /// Configuration
    pub health_check: HealthCheckConfig,

    /// Configuration
    pub circuit_breaker: CircuitBreakerConfig,

    /// Configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub load_balancer: Option<LoadBalancerConfig>,
}

/// Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RoutingStrategyConfig {
    /// Round robin strategy
    #[serde(rename = "round_robin")]
    RoundRobin,

    /// Least load strategy
    #[serde(rename = "least_loaded")]
    LeastLoaded,

    /// Cost optimization strategy
    #[serde(rename = "cost_optimized")]
    CostOptimized {
        /// Performance weight (0.0-1.0)
        performance_weight: f32,
    },

    /// Latency optimization strategy
    #[serde(rename = "latency_based")]
    LatencyBased {
        /// Latency threshold (milliseconds)
        latency_threshold_ms: u64,
    },

    /// Tag-based routing strategy
    #[serde(rename = "tag_based")]
    TagBased {
        /// tag selectors
        selectors: Vec<TagSelector>,
    },

    /// Custom strategy
    #[serde(rename = "custom")]
    Custom {
        /// Strategy class name
        class: String,
        /// Configuration
        config: serde_json::Value,
    },
}

/// tag selectors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagSelector {
    /// Tag key
    pub key: String,

    /// Tag value (supports wildcards)
    pub value: String,

    /// Operator
    #[serde(default)]
    pub operator: TagOperator,
}

/// Tag operator
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TagOperator {
    /// Equals
    #[default]
    Eq,
    /// Not equals
    Ne,
    /// Contains
    In,
    /// Not contains
    NotIn,
    /// Exists
    Exists,
    /// Not exists
    NotExists,
}

/// Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Check
    #[serde(default = "default_health_check_interval")]
    pub interval_seconds: u64,

    /// Timeout duration（seconds）
    #[serde(default = "default_health_check_timeout")]
    pub timeout_seconds: u64,

    /// Health threshold（consecutive successes）
    #[serde(default = "default_health_threshold")]
    pub healthy_threshold: u32,

    /// Unhealthy threshold (consecutive failures)
    #[serde(default = "default_unhealthy_threshold")]
    pub unhealthy_threshold: u32,

    /// Check
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,

    /// Enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
}

/// Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Failure threshold
    #[serde(default = "default_failure_threshold")]
    pub failure_threshold: u32,

    /// Recovery timeout (seconds)
    #[serde(default = "default_recovery_timeout")]
    pub recovery_timeout_seconds: u64,

    /// Request
    #[serde(default = "default_half_open_requests")]
    pub half_open_max_requests: u32,

    /// Enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
}

/// Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancerConfig {
    /// Algorithm type
    pub algorithm: LoadBalancerAlgorithm,

    /// Configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_affinity: Option<SessionAffinityConfig>,
}

/// Load balancer algorithm
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LoadBalancerAlgorithm {
    /// Round robin
    RoundRobin,
    /// Weighted round robin
    WeightedRoundRobin,
    /// Connection
    LeastConnections,
    /// Consistent hash
    ConsistentHash,
}

/// Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionAffinityConfig {
    /// Affinity type
    pub affinity_type: SessionAffinityType,

    /// Timeout duration（seconds）
    #[serde(default = "default_session_timeout")]
    pub timeout_seconds: u64,
}

/// Session affinity type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionAffinityType {
    /// Based on client IP
    ClientIp,
    /// Based on user ID
    UserId,
    /// Based on custom header
    CustomHeader { header_name: String },
}

/// Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// maximumNumber of retries
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// Initial delay（milliseconds）
    #[serde(default = "default_initial_delay_ms")]
    pub initial_delay_ms: u64,

    /// Maximum delay (milliseconds)
    #[serde(default = "default_max_delay_ms")]
    pub max_delay_ms: u64,

    /// Use exponential backoff
    #[serde(default = "default_true")]
    pub exponential_backoff: bool,

    /// Add random jitter
    #[serde(default = "default_true")]
    pub jitter: bool,

    /// Error
    #[serde(default)]
    pub retryable_errors: Vec<String>,
}

/// Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Algorithm type
    pub algorithm: RateLimitAlgorithm,

    /// Request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requests_per_second: Option<u32>,

    /// Request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requests_per_minute: Option<u32>,

    /// Token rate limit (per minute)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens_per_minute: Option<u32>,

    /// Burst limit
    #[serde(skip_serializing_if = "Option::is_none")]
    pub burst_size: Option<u32>,
}

/// Rate limit algorithm
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RateLimitAlgorithm {
    /// Token bucket
    TokenBucket,
    /// Sliding window
    SlidingWindow,
    /// Fixed window
    FixedWindow,
}

/// Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiddlewareConfig {
    /// Configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache: Option<CacheConfig>,

    /// Configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry: Option<RetryConfig>,

    /// Configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limit: Option<RateLimitConfig>,

    /// Configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<AuthConfig>,

    /// Configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cors: Option<CorsConfig>,
}

/// Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CacheConfig {
    /// Memory cache
    #[serde(rename = "memory")]
    Memory {
        /// Maximum size
        max_size: usize,
        /// TTL（seconds）
        #[serde(with = "duration_serde")]
        ttl: Duration,
    },

    /// Redis cache
    #[serde(rename = "redis")]
    Redis {
        /// Redis URL
        url: String,
        /// TTL（seconds）
        #[serde(with = "duration_serde")]
        ttl: Duration,
        /// Connection
        #[serde(default = "default_pool_size")]
        pool_size: u32,
    },

    /// Layered cache
    #[serde(rename = "tiered")]
    Tiered {
        /// L1 cache
        l1: Box<CacheConfig>,
        /// L2 cache
        l2: Box<CacheConfig>,
        /// L3 cache（optional）
        l3: Option<Box<CacheConfig>>,
    },
}

/// Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Enabled authentication methods
    pub methods: Vec<AuthMethod>,

    /// Configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jwt: Option<JwtConfig>,

    /// Configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<ApiKeyConfig>,
}

/// Authentication methods
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthMethod {
    /// JWT Bearer Token
    Jwt,
    /// API Key
    ApiKey,
    /// Basic Auth
    Basic,
    /// Custom authentication
    Custom { handler: String },
}

/// Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    /// Signing key
    pub secret: String,

    /// Algorithm
    #[serde(default = "default_jwt_algorithm")]
    pub algorithm: String,

    /// Expiration time (seconds)
    #[serde(default = "default_jwt_expiration")]
    pub expiration_seconds: u64,

    /// Issuer
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issuer: Option<String>,

    /// Audience
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audience: Option<String>,
}

/// Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyConfig {
    /// Header name
    #[serde(default = "default_api_key_header")]
    pub header_name: String,

    /// Prefix
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,

    /// Valid API key list
    #[serde(default)]
    pub valid_keys: Vec<String>,
}

/// Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    /// Allowed origins
    pub allowed_origins: Vec<String>,

    /// Allowed methods
    #[serde(default = "default_cors_methods")]
    pub allowed_methods: Vec<String>,

    /// Allowed headers
    #[serde(default = "default_cors_headers")]
    pub allowed_headers: Vec<String>,

    /// Allow credentials
    #[serde(default)]
    pub allow_credentials: bool,

    /// Maximum age (seconds)
    #[serde(default = "default_cors_max_age")]
    pub max_age_seconds: u64,
}

/// Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    /// Configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<MetricsConfig>,

    /// Configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracing: Option<TracingConfig>,

    /// Configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logging: Option<LoggingConfig>,
}

/// Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Endpoint path
    #[serde(default = "default_metrics_endpoint")]
    pub endpoint: String,

    /// Collection interval (seconds)
    #[serde(default = "default_metrics_interval")]
    pub interval_seconds: u64,
}

/// Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingConfig {
    /// Enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Sampling rate (0.0-1.0)
    #[serde(default = "default_sampling_rate")]
    pub sampling_rate: f64,

    /// Configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jaeger: Option<JaegerConfig>,
}

/// Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JaegerConfig {
    /// Agent endpoint
    pub agent_endpoint: String,

    /// Service name
    pub service_name: String,
}

/// Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level
    #[serde(default = "default_log_level")]
    pub level: String,

    /// Output format
    #[serde(default = "default_log_format")]
    pub format: LogFormat,

    /// Output target
    #[serde(default)]
    pub outputs: Vec<LogOutput>,
}

/// Log format
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    /// Plain text
    Text,
    /// JSON format
    Json,
    /// Structured format
    Structured,
}

/// Log output
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum LogOutput {
    /// Console output
    #[serde(rename = "console")]
    Console,
    /// File output
    #[serde(rename = "file")]
    File { path: String },
    /// System log
    #[serde(rename = "syslog")]
    Syslog { facility: String },
}

// Default
fn default_true() -> bool {
    true
}
fn default_weight() -> f64 {
    1.0
}
fn default_timeout_seconds() -> u64 {
    30
}
fn default_max_retries() -> u32 {
    3
}
fn default_health_check_interval() -> u64 {
    30
}
fn default_health_check_timeout() -> u64 {
    5
}
fn default_health_threshold() -> u32 {
    2
}
fn default_unhealthy_threshold() -> u32 {
    3
}
fn default_failure_threshold() -> u32 {
    5
}
fn default_recovery_timeout() -> u64 {
    60
}
fn default_half_open_requests() -> u32 {
    3
}
fn default_session_timeout() -> u64 {
    3600
}
fn default_initial_delay_ms() -> u64 {
    100
}
fn default_max_delay_ms() -> u64 {
    30000
}
fn default_pool_size() -> u32 {
    10
}
fn default_jwt_algorithm() -> String {
    "HS256".to_string()
}
fn default_jwt_expiration() -> u64 {
    3600
}
fn default_api_key_header() -> String {
    "Authorization".to_string()
}
fn default_cors_methods() -> Vec<String> {
    vec![
        "GET".to_string(),
        "POST".to_string(),
        "PUT".to_string(),
        "DELETE".to_string(),
    ]
}
fn default_cors_headers() -> Vec<String> {
    vec!["Content-Type".to_string(), "Authorization".to_string()]
}
fn default_cors_max_age() -> u64 {
    3600
}
fn default_metrics_endpoint() -> String {
    "/metrics".to_string()
}
fn default_metrics_interval() -> u64 {
    15
}
fn default_sampling_rate() -> f64 {
    0.1
}
fn default_log_level() -> String {
    "info".to_string()
}
fn default_log_format() -> LogFormat {
    LogFormat::Json
}

// Duration serialization module
mod duration_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_secs())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            workers: None,
            max_connections: None,
            timeout: Duration::from_secs(30),
            tls: None,
            features: Vec::new(),
        }
    }
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            interval_seconds: default_health_check_interval(),
            timeout_seconds: default_health_check_timeout(),
            healthy_threshold: default_health_threshold(),
            unhealthy_threshold: default_unhealthy_threshold(),
            endpoint: None,
            enabled: true,
        }
    }
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: default_failure_threshold(),
            recovery_timeout_seconds: default_recovery_timeout(),
            half_open_max_requests: default_half_open_requests(),
            enabled: true,
        }
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: default_max_retries(),
            initial_delay_ms: default_initial_delay_ms(),
            max_delay_ms: default_max_delay_ms(),
            exponential_backoff: true,
            jitter: true,
            retryable_errors: vec![
                "network_error".to_string(),
                "timeout_error".to_string(),
                "rate_limit_error".to_string(),
                "server_error".to_string(),
            ],
        }
    }
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            metrics: Some(MetricsConfig {
                enabled: true,
                endpoint: default_metrics_endpoint(),
                interval_seconds: default_metrics_interval(),
            }),
            tracing: Some(TracingConfig {
                enabled: true,
                sampling_rate: default_sampling_rate(),
                jaeger: None,
            }),
            logging: Some(LoggingConfig {
                level: default_log_level(),
                format: default_log_format(),
                outputs: vec![LogOutput::Console],
            }),
        }
    }
}
