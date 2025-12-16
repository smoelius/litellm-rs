//! Message types for OpenAI-compatible API
//!
//! This module defines chat messages, roles, content types, and content parts
//! for multimodal interactions.

use serde::{Deserialize, Serialize};

use super::audio::AudioContent;
use super::tools::{FunctionCall, ToolCall};

/// Chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// Message role
    pub role: MessageRole,
    /// Message content
    pub content: Option<MessageContent>,
    /// Message name (for function/tool messages)
    pub name: Option<String>,
    /// Function call (legacy)
    pub function_call: Option<FunctionCall>,
    /// Tool calls
    pub tool_calls: Option<Vec<ToolCall>>,
    /// Tool call ID (for tool messages)
    pub tool_call_id: Option<String>,
    /// Audio content
    pub audio: Option<AudioContent>,
}

/// Message role
#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    /// System message role
    System,
    /// User message role
    User,
    /// Assistant message role
    Assistant,
    /// Function call message role
    Function,
    /// Tool call message role
    Tool,
}

/// Message content (can be string or array of content parts)
#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    /// Plain text content
    Text(String),
    /// Multi-part content (text, images, audio)
    Parts(Vec<ContentPart>),
}

/// Content part for multimodal messages
#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentPart {
    /// Text content part
    #[serde(rename = "text")]
    Text {
        /// Text content
        text: String,
    },
    /// Image URL content part
    #[serde(rename = "image_url")]
    ImageUrl {
        /// Image URL details
        image_url: ImageUrl,
    },
    /// Audio content part
    #[serde(rename = "audio")]
    Audio {
        /// Audio content details
        audio: AudioContent,
    },
}

/// Image URL content
#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub struct ImageUrl {
    /// Image URL
    pub url: String,
    /// Detail level
    pub detail: Option<String>,
}
