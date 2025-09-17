//! Types
//!
//! 定义所有 API 请求的统一数据结构

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Request
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChatRequest {
    /// Model
    pub model: String,

    /// Chat message列表
    pub messages: Vec<ChatMessage>,

    /// 采样温度 (0.0 - 2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// 生成的maximum token 数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,

    /// maximum完成 token 数（OpenAI 新parameter）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: Option<u32>,

    /// 核采样parameter (0.0 - 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    /// 频率惩罚 (-2.0 - 2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,

    /// 存在惩罚 (-2.0 - 2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,

    /// 停止序列
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,

    /// Response
    #[serde(default)]
    pub stream: bool,

    /// 工具列表
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,

    /// 工具选择策略
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,

    /// 并行tool_call
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,

    /// Response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,

    /// user_id
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,

    /// 种子值（用于可重复生成）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i32>,

    /// Returns选择count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,

    /// logit 偏置
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<HashMap<String, f32>>,

    /// 遗留函数定义 (OpenAI Functions)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub functions: Option<Vec<serde_json::Value>>,

    /// 遗留函数call (OpenAI Function Call)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_call: Option<serde_json::Value>,

    /// 是否Returns logprobs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<bool>,

    /// Returns的 top logprobs count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<u32>,

    /// 额外的 provider specific_params
    #[serde(flatten)]
    pub extra_params: HashMap<String, serde_json::Value>,
}


/// Chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// Message role
    pub role: MessageRole,

    /// Message content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<MessageContent>,

    /// message发送者名称
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// tool_call列表
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,

    /// Response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,

    /// 函数call（向后兼容）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_call: Option<FunctionCall>,
}

impl Default for ChatMessage {
    fn default() -> Self {
        Self {
            role: MessageRole::User,
            content: None,
            name: None,
            tool_calls: None,
            tool_call_id: None,
            function_call: None,
        }
    }
}

/// Message role枚举
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    /// System message
    System,
    /// User message
    User,
    /// Assistant message
    Assistant,
    /// Tool message
    Tool,
    /// 函数message（向后兼容）
    Function,
}

impl MessageRole {
    /// Check if the message role is effectively empty
    pub fn is_empty(&self) -> bool {
        // MessageRole is always non-empty by design
        false
    }
}

impl std::fmt::Display for MessageRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageRole::System => write!(f, "system"),
            MessageRole::User => write!(f, "user"),
            MessageRole::Assistant => write!(f, "assistant"),
            MessageRole::Tool => write!(f, "tool"),
            MessageRole::Function => write!(f, "function"),
        }
    }
}

/// Message content（支持多模态）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    /// Plain text content
    Text(String),
    /// 多部分content（支持文本、image、音频etc）
    Parts(Vec<ContentPart>),
}

impl std::fmt::Display for MessageContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageContent::Text(text) => write!(f, "{}", text),
            MessageContent::Parts(parts) => {
                let texts: Vec<String> = parts
                    .iter()
                    .filter_map(|part| match part {
                        ContentPart::Text { text } => Some(text.clone()),
                        ContentPart::ImageUrl { .. } => None,
                        ContentPart::Audio { .. } => None, 
                        ContentPart::Image { .. } => None,
                        ContentPart::Document { .. } => None,
                        ContentPart::ToolResult { .. } => None,
                        ContentPart::ToolUse { .. } => None,
                    })
                    .collect();
                write!(f, "{}", texts.join(" "))
            }
        }
    }
}

/// Content part（多模态支持）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentPart {
    /// Text content
    #[serde(rename = "text")]
    Text { text: String },

    /// imageURL
    #[serde(rename = "image_url")]
    ImageUrl { image_url: ImageUrl },

    /// Audio data
    #[serde(rename = "audio")]
    Audio { audio: AudioData },

    /// Base64 编码的image
    #[serde(rename = "image")]
    Image {
        /// Base64 编码的image数据
        source: ImageSource,
        /// image详细程度
        #[serde(skip_serializing_if = "Option::is_none")]
        detail: Option<String>,
        /// imageURL (兼容性字段)
        #[serde(skip_serializing_if = "Option::is_none")]
        image_url: Option<ImageUrl>,
    },
    
    /// 文档content (PDFetc)
    #[serde(rename = "document")]
    Document {
        /// 文档源数据
        source: DocumentSource,
        /// cache控制 (Anthropic specific)
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },
    
    /// 工具结果
    #[serde(rename = "tool_result")]
    ToolResult {
        /// 工具usageID
        tool_use_id: String,
        /// 结果content
        content: serde_json::Value,
        /// Error
        #[serde(skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
    },
    
    /// 工具usage
    #[serde(rename = "tool_use")]
    ToolUse {
        /// 工具usageID
        id: String,
        /// 工具名称
        name: String,
        /// 工具input
        input: serde_json::Value,
    },
}

