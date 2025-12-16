//! Image response types

use serde::{Deserialize, Serialize};

/// Image response (simple format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageResponse {
    /// Creation timestamp
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

/// Image generation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageGenerationResponse {
    /// Creation timestamp
    pub created: u64,

    /// Generated image list
    pub data: Vec<ImageData>,
}
