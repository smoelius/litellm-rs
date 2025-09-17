//! Response models for the Gateway
//!
//! This module defines internal response structures used by the gateway.

use super::Metadata;
use super::openai::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Internal gateway response wrapper
#[derive(Debug, Clone)]
pub struct GatewayResponse {
    /// Response metadata
    pub metadata: Metadata,
    /// Response type
    pub response_type: ResponseType,
    /// Response data
    pub data: ResponseData,
    /// Provider information
    pub provider_info: ProviderInfo,
    /// Performance metrics
    pub metrics: ResponseMetrics,
    /// Caching information
    pub cache_info: CacheInfo,
}

/// Response type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseType {
    /// Chat completion response
    ChatCompletion,
    /// Text completion response
    Completion,
    /// Embedding response
    Embedding,
    /// Image generation response
    ImageGeneration,
    /// Audio transcription response
    AudioTranscription,
    /// Moderation response
    Moderation,
    /// Rerank response
    Rerank,
    /// Error response
    Error,
}

/// Response data union
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ResponseData {
    /// Chat completion response data
    #[serde(rename = "chat_completion")]
    ChatCompletion(ChatCompletionResponse),
    /// Text completion response data
    #[serde(rename = "completion")]
    Completion(CompletionResponse),
    /// Embedding response data
    #[serde(rename = "embedding")]
    Embedding(EmbeddingResponse),
    /// Image generation response data
    #[serde(rename = "image_generation")]
    ImageGeneration(ImageGenerationResponse),
    /// Audio transcription response data
    #[serde(rename = "audio_transcription")]
    AudioTranscription(AudioTranscriptionResponse),
    /// Moderation response data
    #[serde(rename = "moderation")]
    Moderation(ModerationResponse),
    /// Rerank response data
    #[serde(rename = "rerank")]
    Rerank(RerankResponse),
    /// Error response data
    #[serde(rename = "error")]
    Error(ErrorResponse),
}

/// Completion response (legacy)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    /// Response ID
    pub id: String,
    /// Object type
    pub object: String,
    /// Creation timestamp
    pub created: u64,
    /// Model used
    pub model: String,
    /// Choices
    pub choices: Vec<CompletionChoice>,
    /// Usage statistics
    pub usage: Option<Usage>,
}

/// Completion choice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionChoice {
    /// Choice index
    pub index: u32,
    /// Generated text
    pub text: String,
    /// Logprobs
    pub logprobs: Option<CompletionLogprobs>,
    /// Finish reason
    pub finish_reason: Option<String>,
}

/// Completion logprobs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionLogprobs {
    /// Tokens
    pub tokens: Vec<String>,
    /// Token logprobs
    pub token_logprobs: Vec<f64>,
    /// Top logprobs
    pub top_logprobs: Vec<HashMap<String, f64>>,
    /// Text offset
    pub text_offset: Vec<u32>,
}

/// Embedding response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    /// Object type
    pub object: String,
    /// Embeddings
    pub data: Vec<EmbeddingData>,
    /// Model used
    pub model: String,
    /// Usage statistics
    pub usage: Option<EmbeddingUsage>,
}

/// Embedding data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingData {
    /// Object type
    pub object: String,
    /// Index
    pub index: u32,
    /// Embedding vector
    pub embedding: Vec<f32>,
}

/// Embedding usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingUsage {
    /// Prompt tokens
    pub prompt_tokens: u32,
    /// Total tokens
    pub total_tokens: u32,
}

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

/// Moderation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModerationResponse {
    /// Response ID
    pub id: String,
    /// Model used
    pub model: String,
    /// Results
    pub results: Vec<ModerationResult>,
}

/// Moderation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModerationResult {
    /// Whether content is flagged
    pub flagged: bool,
    /// Category flags
    pub categories: ModerationCategories,
    /// Category scores
    pub category_scores: ModerationCategoryScores,
}

/// Moderation categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModerationCategories {
    /// Sexual content
    pub sexual: bool,
    /// Hate speech
    pub hate: bool,
    /// Harassment
    pub harassment: bool,
    /// Self-harm
    #[serde(rename = "self-harm")]
    pub self_harm: bool,
    /// Sexual content involving minors
    #[serde(rename = "sexual/minors")]
    pub sexual_minors: bool,
    /// Hate speech targeting identity
    #[serde(rename = "hate/threatening")]
    pub hate_threatening: bool,
    /// Harassment threatening
    #[serde(rename = "harassment/threatening")]
    pub harassment_threatening: bool,
    /// Self-harm instructions
    #[serde(rename = "self-harm/instructions")]
    pub self_harm_instructions: bool,
    /// Self-harm intent
    #[serde(rename = "self-harm/intent")]
    pub self_harm_intent: bool,
    /// Violence
    pub violence: bool,
    /// Graphic violence
    #[serde(rename = "violence/graphic")]
    pub violence_graphic: bool,
}

