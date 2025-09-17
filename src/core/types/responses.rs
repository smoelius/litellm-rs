//! Types
//!
//! 定义所有 API 响应的统一数据结构

use serde::{Deserialize, Serialize};

use super::requests::{ChatMessage, MessageContent, MessageRole, ToolCall};

/// Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    /// Response
    pub id: String,

    /// object_type
    pub object: String,

    /// Create
    pub created: i64,

    /// Model
    pub model: String,

    /// 选择列表
    pub choices: Vec<ChatChoice>,

    /// usage情况统计
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,

    /// 系统指纹
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: Option<String>,
}

/// 聊天选择
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChoice {
    /// 选择索引
    pub index: u32,

    /// Response
    pub message: ChatMessage,

    /// 完成原因
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<FinishReason>,

    /// Log probabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<LogProbs>,
}

/// Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChunk {
    /// Response
    pub id: String,

    /// object_type
    pub object: String,

    /// Create
    pub created: i64,

    /// Model
    pub model: String,

    /// 选择列表
    pub choices: Vec<ChatStreamChoice>,

    /// usage情况（通常在最后一个块中）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,

    /// 系统指纹
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: Option<String>,
}

/// 流式选择
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatStreamChoice {
    /// 选择索引
    pub index: u32,

    /// 增量content
    pub delta: ChatDelta,

    /// 完成原因
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<FinishReason>,

    /// Log probabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<LogProbs>,
}

/// 流式增量content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatDelta {
    /// 角色（通常只在第一个块中出现）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<MessageRole>,

    /// content增量
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,

    /// tool_call增量
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCallDelta>>,

    /// 函数call增量（向后兼容）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_call: Option<FunctionCallDelta>,
}

/// tool_call增量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallDelta {
    /// 索引
    pub index: u32,

    /// callID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// tool_type
    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    pub tool_type: Option<String>,

    /// 函数call增量
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function: Option<FunctionCallDelta>,
}

/// 函数call增量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCallDelta {
    /// function_name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// parameter增量
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<String>,
}

/// 完成原因
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    /// 自然停止
    Stop,
    /// 达到长度限制
    Length,
    /// tool_call
    ToolCalls,
    /// content过滤
    ContentFilter,
    /// 函数call（向后兼容）
    FunctionCall,
}

/// usage_stats
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Usage {
    /// 提示 token 数
    pub prompt_tokens: u32,

    /// 完成 token 数
    pub completion_tokens: u32,

    /// 总 token 数
    pub total_tokens: u32,

    /// 提示 token 详细信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_tokens_details: Option<PromptTokensDetails>,

    /// 完成 token 详细信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_tokens_details: Option<CompletionTokensDetails>,
}

/// 提示 token 详细信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTokensDetails {
    /// cache的 token 数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_tokens: Option<u32>,

    /// 音频 token 数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_tokens: Option<u32>,
}

/// 完成 token 详细信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionTokensDetails {
    /// 推理 token 数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_tokens: Option<u32>,

    /// 音频 token 数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_tokens: Option<u32>,
}

/// Log probabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogProbs {
    /// Token 的 log probabilities
    pub content: Vec<TokenLogProb>,

    /// 拒绝采样信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refusal: Option<String>,
}

/// 单个 token 的 log probability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenLogProb {
    /// Token 文本
    pub token: String,

    /// Log probability
    pub logprob: f64,

    /// Token 的字节表示
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes: Option<Vec<u8>>,

    /// Top log probabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<Vec<TopLogProb>>,
}

/// Top log probability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopLogProb {
    /// Token 文本
    pub token: String,

    /// Log probability
    pub logprob: f64,

    /// Token 的字节表示
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes: Option<Vec<u8>>,
}

/// Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedResponse {
    /// object_type
    pub object: String,

    /// 嵌入数据列表
    pub data: Vec<EmbeddingData>,

    /// Model
    pub model: String,

    /// usage_stats
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<EmbeddingUsage>,
}

/// 嵌入数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingData {
    /// object_type
    pub object: String,

    /// 索引
    pub index: u32,

    /// 嵌入向量
    pub embedding: Vec<f32>,
}

/// 嵌入usage_stats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingUsage {
    /// 提示 token 数
    pub prompt_tokens: u32,

    /// 总 token 数
    pub total_tokens: u32,
}

/// Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageResponse {
    /// Create
    pub created: i64,

    /// 图像数据列表
    pub data: Vec<ImageData>,
}

