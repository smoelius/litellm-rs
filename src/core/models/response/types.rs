//! Core response types for the Gateway

use super::super::Metadata;
use super::super::openai::*;
use super::{
    AudioTranscriptionResponse, CompletionResponse, EmbeddingResponse, ErrorResponse,
    ImageGenerationResponse, ModerationResponse, RerankResponse,
};
use super::{CacheInfo, ProviderInfo, ResponseMetrics};
use serde::{Deserialize, Serialize};

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
            error: super::super::ErrorDetail {
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
