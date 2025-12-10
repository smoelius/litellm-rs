//! Audio API module for speech-to-text and text-to-speech
//!
//! Provides unified audio processing capabilities across providers.

use crate::core::providers::{Provider, ProviderRegistry};
use crate::utils::error::{GatewayError, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info};

/// Audio transcription request (OpenAI compatible)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionRequest {
    /// Audio file bytes
    #[serde(skip)]
    pub file: Vec<u8>,

    /// Original filename
    #[serde(skip)]
    pub filename: String,

    /// Model to use (e.g., "whisper-1", "whisper-large-v3")
    pub model: String,

    /// Language of the audio (ISO-639-1 format)
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

/// Audio transcription response (OpenAI compatible)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionResponse {
    /// Transcribed text
    pub text: String,

    /// Task type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task: Option<String>,

    /// Detected or specified language
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    /// Duration of the audio in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<f64>,

    /// Word-level timestamps
    #[serde(skip_serializing_if = "Option::is_none")]
    pub words: Option<Vec<WordInfo>>,

    /// Segment-level timestamps
    #[serde(skip_serializing_if = "Option::is_none")]
    pub segments: Option<Vec<SegmentInfo>>,
}

/// Word-level timestamp information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordInfo {
    /// The word
    pub word: String,
    /// Start time in seconds
    pub start: f64,
    /// End time in seconds
    pub end: f64,
}

/// Segment-level timestamp information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentInfo {
    /// Segment ID
    pub id: u32,
    /// Start time in seconds
    pub start: f64,
    /// End time in seconds
    pub end: f64,
    /// Transcribed text for this segment
    pub text: String,
}

/// Audio translation request (translate to English)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationRequest {
    /// Audio file bytes
    #[serde(skip)]
    pub file: Vec<u8>,

    /// Original filename
    #[serde(skip)]
    pub filename: String,

    /// Model to use
    pub model: String,

    /// Optional text to guide the model's style
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,

    /// Response format: "json", "text", "srt", "verbose_json", "vtt"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<String>,

    /// Temperature for sampling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
}

/// Audio translation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationResponse {
    /// Translated text (always in English)
    pub text: String,

    /// Task type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task: Option<String>,

    /// Source language
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    /// Duration of the audio in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<f64>,

    /// Segments
    #[serde(skip_serializing_if = "Option::is_none")]
    pub segments: Option<Vec<SegmentInfo>>,
}

/// Text-to-speech request (OpenAI compatible)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeechRequest {
    /// Text to convert to speech
    pub input: String,

    /// Model to use (e.g., "tts-1", "tts-1-hd")
    pub model: String,

    /// Voice to use (e.g., "alloy", "echo", "fable", "onyx", "nova", "shimmer")
    pub voice: String,

    /// Audio format: "mp3", "opus", "aac", "flac", "wav", "pcm"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<String>,

    /// Speed of speech (0.25 to 4.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<f32>,
}

/// Text-to-speech response
pub struct SpeechResponse {
    /// Audio data bytes
    pub audio: Vec<u8>,

    /// Content type (e.g., "audio/mpeg", "audio/opus")
    pub content_type: String,
}

/// Audio service for handling audio API requests
pub struct AudioService {
    provider_registry: Arc<ProviderRegistry>,
}

impl AudioService {
    /// Create a new audio service
    pub fn new(provider_registry: Arc<ProviderRegistry>) -> Self {
        Self { provider_registry }
    }

    /// Transcribe audio to text
    pub async fn transcribe(&self, request: TranscriptionRequest) -> Result<TranscriptionResponse> {
        info!(
            "Transcribing audio: model={}, file_size={}",
            request.model,
            request.file.len()
        );

        // Validate file size (max 25MB)
        if request.file.len() > 25 * 1024 * 1024 {
            return Err(GatewayError::validation("Audio file too large (max 25MB)"));
        }

        // Determine provider from model name
        let (provider_name, actual_model) = parse_model_string(&request.model);

        // Find provider
        let providers = self.provider_registry.all();
        let provider = providers
            .iter()
            .find(|p| p.name() == provider_name)
            .ok_or_else(|| {
                GatewayError::internal(format!(
                    "No provider found for audio transcription: {}",
                    provider_name
                ))
            })?;

        // Route to appropriate provider
        match provider {
            Provider::Groq(groq) => {
                debug!("Using Groq for transcription");
                let response = groq
                    .transcribe_audio(
                        request.file,
                        Some(actual_model.to_string()),
                        request.language,
                        request.response_format,
                    )
                    .await
                    .map_err(|e| {
                        GatewayError::internal(format!("Groq transcription error: {}", e))
                    })?;

                Ok(TranscriptionResponse {
                    text: response.text,
                    task: response.task,
                    language: response.language,
                    duration: response.duration.map(|d| d as f64),
                    words: response.words.map(|words| {
                        words
                            .into_iter()
                            .map(|w| WordInfo {
                                word: w.word,
                                start: w.start as f64,
                                end: w.end as f64,
                            })
                            .collect()
                    }),
                    segments: response.segments.map(|segs| {
                        segs.into_iter()
                            .map(|s| SegmentInfo {
                                id: s.id,
                                start: s.start as f64,
                                end: s.end as f64,
                                text: s.text,
                            })
                            .collect()
                    }),
                })
            }
            Provider::OpenAI(_openai) => {
                // OpenAI transcription - similar implementation
                Err(GatewayError::internal(
                    "OpenAI audio transcription not yet implemented",
                ))
            }
            _ => Err(GatewayError::internal(format!(
                "Provider {} does not support audio transcription",
                provider.name()
            ))),
        }
    }

