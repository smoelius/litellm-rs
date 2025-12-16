//! Chat response types

use serde::{Deserialize, Serialize};

use super::super::requests::{ChatMessage, MessageContent, ToolCall};
use super::delta::ChatDelta;
use super::logprobs::{FinishReason, LogProbs};
use super::usage::Usage;

/// Chat completion response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    /// Response ID
    pub id: String,

    /// Object type
    pub object: String,

    /// Creation timestamp
    pub created: i64,

    /// Model used
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

/// Streaming chat chunk response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChunk {
    /// Response ID
    pub id: String,

    /// Object type
    pub object: String,

    /// Creation timestamp
    pub created: i64,

    /// Model used
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

impl ChatResponse {
    /// Get first message content
    pub fn first_content(&self) -> Option<&str> {
        self.choices
            .first()
            .and_then(|choice| match &choice.message.content {
                Some(MessageContent::Text(text)) => Some(text.as_str()),
                _ => None,
            })
    }

    /// Get all message contents
    pub fn all_content(&self) -> Vec<&str> {
        self.choices
            .iter()
            .filter_map(|choice| match &choice.message.content {
                Some(MessageContent::Text(text)) => Some(text.as_str()),
                _ => None,
            })
            .collect()
    }

    /// Check if response has tool calls
    pub fn has_tool_calls(&self) -> bool {
        self.choices
            .iter()
            .any(|choice| choice.message.tool_calls.is_some())
    }

    /// Get first tool calls
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
