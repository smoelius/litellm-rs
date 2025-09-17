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
    /// 监听地址
    pub host: String,

    /// 监听端口
    pub port: u16,

    /// 工作线程数
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

    /// 启用的功能
    #[serde(default)]
    pub features: Vec<String>,
}

/// Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// 证书文件路径
    pub cert_file: String,

    /// 私钥文件路径
    pub key_file: String,

    /// enabled HTTP/2
    #[serde(default)]
    pub http2: bool,
}

/// Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfigEntry {
    /// Provider 名称（唯一标识）
    pub name: String,

    /// Provider 类型
    pub provider_type: String,

    /// enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// 路由权重 (0.0-1.0)
    #[serde(default = "default_weight")]
    pub weight: f64,

    /// Configuration
    pub config: serde_json::Value,

    /// 标签（用于路由和过滤）
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
    /// API 密钥
    pub api_key: String,

    /// API 基础 URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_base: Option<String>,

    /// 组织 ID
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

    /// 自定义头部
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
    /// 路由策略
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
    /// 轮询策略
    #[serde(rename = "round_robin")]
    RoundRobin,

    /// 最少负载策略
    #[serde(rename = "least_loaded")]
    LeastLoaded,

    /// 成本优化策略
    #[serde(rename = "cost_optimized")]
    CostOptimized {
        /// 性能权重 (0.0-1.0)
        performance_weight: f32,
    },

    /// 延迟优化策略
    #[serde(rename = "latency_based")]
    LatencyBased {
        /// 延迟阈值（milliseconds）
        latency_threshold_ms: u64,
    },

    /// 标签路由策略
    #[serde(rename = "tag_based")]
    TagBased {
        /// 标签选择器
        selectors: Vec<TagSelector>,
    },

    /// 自定义策略
    #[serde(rename = "custom")]
    Custom {
        /// 策略类名
        class: String,
        /// Configuration
        config: serde_json::Value,
    },
}

/// 标签选择器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagSelector {
    /// 标签键
    pub key: String,

    /// 标签值（支持通配符）
    pub value: String,

    /// 操作符
    #[serde(default)]
    pub operator: TagOperator,
}

/// 标签操作符
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TagOperator {
    /// etc于
    Eq,
    /// 不etc于
    Ne,
    /// 包含
    In,
    /// 不包含
    NotIn,
    /// 存在
    Exists,
    /// 不存在
    NotExists,
}

impl Default for TagOperator {
    fn default() -> Self {
        Self::Eq
    }
}

/// Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Check
    #[serde(default = "default_health_check_interval")]
    pub interval_seconds: u64,

    /// 超时时间（seconds）
    #[serde(default = "default_health_check_timeout")]
    pub timeout_seconds: u64,

    /// 健康阈值（连续成功次数）
    #[serde(default = "default_health_threshold")]
    pub healthy_threshold: u32,

    /// 不健康阈值（连续失败次数）
    #[serde(default = "default_unhealthy_threshold")]
    pub unhealthy_threshold: u32,

    /// Check
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,

    /// enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
}

/// Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// 失败阈值
    #[serde(default = "default_failure_threshold")]
    pub failure_threshold: u32,

    /// 恢复超时时间（seconds）
    #[serde(default = "default_recovery_timeout")]
    pub recovery_timeout_seconds: u64,

    /// Request
    #[serde(default = "default_half_open_requests")]
    pub half_open_max_requests: u32,

    /// enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
}

/// Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancerConfig {
    /// 算法类型
    pub algorithm: LoadBalancerAlgorithm,

    /// Configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_affinity: Option<SessionAffinityConfig>,
}

/// 负载均衡算法
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LoadBalancerAlgorithm {
    /// 轮询
    RoundRobin,
    /// 加权轮询
    WeightedRoundRobin,
    /// Connection
    LeastConnections,
    /// 一致性哈希
    ConsistentHash,
}

/// Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionAffinityConfig {
    /// 粘性类型
    pub affinity_type: SessionAffinityType,

    /// 超时时间（seconds）
    #[serde(default = "default_session_timeout")]
    pub timeout_seconds: u64,
}