/// 图像数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageData {
    /// 图像URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// Base64 编码的图像
    #[serde(skip_serializing_if = "Option::is_none")]
    pub b64_json: Option<String>,

    /// 修订提示（如果有）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revised_prompt: Option<String>,
}

/// Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioTranscriptionResponse {
    /// 转录文本
    pub text: String,

    /// 语言
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    /// 持续时间
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<f64>,

    /// 详细信息（当启用时）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub words: Option<Vec<WordInfo>>,

    /// 段落信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub segments: Option<Vec<SegmentInfo>>,
}

/// 单词信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordInfo {
    /// 单词文本
    pub word: String,

    /// 开始时间
    pub start: f64,

    /// 结束时间
    pub end: f64,
}

/// 段落信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentInfo {
    /// 段落 ID
    pub id: u32,

    /// 开始时间
    pub start: f64,

    /// 结束时间
    pub end: f64,

    /// Text content
    pub text: String,

    /// 温度
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// 平均 log probability
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_logprob: Option<f64>,

    /// 压缩比
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compression_ratio: Option<f64>,

    /// 无语音概率
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_speech_prob: Option<f64>,
}

/// Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: ApiError,
}

/// Error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    /// Error
    pub message: String,

    /// Error
    #[serde(rename = "type")]
    pub error_type: String,

    /// Error
    #[serde(skip_serializing_if = "Option::is_none")]
    pub param: Option<String>,

    /// Error
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

impl ChatResponse {
    /// Get
    pub fn first_content(&self) -> Option<&str> {
        self.choices
            .first()
            .and_then(|choice| match &choice.message.content {
                Some(MessageContent::Text(text)) => Some(text.as_str()),
                _ => None,
            })
    }

    /// Get
    pub fn all_content(&self) -> Vec<&str> {
        self.choices
            .iter()
            .filter_map(|choice| match &choice.message.content {
                Some(MessageContent::Text(text)) => Some(text.as_str()),
                _ => None,
            })
            .collect()
    }

    /// Check
    pub fn has_tool_calls(&self) -> bool {
        self.choices
            .iter()
            .any(|choice| choice.message.tool_calls.is_some())
    }

    /// Get
    pub fn first_tool_calls(&self) -> Option<&[ToolCall]> {
        self.choices
            .first()
            .and_then(|choice| choice.message.tool_calls.as_ref())
            .map(|calls| calls.as_slice())
    }

    /// 计算总成本（需要价格信息）
    pub fn calculate_cost(&self, input_cost_per_1k: f64, output_cost_per_1k: f64) -> f64 {
        if let Some(usage) = &self.usage {
            let input_cost = (usage.prompt_tokens as f64 / 1000.0) * input_cost_per_1k;
            let output_cost = (usage.completion_tokens as f64 / 1000.0) * output_cost_per_1k;
            input_cost + output_cost
        } else {
            0.0
        }
    }
}

impl Default for ChatResponse {
    fn default() -> Self {
        Self {
            id: format!("chatcmpl-{}", uuid::Uuid::new_v4().simple()),
            object: "chat.completion".to_string(),
            created: chrono::Utc::now().timestamp(),
            model: String::new(),
            choices: Vec::new(),
            usage: None,
            system_fingerprint: None,
        }
    }
}


impl Usage {
    pub fn new(prompt_tokens: u32, completion_tokens: u32) -> Self {
        Self {
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
            prompt_tokens_details: None,
            completion_tokens_details: None,
        }
    }
}

/// Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    /// Response
    pub id: String,
    /// object_type
    pub object: String,
    /// Create
    pub created: i64,
    /// Model
    pub model: String,
    /// 选择列表
    pub choices: Vec<CompletionChoice>,
    /// usage_stats
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
    /// 系统指纹
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: Option<String>,
}

/// 补全选择
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionChoice {
    /// 选择索引
    pub index: u32,
    /// 生成的文本
    pub text: String,
    /// 完成原因
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<FinishReason>,
    /// Log概率信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<LogProbs>,
}

/// Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    /// object_type
    pub object: String,
    /// 嵌入数据列表
    pub data: Vec<EmbeddingData>,
    /// Model
    pub model: String,
    /// usage_stats
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
    /// 嵌入数据列表 (向后兼容字段)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embeddings: Option<Vec<EmbeddingData>>,
}

// EmbeddingData already defined earlier in this file

/// Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageGenerationResponse {
    /// Create
    pub created: u64,
    /// 生成的图像列表
    pub data: Vec<ImageData>,
}

// ImageData already defined earlier in this file
