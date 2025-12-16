//! Audio transcription response types

use serde::{Deserialize, Serialize};

/// Audio transcription response
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

    /// Word details (when enabled)
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
