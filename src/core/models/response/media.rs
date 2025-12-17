//! Media response types (Image and Audio)

use serde::{Deserialize, Serialize};

/// Image generation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageGenerationResponse {
    /// Creation timestamp
    pub created: u64,
    /// Generated images
    pub data: Vec<ImageData>,
}

/// Image data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageData {
    /// Image URL
    pub url: Option<String>,
    /// Base64 encoded image
    pub b64_json: Option<String>,
    /// Revised prompt
    pub revised_prompt: Option<String>,
}

/// Audio transcription response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioTranscriptionResponse {
    /// Transcribed text
    pub text: String,
    /// Language (if detected)
    pub language: Option<String>,
    /// Duration
    pub duration: Option<f64>,
    /// Segments (if requested)
    pub segments: Option<Vec<TranscriptionSegment>>,
    /// Words (if requested)
    pub words: Option<Vec<TranscriptionWord>>,
}

/// Transcription segment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionSegment {
    /// Segment ID
    pub id: u32,
    /// Seek offset
    pub seek: u32,
    /// Start time
    pub start: f64,
    /// End time
    pub end: f64,
    /// Segment text
    pub text: String,
    /// Tokens
    pub tokens: Vec<u32>,
    /// Temperature
    pub temperature: f64,
    /// Average log probability
    pub avg_logprob: f64,
    /// Compression ratio
    pub compression_ratio: f64,
    /// No speech probability
    pub no_speech_prob: f64,
}

/// Transcription word
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionWord {
    /// Word text
    pub word: String,
    /// Start time
    pub start: f64,
    /// End time
    pub end: f64,
}
