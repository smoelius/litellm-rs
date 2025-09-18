//! Speech-to-Text (STT) Module for Groq
//!
//! Provides audio transcription capabilities using Groq's Whisper models.

use super::error::GroqError;
use serde::{Deserialize, Serialize};

/// Speech-to-text request
#[derive(Debug, Clone, Serialize)]
pub struct SpeechToTextRequest {
    /// Audio file to transcribe (base64 encoded or raw bytes)
    pub file: Vec<u8>,

    /// Model to use for transcription
    /// Options: "whisper-large-v3", "whisper-large-v3-turbo", "distil-whisper-large-v3-en"
    pub model: String,

    /// Language of the audio (ISO-639-1 format)
    /// Example: "en", "es", "fr", "de", "zh"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    /// Optional text to guide the model's style
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,

    /// Response format: "json", "text", "srt", "verbose_json", "vtt"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<String>,

    /// Temperature for sampling (0.0 to 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Timestamp granularities: "segment", "word"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp_granularities: Option<Vec<String>>,
}

/// Transcription response
#[derive(Debug, Clone, Deserialize)]
pub struct TranscriptionResponse {
    /// Transcribed text
    pub text: String,

    /// Task type (always "transcribe" for Groq)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task: Option<String>,

    /// Detected or specified language
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    /// Duration of the audio in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<f32>,

    /// Word-level timestamps (if requested)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub words: Option<Vec<WordTimestamp>>,

    /// Segment-level timestamps (if requested)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub segments: Option<Vec<SegmentTimestamp>>,
}

/// Word-level timestamp information
#[derive(Debug, Clone, Deserialize)]
pub struct WordTimestamp {
    /// The word
    pub word: String,

    /// Start time in seconds
    pub start: f32,

    /// End time in seconds
    pub end: f32,
}

/// Segment-level timestamp information
#[derive(Debug, Clone, Deserialize)]
pub struct SegmentTimestamp {
    /// Segment ID
    pub id: u32,

    /// Start time in seconds
    pub start: f32,

    /// End time in seconds
    pub end: f32,

    /// Transcribed text for this segment
    pub text: String,

    /// Temperature used for this segment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Average log probability
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_logprob: Option<f32>,

    /// Compression ratio
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compression_ratio: Option<f32>,

    /// Probability of no speech
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_speech_prob: Option<f32>,

    /// Tokens in this segment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens: Option<Vec<u32>>,
}

/// Create multipart form for audio upload
pub fn create_multipart_form(
    request: SpeechToTextRequest,
) -> Result<reqwest::multipart::Form, GroqError> {
    use reqwest::multipart;

    let mut form = multipart::Form::new();

    // Add audio file
    let file_part = multipart::Part::bytes(request.file)
        .file_name("audio.mp3") // Default filename
        .mime_str("audio/mpeg")
        .map_err(|e| GroqError::InvalidRequestError(format!("Invalid MIME type: {}", e)))?;
    form = form.part("file", file_part);

    // Add model
    form = form.text("model", request.model);

    // Add optional parameters
    if let Some(language) = request.language {
        form = form.text("language", language);
    }
    if let Some(prompt) = request.prompt {
        form = form.text("prompt", prompt);
    }
    if let Some(response_format) = request.response_format {
        form = form.text("response_format", response_format);
    }
    if let Some(temperature) = request.temperature {
        form = form.text("temperature", temperature.to_string());
    }
    if let Some(timestamp_granularities) = request.timestamp_granularities {
        for granularity in timestamp_granularities {
            form = form.text("timestamp_granularities[]", granularity);
        }
    }

    Ok(form)
}

/// Supported audio formats
pub fn supported_audio_formats() -> &'static [&'static str] {
    &["mp3", "mp4", "mpeg", "mpga", "m4a", "wav", "webm"]
}

/// Maximum file size in bytes (25MB)
pub const MAX_FILE_SIZE: usize = 25 * 1024 * 1024;
