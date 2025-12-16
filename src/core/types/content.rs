//! Content part types for multimodal messages

use serde::{Deserialize, Serialize};

/// Content part (multimodal support)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentPart {
    /// Text content
    #[serde(rename = "text")]
    Text { text: String },

    /// Image URL
    #[serde(rename = "image_url")]
    ImageUrl { image_url: ImageUrl },

    /// Audio data
    #[serde(rename = "audio")]
    Audio { audio: AudioData },

    /// Base64 encoded image
    #[serde(rename = "image")]
    Image {
        /// Base64 encoded image data
        source: ImageSource,
        /// Image detail level
        #[serde(skip_serializing_if = "Option::is_none")]
        detail: Option<String>,
        /// Image URL (compatibility field)
        #[serde(skip_serializing_if = "Option::is_none")]
        image_url: Option<ImageUrl>,
    },

    /// Document content (PDF etc)
    #[serde(rename = "document")]
    Document {
        /// Document source data
        source: DocumentSource,
        /// Cache control (Anthropic specific)
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },

    /// Tool result
    #[serde(rename = "tool_result")]
    ToolResult {
        /// Tool usage ID
        tool_use_id: String,
        /// Result content
        content: serde_json::Value,
        /// Error flag
        #[serde(skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
    },

    /// Tool usage
    #[serde(rename = "tool_use")]
    ToolUse {
        /// Tool usage ID
        id: String,
        /// Tool name
        name: String,
        /// Tool input
        input: serde_json::Value,
    },
}

/// Image URL structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageUrl {
    /// Image URL
    pub url: String,
    /// Detail level ("auto", "low", "high")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// Image source data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageSource {
    /// Media type
    pub media_type: String,
    /// Base64 encoded data
    pub data: String,
}

/// Audio data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioData {
    /// Base64 encoded audio data
    pub data: String,
    /// Audio format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
}

/// Input audio data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputAudio {
    /// Base64 encoded audio data
    pub data: String,
    /// Audio format
    pub format: String,
}

/// Document source data (Anthropic PDF support)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSource {
    /// Media type (application/pdf)
    pub media_type: String,
    /// Base64 encoded data
    pub data: String,
}

/// Cache control (Anthropic Cache Control)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheControl {
    /// Cache type ("ephemeral", "persistent")
    #[serde(rename = "type")]
    pub cache_type: String,
}
