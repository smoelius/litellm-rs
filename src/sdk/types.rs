//! SDK data types

use serde::{Deserialize, Serialize};

/// Message role
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// System message
    System,
    /// User message
    User,
    /// Assistant message
    Assistant,
    /// Tool message
    Tool,
}

/// Message content type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Content {
    /// Plain text content
    Text(String),
    /// Multimodal content
    Multimodal(Vec<ContentPart>),
}

/// Content part
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentPart {
    /// Text content
    #[serde(rename = "text")]
    Text {
        /// Text string
        text: String,
    },
    /// Image content
    #[serde(rename = "image_url")]
    Image {
        /// Image URL information
        image_url: ImageUrl,
    },
    /// Audio content
    #[serde(rename = "audio")]
    Audio {
        /// Audio data
        audio: AudioData,
    },
}

/// imageURL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageUrl {
    /// imageURL或base64数据
    pub url: String,
    /// image详细度
    pub detail: Option<String>,
}

/// Audio data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioData {
    /// Audio data或URL
    pub data: String,
    /// 音频format
    pub format: Option<String>,
}

/// Chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Message role
    pub role: Role,
    /// Message content
    pub content: Option<Content>,
    /// message名称
    pub name: Option<String>,
    /// tool_call
    pub tool_calls: Option<Vec<ToolCall>>,
}

/// tool_call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// callID
    pub id: String,
    /// tool_type
    #[serde(rename = "type")]
    pub tool_type: String,
    /// 函数call
    pub function: Function,
}

/// 函数定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Function {
    /// function_name
    pub name: String,
    /// 函数描述
    pub description: Option<String>,
    /// 函数parameterSchema
    pub parameters: serde_json::Value,
    /// 函数parameter（用于call）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<String>,
}

/// 工具定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// tool_type
    #[serde(rename = "type")]
    pub tool_type: String,
    /// 函数定义
    pub function: Function,
}

/// 工具选择
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolChoice {
    /// 不usage工具
    None,
    /// 自动选择
    Auto,
    /// 必须usage工具
    Required,
    /// 指定函数
    Function {
        /// function_name
        name: String,
    },
}

/// Request
#[derive(Debug, Clone)]
pub struct ChatRequest {
    /// Model
    pub model: String,
    /// message列表
    pub messages: Vec<Message>,
    /// Request
    pub options: ChatOptions,
}

/// 聊天选项
#[derive(Debug, Clone, Default)]
pub struct ChatOptions {
    /// 温度parameter
    pub temperature: Option<f32>,
    /// maximumtoken数
    pub max_tokens: Option<u32>,
    /// Top-pparameter
    pub top_p: Option<f32>,
    /// 频率惩罚
    pub frequency_penalty: Option<f32>,
    /// 存在惩罚
    pub presence_penalty: Option<f32>,
    /// 停止序列
    pub stop: Option<Vec<String>>,
    /// Response
    pub stream: bool,
    /// 工具列表
    pub tools: Option<Vec<Tool>>,
    /// 工具选择
    pub tool_choice: Option<ToolChoice>,
}

/// Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    /// Response
    pub id: String,
    /// Model
    pub model: String,
    /// 选择列表
    pub choices: Vec<ChatChoice>,
    /// usage_stats
    pub usage: Usage,
    /// Create
    pub created: u64,
}

/// 聊天选择
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChoice {
    /// 选择索引
    pub index: u32,
    /// message
    pub message: Message,
    /// 结束原因
    pub finish_reason: Option<String>,
}

/// Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChunk {
    /// Response
    pub id: String,
    /// Model
    pub model: String,
    /// 选择列表
    pub choices: Vec<ChunkChoice>,
}

/// 流式选择
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkChoice {
    /// 选择索引
    pub index: u32,
    /// 增量message
    pub delta: MessageDelta,
    /// 结束原因
    pub finish_reason: Option<String>,
}

/// 增量message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageDelta {
    /// Message role
    pub role: Option<Role>,
    /// Message content
    pub content: Option<String>,
    /// tool_call
    pub tool_calls: Option<Vec<ToolCall>>,
}

/// usage_stats
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Usage {
    /// 提示token数
    pub prompt_tokens: u32,
    /// 完成token数
    pub completion_tokens: u32,
    /// 总token数
    pub total_tokens: u32,
}

/// 成本信息
#[derive(Debug, Clone)]
pub struct Cost {
    /// 成本金额
    pub amount: f64,
    /// 货币类型
    pub currency: String,
    /// 成本分解
    pub breakdown: CostBreakdown,
}

/// 成本分解
#[derive(Debug, Clone)]
pub struct CostBreakdown {
    /// input成本
    pub input_cost: f64,
    /// output成本
    pub output_cost: f64,
    /// 总成本
    pub total_cost: f64,
}