    /// Translate audio to English text
    pub async fn translate(&self, request: TranslationRequest) -> Result<TranslationResponse> {
        info!(
            "Translating audio: model={}, file_size={}",
            request.model,
            request.file.len()
        );

        // Validate file size (max 25MB)
        if request.file.len() > 25 * 1024 * 1024 {
            return Err(GatewayError::validation("Audio file too large (max 25MB)"));
        }

        // For translation, we use transcription with target language = English
        // Most providers use the same endpoint with different parameters
        let (provider_name, actual_model) = parse_model_string(&request.model);

        let providers = self.provider_registry.all();
        let provider = providers
            .iter()
            .find(|p| p.name() == provider_name)
            .ok_or_else(|| {
                GatewayError::internal(format!(
                    "No provider found for audio translation: {}",
                    provider_name
                ))
            })?;

        match provider {
            Provider::Groq(groq) => {
                // Groq uses the same endpoint, we set language to "en" for translation
                let response = groq
                    .transcribe_audio(
                        request.file,
                        Some(actual_model.to_string()),
                        Some("en".to_string()), // Force English output
                        request.response_format,
                    )
                    .await
                    .map_err(|e| {
                        GatewayError::internal(format!("Groq translation error: {}", e))
                    })?;

                Ok(TranslationResponse {
                    text: response.text,
                    task: Some("translate".to_string()),
                    language: response.language,
                    duration: response.duration.map(|d| d as f64),
                    segments: response.segments.map(|segs| {
                        segs.into_iter()
                            .map(|s| SegmentInfo {
                                id: s.id,
                                start: s.start as f64,
                                end: s.end as f64,
                                text: s.text,
                            })
                            .collect()
                    }),
                })
            }
            _ => Err(GatewayError::internal(format!(
                "Provider {} does not support audio translation",
                provider.name()
            ))),
        }
    }

    /// Convert text to speech
    pub async fn speech(&self, request: SpeechRequest) -> Result<SpeechResponse> {
        info!(
            "Generating speech: model={}, voice={}, text_len={}",
            request.model,
            request.voice,
            request.input.len()
        );

        // Validate input length (max 4096 characters for most providers)
        if request.input.len() > 4096 {
            return Err(GatewayError::validation(
                "Input text too long (max 4096 characters)",
            ));
        }

        let (provider_name, _actual_model) = parse_model_string(&request.model);

        let providers = self.provider_registry.all();
        let provider = providers
            .iter()
            .find(|p| p.name() == provider_name)
            .ok_or_else(|| {
                GatewayError::internal(format!(
                    "No provider found for text-to-speech: {}",
                    provider_name
                ))
            })?;

        match provider {
            Provider::OpenAI(_openai) => {
                // OpenAI TTS implementation would go here
                Err(GatewayError::internal(
                    "OpenAI text-to-speech not yet implemented",
                ))
            }
            _ => Err(GatewayError::internal(format!(
                "Provider {} does not support text-to-speech",
                provider.name()
            ))),
        }
    }
}

/// Parse model string to extract provider and model name
/// Format: "provider/model" or just "model"
fn parse_model_string(model: &str) -> (&str, &str) {
    if let Some(idx) = model.find('/') {
        let provider = &model[..idx];
        let model_name = &model[idx + 1..];
        (provider, model_name)
    } else {
        // Default provider based on model name
        if model.starts_with("whisper") {
            ("groq", model) // Default Whisper to Groq (faster)
        } else {
            // Default to OpenAI for TTS and other models
            ("openai", model)
        }
    }
}

/// Supported audio formats
pub fn supported_audio_formats() -> &'static [&'static str] {
    &[
        "flac", "m4a", "mp3", "mp4", "mpeg", "mpga", "oga", "ogg", "wav", "webm",
    ]
}

/// Get content type from format
pub fn format_to_content_type(format: &str) -> &'static str {
    match format.to_lowercase().as_str() {
        "mp3" => "audio/mpeg",
        "opus" => "audio/opus",
        "aac" => "audio/aac",
        "flac" => "audio/flac",
        "wav" => "audio/wav",
        "pcm" => "audio/pcm",
        _ => "audio/mpeg",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_model_string() {
        assert_eq!(
            parse_model_string("groq/whisper-large-v3"),
            ("groq", "whisper-large-v3")
        );
        assert_eq!(
            parse_model_string("openai/whisper-1"),
            ("openai", "whisper-1")
        );
        assert_eq!(
            parse_model_string("whisper-large-v3"),
            ("groq", "whisper-large-v3")
        );
        assert_eq!(parse_model_string("tts-1"), ("openai", "tts-1"));
    }

    #[test]
    fn test_format_to_content_type() {
        assert_eq!(format_to_content_type("mp3"), "audio/mpeg");
        assert_eq!(format_to_content_type("opus"), "audio/opus");
        assert_eq!(format_to_content_type("wav"), "audio/wav");
    }

    #[test]
    fn test_supported_formats() {
        let formats = supported_audio_formats();
        assert!(formats.contains(&"mp3"));
        assert!(formats.contains(&"wav"));
        assert!(formats.contains(&"webm"));
    }
}
