//! Type definitions for streaming responses

use crate::core::models::openai::Usage;
use crate::core::types::MessageRole;
use actix_web::web;

/// Simple Event structure for SSE compatibility
#[derive(Debug, Clone, Default)]
pub struct Event {
    /// Event type
    pub event: Option<String>,
    /// Event data
    pub data: String,
}

impl Event {
    /// Create a new empty event
    pub fn new() -> Self {
        Self {
            event: None,
            data: String::new(),
        }
    }

    /// Set the event type
    pub fn event(mut self, event: &str) -> Self {
        self.event = Some(event.to_string());
        self
    }

    /// Set the event data
    pub fn data(mut self, data: &str) -> Self {
        self.data = data.to_string();
        self
    }

    /// Convert event to bytes for SSE transmission
    pub fn to_bytes(&self) -> web::Bytes {
        let mut result = String::new();
        if let Some(event) = &self.event {
            result.push_str(&format!("event: {}\n", event));
        }
        result.push_str(&format!("data: {}\n\n", self.data));
        web::Bytes::from(result)
    }
}

/// Streaming response chunk for chat completions
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionChunk {
    /// Unique identifier for the completion
    pub id: String,
    /// Object type (always "chat.completion.chunk")
    pub object: String,
    /// Unix timestamp of creation
    pub created: u64,
    /// Model used for completion
    pub model: String,
    /// System fingerprint
    pub system_fingerprint: Option<String>,
    /// Array of completion choices
    pub choices: Vec<ChatCompletionChunkChoice>,
    /// Usage statistics (only in final chunk)
    pub usage: Option<Usage>,
}

/// Choice in a streaming chat completion chunk
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionChunkChoice {
    /// Index of the choice
    pub index: u32,
    /// Delta containing the incremental content
    pub delta: ChatCompletionDelta,
    /// Reason for finishing (only in final chunk)
    pub finish_reason: Option<String>,
    /// Log probabilities
    pub logprobs: Option<serde_json::Value>,
}

/// Delta containing incremental content in streaming response
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatCompletionDelta {
    /// Role of the message (only in first chunk)
    pub role: Option<MessageRole>,
    /// Incremental content
    pub content: Option<String>,
    /// Tool calls (for function calling)
    pub tool_calls: Option<Vec<ToolCallDelta>>,
}

/// Tool call delta for streaming function calls
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolCallDelta {
    /// Index of the tool call
    pub index: u32,
    /// Tool call ID (only in first chunk)
    pub id: Option<String>,
    /// Type of tool call (only in first chunk)
    #[serde(rename = "type")]
    pub tool_type: Option<String>,
    /// Function call details
    pub function: Option<FunctionCallDelta>,
}

/// Function call delta for streaming
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FunctionCallDelta {
    /// Function name (only in first chunk)
    pub name: Option<String>,
    /// Incremental function arguments
    pub arguments: Option<String>,
}
