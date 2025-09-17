//! LLM Provider 核心 trait 定义
//!
//! Implementation

use async_trait::async_trait;
use futures::Stream;
use std::fmt::Debug;
use std::pin::Pin;

use crate::core::types::{
    common::{HealthStatus, ModelInfo, ProviderCapability, RequestContext},
    requests::{ChatRequest, EmbeddingRequest, ImageGenerationRequest},
    responses::{ChatChunk, ChatResponse, EmbeddingResponse, ImageGenerationResponse},
};

use super::error_mapper::ErrorMapper;
use crate::core::types::errors::ProviderErrorTrait;
use serde_json::Value;
use std::collections::HashMap;

/// LLM Provider 统一接口
///
/// 这是 LiteLLM 的核心抽象，所有 AI 提供商都必须implementation此 trait
///
/// # 设计原则
///
/// Request
/// 2. **能力驱动**: 通过 capabilities() 声明支持的功能
/// Default
/// 4. **类型安全**: usage关联类型确保编译时类型安全
/// 5. **异步优先**: 所有 I/O 操作都是异步的
/// 6. **可观测性**: 内置成本计算、延迟统计etc监控功能
///
/// # 示例
///
/// ```rust
/// use async_trait::async_trait;
///
/// #[async_trait]
/// impl LLMProvider for MyProvider {
///     type Config = MyConfig;
///     type Error = MyError;
///     
///     fn name(&self) -> &'static str {
///         "my_provider"
///     }
///     
///     fn capabilities(&self) -> &'static [ProviderCapability] {
///         &[ProviderCapability::ChatCompletion]
///     }
///     
///     // implementation其他必需方法...
/// }
/// ```
#[async_trait]
pub trait LLMProvider: Send + Sync + Debug + 'static {
    /// Configuration
    ///
    /// Configuration
    type Config: ProviderConfig + Clone + Send + Sync;

    /// Error
    ///
    /// Error
    type Error: ProviderErrorTrait;

    /// Error
    ///
    /// Error
    type ErrorMapper: ErrorMapper<Self::Error>;

    // ==================== 基础元数据 ====================

    /// Get
    ///
    /// # Returns
    /// Provider 的静态字符串标识符，如 "openai", "anthropic", "v0" etc
    ///
    /// # 注意
    /// 此名称用于路由和日志记录，必须在整个系统中唯一
    fn name(&self) -> &'static str;

    /// Get
    ///
    /// # Returns
    /// 静态能力列表，用于快速查询该 provider 支持哪些功能
    ///
    /// # purpose
    /// Request
    /// Check
    /// - UI 显示：展示 provider 的功能特性
    fn capabilities(&self) -> &'static [ProviderCapability];

    /// Model
    ///
    /// # Returns
    /// Model
    ///
    /// # implementation建议
    /// Configuration
    /// Get
    /// - 建议cache以提高性能
    fn models(&self) -> &[ModelInfo];

    // ==================== 能力查询方法 ====================

    /// Check
    ///
    /// # parameter
    /// Model
    ///
    /// # Returns
    /// Model
    ///
    /// Default
    /// 在 models() Returns的列表中查找
    fn supports_model(&self, model: &str) -> bool {
        self.models().iter().any(|m| m.id == model)
    }

    /// Check
    ///
    /// # Returns
    /// 如果支持tool_callReturns true
    ///
    /// Default
    /// Check
    fn supports_tools(&self) -> bool {
        self.capabilities()
            .contains(&ProviderCapability::ToolCalling)
    }

    /// Response
    ///
    /// # Returns
    /// 如果支持 Server-Sent Events 流式outputReturns true
    ///
    /// Default
    /// Check
    fn supports_streaming(&self) -> bool {
        self.capabilities()
            .contains(&ProviderCapability::ChatCompletionStream)
    }

    /// Check
    ///
    /// # Returns
    /// 如果支持图像生成（如 DALL-E）Returns true
    fn supports_image_generation(&self) -> bool {
        self.capabilities()
            .contains(&ProviderCapability::ImageGeneration)
    }

    /// Check
    ///
    /// # Returns
    /// 如果支持生成文本嵌入向量Returns true
    fn supports_embeddings(&self) -> bool {
        self.capabilities()
            .contains(&ProviderCapability::Embeddings)
    }

    /// Check
    ///
    /// # Returns
    /// Handle
    fn supports_vision(&self) -> bool {
        // 暂时Returns false，因为 ProviderCapability 中没有 Vision 变体
        false
    }

    // ==================== Python LiteLLM 兼容接口 ====================

    /// Get
    ///
    /// Returns该 provider 支持的所有 OpenAI 标准parameter名称
    ///
    /// # parameter
    /// Model
    ///
    /// # Returns
    /// 支持的parameter名称列表
    ///
    /// # 示例
    /// ```
    /// // OpenAI provider 可能Returns：
    /// // ["temperature", "max_tokens", "top_p", "frequency_penalty", "presence_penalty", "tools"]
    /// //
    /// // Anthropic provider 可能Returns：
    /// // ["temperature", "max_tokens", "top_p", "top_k", "tools"]
    /// ```
    fn get_supported_openai_params(&self, model: &str) -> &'static [&'static str];

    /// 映射 OpenAI parameter到 provider specific_params
    ///
    /// 将标准的 OpenAI parameter转换为该 provider 能理解的format
    ///
    /// # parameter
    /// * `params` - input的parameter映射（OpenAI format）
    /// Model
    ///
    /// # Returns
    /// 转换后的parameter映射（provider 特定format）
    ///
    /// # 示例
    /// ```
    /// // 对于 Anthropic provider：
    /// // input: {"max_tokens": 100, "temperature": 0.7}
    /// // output: {"max_tokens_to_sample": 100, "temperature": 0.7}
    /// //
    /// // 对于 Azure provider：
    /// // input: {"user": "alice", "stream": true}
    /// // output: {"end_user_id": "alice", "stream": true}
    /// ```
    async fn map_openai_params(
        &self,
        params: HashMap<String, Value>,
        model: &str,
    ) -> Result<HashMap<String, Value>, Self::Error>;

    /// Request
    ///
    /// Request
    ///
    /// # parameter
    /// Request
    /// Request
    ///
    /// # Returns
    /// Request
    ///
    /// # implementation说明
    /// 此方法应该：
    /// Request
    /// 2. 转换messageformat
    /// Model
    /// Handle
    /// Settings
    async fn transform_request(
        &self,
        request: ChatRequest,
        context: RequestContext,
    ) -> Result<Value, Self::Error>;

    /// Response
    ///
    /// Response
    ///
    /// # parameter
    /// Response
    /// Model
    /// Request
    ///
    /// # Returns
    /// Response
    ///
    /// # implementation说明
    /// 此方法应该：
    /// Response
    /// 2. 提取选择项和message
    /// 3. 转换tool_callformat
    /// 4. 统计 token usage量
    /// Response
    async fn transform_response(
        &self,
        raw_response: &[u8],
        model: &str,
        request_id: &str,
    ) -> Result<ChatResponse, Self::Error>;

    /// Error
    ///
    /// Error
    ///
    /// # Returns
    /// Handle
    fn get_error_mapper(&self) -> Self::ErrorMapper;

    // ==================== 核心功能：聊天完成 ====================

    /// Request
    ///
    /// 这是所有 LLM provider 必须implementation的核心方法
    ///
    /// # parameter
    /// Request
    /// Request
    ///
    /// # Returns
    /// Response
    ///
    /// Error
    /// * `Self::Error::authentication()` - 认证失败
    /// Model
    /// Request
    /// * `Self::Error::rate_limit()` - 达到速率限制
    /// Error
    async fn chat_completion(
        &self,
        request: ChatRequest,
        context: RequestContext,
    ) -> Result<ChatResponse, Self::Error>;

    /// Request
    ///
    /// Response
    ///
    /// # parameter
    /// Request
    /// Request
    ///
    /// # Returns
    /// Returns一个 Stream，每个 item 是 ChatChunk
    ///
    /// Default
    /// Error
    ///
    /// # 注意
    /// 只有在 supports_streaming() Returns true 时才应该call此方法
    async fn chat_completion_stream(
        &self,
        _request: ChatRequest,
        _context: RequestContext,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatChunk, Self::Error>> + Send>>, Self::Error>
    {
        Err(Self::Error::not_supported("streaming"))
    }

    // ==================== optional功能 ====================

    /// 生成文本嵌入向量
    ///
    /// 将文本转换为高维向量，用于语义搜索、聚类etc应用
    ///
    /// # parameter
    /// Handle
    /// Request
    ///
    /// # Returns
    /// Response
    ///
    /// Default
    /// Error
    ///
    /// # usage场景
    /// - 语义搜索
    /// - 文档相似度计算
    /// - 推荐系统
    /// - RAG（检索增强生成）系统
    async fn embeddings(
        &self,
        _request: EmbeddingRequest,
        _context: RequestContext,
    ) -> Result<EmbeddingResponse, Self::Error> {
        Err(Self::Error::not_supported("embeddings"))
    }

    /// 生成图像
    ///
    /// 根据文本描述生成图像
    ///
    /// # parameter
    /// Request
    /// Request
    ///
    /// # Returns
    /// Response
    ///
    /// Default
    /// Error
    ///
    /// Model
    /// - OpenAI DALL-E 系列
    /// - Midjourney（通过代理）
    /// - Stable Diffusion
    async fn image_generation(
        &self,
        _request: ImageGenerationRequest,
        _context: RequestContext,
    ) -> Result<ImageGenerationResponse, Self::Error> {
        Err(Self::Error::not_supported("image_generation"))
    }

    // ==================== 健康监控 ====================

    /// Check
    ///
    /// Validation
    ///
    /// # Returns
    /// HealthStatus 枚举，包含 Healthy, Degraded, Unhealthy etc状态
    ///
    /// # implementation建议
    /// Check
    /// Validation
    /// Request
    /// Check
    ///
    /// # purpose
    /// Configuration
    /// Check
    /// - 故障转移决策
    /// - 监控告警
    async fn health_check(&self) -> HealthStatus;

    // ==================== 成本管理 ====================

    /// Request
    ///
    /// # parameter
    /// Model
    /// * `input_tokens` - input token count
    /// * `output_tokens` - output token count
    ///
    /// # Returns
    /// 预估成本（美元）
    ///
    /// # purpose
    /// - 成本控制和预算管理
    /// - 用户配额管理
    /// - 成本优化决策
    /// - 计费和统计
    async fn calculate_cost(
        &self,
        model: &str,
        input_tokens: u32,
        output_tokens: u32,
    ) -> Result<f64, Self::Error>;

    // ==================== 性能指标 ====================

    /// Response
    ///
    /// # Returns
    /// Response
    ///
    /// Default
    /// Returns 100ms，子类应该基于实际统计数据覆盖此方法
    ///
    /// # purpose
    /// - 路由选择：优先选择延迟较低的 provider
    /// Settings
    /// - 性能监控和优化
    async fn get_average_latency(&self) -> Result<std::time::Duration, Self::Error> {
        Ok(std::time::Duration::from_millis(100))
    }

    /// Get
    ///
    /// # Returns
    /// 0.0 到 1.0 之间的成功率
    ///
    /// Default
    /// Returns 0.99（99% 成功率）
    ///
    /// # purpose
    /// - 服务质量评估
    /// - 自动故障转移
    /// - SLA 监控
    async fn get_success_rate(&self) -> Result<f32, Self::Error> {
        Ok(0.99)
    }

    // ==================== 工具方法 ====================

    /// 预估文本的 token count
    ///
    /// # parameter
    /// * `text` - 待分析的文本
    ///
    /// # Returns
    /// 预估的 token count
    ///
    /// Default
    /// usage简单的启发式算法：平均 4 个字符约etc于 1 个 token
    ///
    /// # implementation建议
    /// Model
    /// - OpenAI：usage tiktoken 库
    /// - Anthropic：usage Claude tokenizer
    /// - 其他：可以usage在线 API 或者简单估算
    ///
    /// # purpose
    /// Request
    /// - 成本预估
    /// Handle
    async fn estimate_tokens(&self, text: &str) -> Result<u32, Self::Error> {
        // 简单估算：4 个字符约etc于 1 个 token
        // 子类应该implementation更精确的 tokenization
        Ok((text.len() as f64 / 4.0).ceil() as u32)
    }
}

