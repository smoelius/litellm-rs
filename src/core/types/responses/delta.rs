//! Streaming delta types

use serde::{Deserialize, Serialize};

use super::super::requests::MessageRole;
use super::super::thinking::ThinkingDelta;

/// Streaming delta content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatDelta {
    /// Role (usually only appears in first chunk)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<MessageRole>,

    /// Content delta
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,

    /// Thinking/reasoning delta (for thinking-enabled models)
    ///
    /// When streaming from thinking models, thinking content may arrive
    /// before or alongside the main content. Use this field to track
    /// the model's reasoning process in real-time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<ThinkingDelta>,

    /// Tool call delta
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCallDelta>>,

    /// Function call delta (backward compatibility)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_call: Option<FunctionCallDelta>,
}

impl ChatDelta {
    /// Check if this delta contains thinking content
    pub fn has_thinking(&self) -> bool {
        self.thinking.is_some()
    }

    /// Get thinking content if present
    pub fn thinking_content(&self) -> Option<&str> {
        self.thinking.as_ref().and_then(|t| t.content.as_deref())
    }
}

/// Tool call delta
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallDelta {
    /// Index
    pub index: u32,

    /// Call ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Tool type
    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    pub tool_type: Option<String>,

    /// Function call delta
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function: Option<FunctionCallDelta>,
}

/// Function call delta
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCallDelta {
    /// Function name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Parameter delta
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<String>,
}