/// imageURL结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageUrl {
    /// imageURL
    pub url: String,
    /// 详细程度 ("auto", "low", "high")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// image源数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageSource {
    /// 媒体类型
    pub media_type: String,
    /// Base64 编码的数据
    pub data: String,
}

/// Audio data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioData {
    /// Base64 编码的Audio data
    pub data: String,
    /// 音频format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
}

/// tool_type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ToolType {
    Function,
}

/// 工具定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// tool_type
    #[serde(rename = "type")]
    pub tool_type: ToolType,

    /// 函数定义
    pub function: FunctionDefinition,
}

/// 函数定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDefinition {
    /// function_name
    pub name: String,
    /// 函数描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// parameter JSON Schema
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,
}

/// 工具选择策略
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolChoice {
    /// 字符串选择 ("auto", "none")
    String(String),
    /// 具体工具选择
    Specific {
        #[serde(rename = "type")]
        choice_type: String,
        function: Option<FunctionChoice>,
    },
}

/// 具体函数选择
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionChoice {
    pub name: String,
}

/// tool_call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// callID
    pub id: String,
    /// tool_type
    #[serde(rename = "type")]
    pub tool_type: String,
    /// 函数call详情
    pub function: FunctionCall,
}

/// 函数call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    /// function_name
    pub name: String,
    /// 函数parameter（JSON 字符串）
    pub arguments: String,
}

/// Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseFormat {
    /// format类型 ("text", "json_object", "json_schema")
    #[serde(rename = "type")]
    pub format_type: String,

    /// JSON Schema（当类型为 json_schema 时）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub json_schema: Option<serde_json::Value>,

    /// Response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_type: Option<String>,
}

/// Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedRequest {
    /// Model
    pub model: String,

    /// input文本
    pub input: EmbedInput,

    /// 编码format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding_format: Option<String>,

    /// 维度
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<u32>,

    /// user_id
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

/// 嵌入input
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EmbedInput {
    /// 单个字符串
    Single(String),
    /// 字符串数组
    Multiple(Vec<String>),
    /// 整数数组（token IDs）
    TokenIds(Vec<u32>),
    /// 整数数组的数组
    MultipleTokenIds(Vec<Vec<u32>>),
}

/// Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageRequest {
    /// Model
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// 图像描述提示
    pub prompt: String,

    /// 生成图像count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,

    /// 图像质量
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<String>,

    /// Response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<String>,

    /// 图像尺寸
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,

    /// 图像风格
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,

    /// user_id
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

/// Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioTranscriptionRequest {
    /// 音频文件数据
    pub file: Vec<u8>,

    /// Model
    pub model: String,

    /// 语言（ISO-639-1 format）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    /// 提示词
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,

    /// Response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<String>,

    /// 温度
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
}

impl ChatRequest {
    /// Create
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            ..Default::default()
        }
    }

    /// 添加message
    pub fn add_message(mut self, role: MessageRole, content: impl Into<MessageContent>) -> Self {
        self.messages.push(ChatMessage {
            role,
            content: Some(content.into()),
            name: None,
            tool_calls: None,
            tool_call_id: None,
            function_call: None,
        });
        self
    }

    /// 添加System message
    pub fn add_system_message(self, content: impl Into<String>) -> Self {
        self.add_message(MessageRole::System, MessageContent::Text(content.into()))
    }

    /// 添加User message
    pub fn add_user_message(self, content: impl Into<String>) -> Self {
        self.add_message(MessageRole::User, MessageContent::Text(content.into()))
    }

    /// 添加Assistant message
    pub fn add_assistant_message(self, content: impl Into<String>) -> Self {
        self.add_message(MessageRole::Assistant, MessageContent::Text(content.into()))
    }

    /// Settings
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Settings
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Response
    pub fn with_streaming(mut self) -> Self {
        self.stream = true;
        self
    }

    /// 添加工具
    pub fn with_tools(mut self, tools: Vec<Tool>) -> Self {
        self.tools = Some(tools);
        self
    }

    /// 估算input token count
    pub fn estimate_input_tokens(&self) -> u32 {
        let mut total = 0;

        // 粗略估算：每个message的角色和content
        for message in &self.messages {
            total += 4; // message结构开销

            if let Some(content) = &message.content {
                match content {
                    MessageContent::Text(text) => {
                        total += (text.len() as f64 / 4.0).ceil() as u32;
                    }
                    MessageContent::Parts(parts) => {
                        for part in parts {
                            match part {
                                ContentPart::Text { text } => {
                                    total += (text.len() as f64 / 4.0).ceil() as u32;
                                }
                                ContentPart::ImageUrl { .. } => {
                                    total += 85; // 固定的图像 token consumption
                                }
                                ContentPart::Audio { .. } => {
                                    total += 100; // 估算的音频 token consumption
                                }
                                ContentPart::Image { .. } => {
                                    total += 85; // 固定的图像 token consumption
                                }
                                ContentPart::Document { .. } => {
                                    total += 1000; // 估算的文档 token consumption
                                }
                                ContentPart::ToolResult { .. } => {
                                    total += 50; // 工具结果 token consumption
                                }
                                ContentPart::ToolUse { .. } => {
                                    total += 100; // 工具usage token consumption
                                }
                            }
                        }
                    }
                }
            }
        }

        total
    }
}

