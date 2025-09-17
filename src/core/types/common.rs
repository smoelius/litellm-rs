//! Types
//!
//! 定义系统中usage的通用数据结构和枚举

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

    /// 用户 ID
    pub user_id: Option<String>,

    /// 客户端 IP
    pub client_ip: Option<String>,

    /// 用户代理
    pub user_agent: Option<String>,

    /// 自定义头部
    pub headers: HashMap<String, String>,

    /// 开始时间
    pub start_time: SystemTime,

    /// 额外的元数据
    pub metadata: HashMap<String, serde_json::Value>,

    /// 追踪 ID（用于分布式追踪）
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
    /// Create
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

    /// 添加头部
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// 添加元数据
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

/// Provider 能力枚举
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderCapability {
    /// 聊天完成
    ChatCompletion,
    /// 流式聊天完成
    ChatCompletionStream,
    /// 嵌入生成
    Embeddings,
    /// 图像生成
    ImageGeneration,
    /// 图像编辑
    ImageEdit,
    /// 图像变化
    ImageVariation,
    /// 音频转录
    AudioTranscription,
    /// 音频翻译
    AudioTranslation,
    /// 语音合成
    TextToSpeech,
    /// tool_call
    ToolCalling,
    /// 函数call（向后兼容）
    FunctionCalling,
    /// 代码执行
    CodeExecution,
    /// 文件上传
    FileUpload,
    /// 微调
    FineTuning,
    /// Handle
    BatchProcessing,
    /// 实时 API
    RealtimeApi,
}

/// Model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Model
    pub id: String,

    /// Model
    pub name: String,

    /// 提供商
    pub provider: String,

    /// maximum上下文长度
    pub max_context_length: u32,

    /// maximumoutput长度
    pub max_output_length: Option<u32>,

    /// 是否支持流式
    pub supports_streaming: bool,

    /// 是否支持tool_call
    pub supports_tools: bool,

    /// 是否支持多模态
    pub supports_multimodal: bool,

    /// input价格（每 1K token）
    pub input_cost_per_1k_tokens: Option<f64>,

    /// output价格（每 1K token）
    pub output_cost_per_1k_tokens: Option<f64>,

    /// 货币单位
    pub currency: String,

    /// 支持的功能
    pub capabilities: Vec<ProviderCapability>,

    /// Create
    pub created_at: Option<SystemTime>,

    /// Update
    pub updated_at: Option<SystemTime>,

    /// 额外的元数据
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

/// 健康状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    /// 健康
    Healthy,
    /// 不健康
    Unhealthy,
    /// 未知状态
    Unknown,
    /// 降级服务
    Degraded,
}

impl Default for HealthStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    /// 状态
    pub status: HealthStatus,

    /// Check
    pub checked_at: SystemTime,

    /// 延迟（milliseconds）
    pub latency_ms: Option<u64>,

    /// Error
    pub error: Option<String>,

    /// 额外的详情
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

/// 指标数据点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricDataPoint {
    /// 指标名称
    pub name: String,

    /// 指标值
    pub value: f64,

    /// 标签
    pub labels: HashMap<String, String>,

    /// 时间戳
    pub timestamp: SystemTime,
}

/// 指标类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MetricType {
    /// 计数器
    Counter,
    /// 仪表盘
    Gauge,
    /// 直方图
    Histogram,
    /// 摘要
    Summary,
}

/// 指标定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricDefinition {
    /// 指标名称
    pub name: String,

    /// 指标类型
    pub metric_type: MetricType,

    /// 描述
    pub description: String,

    /// 单位
    pub unit: Option<String>,

    /// 标签
    pub labels: Vec<String>,
}

/// cache键类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheKey {
    /// cache类型
    pub cache_type: String,

    /// 键值
    pub key: String,

    /// 额外的标识符
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApiVersion {
    /// v1 version
    V1,
    /// v2 version（未来扩展）
    V2,
}

impl Default for ApiVersion {
    fn default() -> Self {
        Self::V1
    }
}

impl std::fmt::Display for ApiVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::V1 => write!(f, "v1"),
            Self::V2 => write!(f, "v2"),
        }
    }
}

/// 服务状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStatus {
    /// 服务名称
    pub name: String,

    /// version
    pub version: String,

    /// 状态
    pub status: HealthStatus,

    /// 启动时间
    pub uptime: SystemTime,

    /// Connection
    pub active_connections: u32,

    /// Handle
    pub requests_processed: u64,

    /// Error
    pub errors: u64,

    /// Response
    pub avg_response_time_ms: f64,

    /// 内存usage（字节）
    pub memory_usage_bytes: u64,

    /// CPU usage率（百分比）
    pub cpu_usage_percent: f64,
}

/// 分页parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    /// 页码（从 1 开始）
    pub page: u32,

    /// 每页大小
    pub per_page: u32,

    /// 总数
    pub total: Option<u64>,

    /// 总页数
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
    /// 计算偏移量
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

/// 排序parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortOrder {
    /// 排序字段
    pub field: String,

    /// 排序方向
    pub direction: SortDirection,
}

/// 排序方向
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortDirection {
    /// 升序
    Asc,
    /// 降序
    Desc,
}

impl Default for SortDirection {
    fn default() -> Self {
        Self::Asc
    }
}

/// 过滤条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Filter {
    /// 字段名
    pub field: String,

    /// 操作符
    pub operator: FilterOperator,

    /// 值
    pub value: serde_json::Value,
}

/// 过滤操作符
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterOperator {
    /// etc于
    Eq,
    /// 不etc于
    Ne,
    /// 大于
    Gt,
    /// 大于etc于
    Gte,
    /// 小于
    Lt,
    /// 小于etc于
    Lte,
    /// 包含
    Contains,
    /// 不包含
    NotContains,
    /// 在列表中
    In,
    /// 不在列表中
    NotIn,
    /// 以...开始
    StartsWith,
    /// 以...结束
    EndsWith,
    /// 正则匹配
    Regex,
    /// 为空
    IsNull,
    /// 不为空
    IsNotNull,
}
