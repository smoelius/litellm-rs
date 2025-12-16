//! Image generation request types

use serde::{Deserialize, Serialize};

/// Image request (short form)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageRequest {
    /// Model name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Image description prompt
    pub prompt: String,
    /// Number of images to generate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,
    /// Image quality
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<String>,
    /// Response format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<String>,
    /// Image size
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    /// Image style
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
    /// User ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

/// Image generation request (full form)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageGenerationRequest {
    /// Image description prompt
    pub prompt: String,
    /// Model name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Number of images to generate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,
    /// Image size
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    /// Image quality
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<String>,
    /// Response format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<String>,
    /// Style
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
    /// User ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

/// Audio transcription request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioTranscriptionRequest {
    /// Audio file data
    pub file: Vec<u8>,
    /// Model name
    pub model: String,
    /// Language (ISO-639-1 format)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    /// Prompt
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    /// Response format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<String>,
    /// Temperature
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
}

/// Completion request (legacy text completion)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    /// Model name
    pub model: String,
    /// Input text prompt
    pub prompt: String,
    /// Sampling temperature
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// Maximum number of tokens to generate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    /// Nucleus sampling parameter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    /// Frequency penalty
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,
    /// Presence penalty
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,
    /// Stop sequences
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    /// Number of choices to return
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,
    /// Enable streaming
    #[serde(default)]
    pub stream: bool,
    /// User ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}