/// 会话粘性类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionAffinityType {
    /// 基于客户端 IP
    ClientIp,
    /// 基于用户 ID
    UserId,
    /// 基于自定义头部
    CustomHeader { header_name: String },
}

/// Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// maximumNumber of retries
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// 初始延迟（milliseconds）
    #[serde(default = "default_initial_delay_ms")]
    pub initial_delay_ms: u64,

    /// maximum延迟（milliseconds）
    #[serde(default = "default_max_delay_ms")]
    pub max_delay_ms: u64,

    /// 是否usage指数退避
    #[serde(default = "default_true")]
    pub exponential_backoff: bool,

    /// 是否添加随机抖动
    #[serde(default = "default_true")]
    pub jitter: bool,

    /// Error
    #[serde(default)]
    pub retryable_errors: Vec<String>,
}

/// Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// 算法类型
    pub algorithm: RateLimitAlgorithm,

    /// Request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requests_per_second: Option<u32>,

    /// Request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requests_per_minute: Option<u32>,

    /// Token 速率限制（每分钟）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens_per_minute: Option<u32>,

    /// 突发限制
    #[serde(skip_serializing_if = "Option::is_none")]
    pub burst_size: Option<u32>,
}

/// 速率限制算法
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RateLimitAlgorithm {
    /// 令牌桶
    TokenBucket,
    /// 滑动窗口
    SlidingWindow,
    /// 固定窗口
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
    /// 内存cache
    #[serde(rename = "memory")]
    Memory {
        /// maximum大小
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

    /// 分层cache
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
    /// 启用的认证方式
    pub methods: Vec<AuthMethod>,

    /// Configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jwt: Option<JwtConfig>,

    /// Configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<ApiKeyConfig>,
}

/// 认证方式
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthMethod {
    /// JWT Bearer Token
    Jwt,
    /// API Key
    ApiKey,
    /// Basic Auth
    Basic,
    /// 自定义认证
    Custom { handler: String },
}

/// Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    /// 签名密钥
    pub secret: String,

    /// 算法
    #[serde(default = "default_jwt_algorithm")]
    pub algorithm: String,

    /// 过期时间（seconds）
    #[serde(default = "default_jwt_expiration")]
    pub expiration_seconds: u64,

    /// 发行人
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issuer: Option<String>,

    /// 受众
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audience: Option<String>,
}

/// Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyConfig {
    /// 头部名称
    #[serde(default = "default_api_key_header")]
    pub header_name: String,

    /// 前缀
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,

    /// 有效的 API 密钥列表
    #[serde(default)]
    pub valid_keys: Vec<String>,
}

/// Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    /// 允许的源
    pub allowed_origins: Vec<String>,

    /// 允许的方法
    #[serde(default = "default_cors_methods")]
    pub allowed_methods: Vec<String>,

    /// 允许的头部
    #[serde(default = "default_cors_headers")]
    pub allowed_headers: Vec<String>,

    /// 是否允许凭证
    #[serde(default)]
    pub allow_credentials: bool,

    /// maximum年龄（seconds）
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
    /// enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// 端点路径
    #[serde(default = "default_metrics_endpoint")]
    pub endpoint: String,

    /// 收集间隔（seconds）
    #[serde(default = "default_metrics_interval")]
    pub interval_seconds: u64,
}

/// Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingConfig {
    /// enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// 采样率 (0.0-1.0)
    #[serde(default = "default_sampling_rate")]
    pub sampling_rate: f64,

    /// Configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jaeger: Option<JaegerConfig>,
}

/// Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JaegerConfig {
    /// Agent 端点
    pub agent_endpoint: String,

    /// 服务名称
    pub service_name: String,
}

/// Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// 日志级别
    #[serde(default = "default_log_level")]
    pub level: String,

    /// outputformat
    #[serde(default = "default_log_format")]
    pub format: LogFormat,

    /// output目标
    #[serde(default)]
    pub outputs: Vec<LogOutput>,
}

/// 日志format
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    /// 纯文本
    Text,
    /// JSON format
    Json,
    /// 结构化format
    Structured,
}

/// 日志output
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum LogOutput {
    /// 控制台output
    #[serde(rename = "console")]
    Console,
    /// 文件output
    #[serde(rename = "file")]
    File { path: String },
    /// 系统日志
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

// Duration 序列化模块
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
