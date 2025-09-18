//! Response types
//!
//! Defines unified data structures for all API responses

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

    /// Choice list
    pub choices: Vec<ChatChoice>,

    /// Usage statistics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,

    /// System fingerprint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: Option<String>,
}

/// Chat choice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChoice {
    /// Choice index
    pub index: u32,

    /// Response message
    pub message: ChatMessage,

    /// Completion reason
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

    /// Choice list
    pub choices: Vec<ChatStreamChoice>,

    /// Usage (usually in last chunk)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,

    /// System fingerprint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: Option<String>,
}

/// Streaming choice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatStreamChoice {
    /// Choice index
    pub index: u32,

    /// Delta content
    pub delta: ChatDelta,

    /// Finish reason
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<FinishReason>,

    /// Log probabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<LogProbs>,
}

/// Streaming delta content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatDelta {
    /// Role (usually only appears in first chunk)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<MessageRole>,

    /// Content delta
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,

    /// Tool call delta
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCallDelta>>,

    /// Function call delta (backward compatibility)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_call: Option<FunctionCallDelta>,
}

/// Tool call delta
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallDelta {
    /// Index
    pub index: u32,

    /// callID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// tool_type
    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    pub tool_type: Option<String>,

    /// Function call delta
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function: Option<FunctionCallDelta>,
}

/// Function call delta
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCallDelta {
    /// function_name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Parameter delta
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<String>,
}

/// Finish reason
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    /// Natural stop
    Stop,
    /// Length limit reached
    Length,
    /// tool_call
    ToolCalls,
    /// Content filter
    ContentFilter,
    /// Function call (backward compatibility)
    FunctionCall,
}

/// usage_stats
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Usage {
    /// Prompt token count
    pub prompt_tokens: u32,

    /// Completion token count
    pub completion_tokens: u32,

    /// Total token count
    pub total_tokens: u32,

    /// Prompt token details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_tokens_details: Option<PromptTokensDetails>,

    /// Completion token details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_tokens_details: Option<CompletionTokensDetails>,
}

/// Prompt token details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTokensDetails {
    /// Cached token count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_tokens: Option<u32>,

    /// Audio token count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_tokens: Option<u32>,
}

/// Completion token details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionTokensDetails {
    /// Reasoning token count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_tokens: Option<u32>,

    /// Audio token count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_tokens: Option<u32>,
}

/// Log probabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogProbs {
    /// Token log probabilities
    pub content: Vec<TokenLogProb>,

    /// Refusal sampling information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refusal: Option<String>,
}

/// Single token log probability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenLogProb {
    /// Token text
    pub token: String,

    /// Log probability
    pub logprob: f64,

    /// Token byte representation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes: Option<Vec<u8>>,

    /// Top log probabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<Vec<TopLogProb>>,
}

/// Top log probability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopLogProb {
    /// Token text
    pub token: String,

    /// Log probability
    pub logprob: f64,

    /// Token byte representation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes: Option<Vec<u8>>,
}

/// Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedResponse {
    /// object_type
    pub object: String,

    /// Embedding data list
    pub data: Vec<EmbeddingData>,

    /// Model
    pub model: String,

    /// usage_stats
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<EmbeddingUsage>,
}

/// Embedding data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingData {
    /// object_type
    pub object: String,

    /// Index
    pub index: u32,

    /// Embedding vector
    pub embedding: Vec<f32>,
}

/// Embedding usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingUsage {
    /// Prompt token count
    pub prompt_tokens: u32,

    /// Total token count
    pub total_tokens: u32,
}

/// Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageResponse {
    /// Create
    pub created: i64,

    /// Image data list
    pub data: Vec<ImageData>,
}

/// Image data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageData {
    /// Image URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// Base64 encoded image
    #[serde(skip_serializing_if = "Option::is_none")]
    pub b64_json: Option<String>,

    /// Revised prompt (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revised_prompt: Option<String>,
}

/// Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioTranscriptionResponse {
    /// Transcription text
    pub text: String,

    /// Language
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    /// Duration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<f64>,

    /// Details (when enabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub words: Option<Vec<WordInfo>>,

    /// Segment information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub segments: Option<Vec<SegmentInfo>>,
}

/// Word information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordInfo {
    /// Word text
    pub word: String,

    /// Start time
    pub start: f64,

    /// End time
    pub end: f64,
}

/// Segment information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentInfo {
    /// Segment ID
    pub id: u32,

    /// Start time
    pub start: f64,

    /// End time
    pub end: f64,

    /// Text content
    pub text: String,

    /// Temperature
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Average log probability
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_logprob: Option<f64>,

    /// Compression ratio
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compression_ratio: Option<f64>,

    /// No speech probability
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

    /// Calculate total cost (requires pricing information)
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
    /// Choice list
    pub choices: Vec<CompletionChoice>,
    /// usage_stats
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
    /// System fingerprint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: Option<String>,
}

/// Completion choice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionChoice {
    /// Choice index
    pub index: u32,
    /// Generated text
    pub text: String,
    /// Finish reason
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<FinishReason>,
    /// Log probability information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<LogProbs>,
}

/// Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    /// object_type
    pub object: String,
    /// Embedding data list
    pub data: Vec<EmbeddingData>,
    /// Model
    pub model: String,
    /// usage_stats
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
    /// Embedding data list (backward compatibility field)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embeddings: Option<Vec<EmbeddingData>>,
}

// EmbeddingData already defined earlier in this file

/// Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageGenerationResponse {
    /// Create
    pub created: u64,
    /// Generated image list
    pub data: Vec<ImageData>,
}

// ImageData already defined earlier in this file