impl From<String> for MessageContent {
    fn from(s: String) -> Self {
        Self::Text(s)
    }
}

impl From<&str> for MessageContent {
    fn from(s: &str) -> Self {
        Self::Text(s.to_string())
    }
}

/// inputAudio data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputAudio {
    /// Base64 编码的Audio data
    pub data: String,
    /// 音频format
    pub format: String,
}

/// 函数call选择
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FunctionCallChoice {
    /// 不call函数
    None,
    /// 自动决定
    Auto,
    /// 指定函数
    Function {
        /// function_name
        name: String,
    },
}

/// Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    /// Model
    pub model: String,
    /// input文本提示
    pub prompt: String,
    /// 采样温度
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// 生成的maximum token 数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    /// 核采样parameter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    /// 频率惩罚
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,
    /// 存在惩罚
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,
    /// 停止序列
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    /// Returns选择count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,
    /// Response
    #[serde(default)]
    pub stream: bool,
    /// user_id
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

/// Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingRequest {
    /// Model
    pub model: String,
    /// input文本或文本列表
    pub input: EmbeddingInput,
    /// user_id
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    /// 嵌入format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding_format: Option<String>,
    /// 维度count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<u32>,
    /// 任务类型 (用于Vertex AIetc)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_type: Option<String>,
}

/// 嵌入input类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EmbeddingInput {
    /// 单个文本
    Text(String),
    /// 文本列表
    Array(Vec<String>),
}

impl EmbeddingInput {
    /// Get
    pub fn iter(&self) -> Box<dyn Iterator<Item = &String> + '_> {
        match self {
            EmbeddingInput::Text(text) => Box::new(std::iter::once(text)),
            EmbeddingInput::Array(texts) => Box::new(texts.iter()),
        }
    }

    /// 转换为文本向量
    pub fn to_vec(&self) -> Vec<String> {
        match self {
            EmbeddingInput::Text(text) => vec![text.clone()],
            EmbeddingInput::Array(texts) => texts.clone(),
        }
    }
}

/// Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageGenerationRequest {
    /// 图像描述提示
    pub prompt: String,
    /// Model
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// 生成图像count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,
    /// 图像尺寸
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    /// 图像质量
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<String>,
    /// Response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<String>,
    /// 风格
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
    /// user_id
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

// ============================================================================
// Anthropic-specific Types
// ============================================================================

/// 文档源数据 (Anthropic PDF 支持)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSource {
    /// 媒体类型 (application/pdf)
    pub media_type: String,
    /// Base64 编码的数据
    pub data: String,
}

/// cache控制 (Anthropic Cache Control)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheControl {
    /// cache类型 ("ephemeral", "persistent")
    #[serde(rename = "type")]
    pub cache_type: String,
}

/// Settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingConfig {
    /// enabled思考模式
    pub enabled: bool,
}

/// Settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputerToolConfig {
    /// 屏幕宽度
    pub display_width: u32,
    /// 屏幕高度
    pub display_height: u32,
    /// 显示密度
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_density: Option<u32>,
}

/// Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    /// 服务器名称
    pub name: String,
    /// 服务器端点
    pub endpoint: String,
    /// 认证信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<serde_json::Value>,
}

/// Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicRequestParams {
    /// System message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    
    /// 停止序列
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,
    
    /// 顶部 K 采样
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,
    
    /// 元数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<AnthropicMetadata>,
    
    /// Configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<ThinkingConfig>,
    
    /// Configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub computer_use: Option<ComputerToolConfig>,
    
    /// MCP 服务器列表
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp_servers: Option<Vec<McpServerConfig>>,
}

/// Anthropic 元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicMetadata {
    /// 用户 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    /// 会话 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// 自定义数据
    #[serde(flatten)]
    pub custom: HashMap<String, serde_json::Value>,
}

/// 增强的 ChatRequest 以支持 Anthropic 特性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicChatRequest {
    #[serde(flatten)]
    pub base: ChatRequest,
    
    #[serde(flatten)]
    pub anthropic_params: AnthropicRequestParams,
}