/// Configuration
pub trait ProviderConfig: Send + Sync + Clone + Debug + 'static {
    /// Configuration
    fn validate(&self) -> Result<(), String>;

    /// Get
    fn api_key(&self) -> Option<&str>;

    /// Get
    fn api_base(&self) -> Option<&str>;

    /// Get
    fn timeout(&self) -> std::time::Duration;

    /// Get
    fn max_retries(&self) -> u32;
}

/// Provider 句柄，用于路由系统
pub struct ProviderHandle {
    name: String,
    provider: std::sync::Arc<dyn std::any::Any + Send + Sync>,
    weight: f64,
    enabled: bool,
}

impl ProviderHandle {
    pub fn new<P>(provider: P, weight: f64) -> Self
    where
        P: LLMProvider + Send + Sync + 'static,
    {
        Self {
            name: provider.name().to_string(),
            provider: std::sync::Arc::new(provider)
                as std::sync::Arc<dyn std::any::Any + Send + Sync>,
            weight,
            enabled: true,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn weight(&self) -> f64 {
        self.weight
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub async fn chat_completion(
        &self,
        _request: ChatRequest,
        _context: RequestContext,
    ) -> Result<ChatResponse, Box<dyn std::error::Error + Send + Sync>> {
        // This is a simplified implementation - in a real system,
        // you'd need to properly downcast and handle the provider
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Provider chat_completion not implemented",
        )))
    }

    pub fn supports_model(&self, _model: &str) -> bool {
        // Simplified implementation
        true
    }

    pub fn supports_tools(&self) -> bool {
        // Simplified implementation
        true
    }

    pub async fn health_check(&self) -> HealthStatus {
        // Simplified implementation
        HealthStatus::Healthy
    }

    pub async fn calculate_cost(
        &self,
        _model: &str,
        _input: u32,
        _output: u32,
    ) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
        // Simplified implementation
        Ok(0.0)
    }

    pub async fn get_average_latency(
        &self,
    ) -> Result<std::time::Duration, Box<dyn std::error::Error + Send + Sync>> {
        // Simplified implementation
        Ok(std::time::Duration::from_millis(100))
    }

    pub async fn get_success_rate(&self) -> Result<f32, Box<dyn std::error::Error + Send + Sync>> {
        // Simplified implementation
        Ok(1.0)
    }
}