/// Moderation category scores
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModerationCategoryScores {
    /// Sexual content score
    pub sexual: f64,
    /// Hate speech score
    pub hate: f64,
    /// Harassment score
    pub harassment: f64,
    /// Self-harm score
    #[serde(rename = "self-harm")]
    pub self_harm: f64,
    /// Sexual content involving minors score
    #[serde(rename = "sexual/minors")]
    pub sexual_minors: f64,
    /// Hate speech targeting identity score
    #[serde(rename = "hate/threatening")]
    pub hate_threatening: f64,
    /// Harassment threatening score
    #[serde(rename = "harassment/threatening")]
    pub harassment_threatening: f64,
    /// Self-harm instructions score
    #[serde(rename = "self-harm/instructions")]
    pub self_harm_instructions: f64,
    /// Self-harm intent score
    #[serde(rename = "self-harm/intent")]
    pub self_harm_intent: f64,
    /// Violence score
    pub violence: f64,
    /// Graphic violence score
    #[serde(rename = "violence/graphic")]
    pub violence_graphic: f64,
}

/// Rerank response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankResponse {
    /// Response ID
    pub id: String,
    /// Model used
    pub model: String,
    /// Reranked results
    pub results: Vec<RerankResult>,
    /// Usage statistics
    pub usage: Option<RerankUsage>,
}

/// Rerank result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankResult {
    /// Document index
    pub index: u32,
    /// Relevance score
    pub relevance_score: f64,
    /// Document text (if requested)
    pub document: Option<String>,
}

/// Rerank usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankUsage {
    /// Total tokens
    pub total_tokens: u32,
}

/// Error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Error details
    pub error: ErrorDetail,
}

/// Error detail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetail {
    /// Error message
    pub message: String,
    /// Error type
    #[serde(rename = "type")]
    pub error_type: String,
    /// Error code
    pub code: Option<String>,
    /// Parameter that caused the error
    pub param: Option<String>,
}

/// Provider information
#[derive(Debug, Clone, Default)]
pub struct ProviderInfo {
    /// Provider name
    pub name: String,
    /// Provider type
    pub provider_type: String,
    /// Model used
    pub model: String,
    /// API version
    pub api_version: Option<String>,
    /// Region
    pub region: Option<String>,
    /// Deployment ID
    pub deployment_id: Option<String>,
}

/// Response performance metrics
#[derive(Debug, Clone, Default)]
pub struct ResponseMetrics {
    /// Total response time in milliseconds
    pub total_time_ms: u64,
    /// Provider response time in milliseconds
    pub provider_time_ms: u64,
    /// Queue time in milliseconds
    pub queue_time_ms: u64,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
    /// Number of retries
    pub retry_count: u32,
    /// Whether response was cached
    pub from_cache: bool,
    /// Cache hit type
    pub cache_type: Option<String>,
}

/// Cache information
#[derive(Debug, Clone, Default)]
pub struct CacheInfo {
    /// Whether response was cached
    pub cached: bool,
    /// Cache key
    pub cache_key: Option<String>,
    /// Cache TTL
    pub ttl_seconds: Option<u64>,
    /// Cache hit/miss
    pub hit: bool,
    /// Cache type (memory, redis, semantic)
    pub cache_type: Option<String>,
    /// Semantic similarity score (for semantic cache)
    pub similarity_score: Option<f32>,
}

impl GatewayResponse {
    /// Create a new gateway response
    pub fn new(response_type: ResponseType, data: ResponseData) -> Self {
        Self {
            metadata: Metadata::new(),
            response_type,
            data,
            provider_info: ProviderInfo::default(),
            metrics: ResponseMetrics::default(),
            cache_info: CacheInfo::default(),
        }
    }

    /// Set provider information
    pub fn with_provider_info(mut self, provider_info: ProviderInfo) -> Self {
        self.provider_info = provider_info;
        self
    }

    /// Set metrics
    pub fn with_metrics(mut self, metrics: ResponseMetrics) -> Self {
        self.metrics = metrics;
        self
    }

    /// Set cache information
    pub fn with_cache_info(mut self, cache_info: CacheInfo) -> Self {
        self.cache_info = cache_info;
        self
    }

    /// Check if response is an error
    pub fn is_error(&self) -> bool {
        matches!(self.response_type, ResponseType::Error)
    }

    /// Get usage information if available
    pub fn usage(&self) -> Option<&Usage> {
        match &self.data {
            ResponseData::ChatCompletion(resp) => resp.usage.as_ref(),
            ResponseData::Completion(resp) => resp.usage.as_ref(),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gateway_response_creation() {
        let chat_response = ChatCompletionResponse {
            id: "test-id".to_string(),
            object: "chat.completion".to_string(),
            created: 1234567890,
            model: "gpt-4".to_string(),
            system_fingerprint: None,
            choices: vec![],
            usage: None,
        };

        let data = ResponseData::ChatCompletion(chat_response);
        let response = GatewayResponse::new(ResponseType::ChatCompletion, data);

        assert!(matches!(
            response.response_type,
            ResponseType::ChatCompletion
        ));
        assert!(!response.is_error());
    }

    #[test]
    fn test_error_response() {
        let error_response = ErrorResponse {
            error: ErrorDetail {
                message: "Test error".to_string(),
                error_type: "invalid_request".to_string(),
                code: Some("400".to_string()),
                param: None,
            },
        };

        let data = ResponseData::Error(error_response);
        let response = GatewayResponse::new(ResponseType::Error, data);

        assert!(response.is_error());
    }
}
