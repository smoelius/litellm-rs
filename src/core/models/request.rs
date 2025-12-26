//! Request models for the Gateway
//!
//! This module defines internal request structures used by the gateway.

use super::openai::*;
use super::{Metadata, RequestContext};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Internal gateway request wrapper
#[derive(Debug, Clone)]
pub struct GatewayRequest {
    /// Request metadata
    pub metadata: Metadata,
    /// Request context
    pub context: RequestContext,
    /// Request type
    pub request_type: RequestType,
    /// Original request data
    pub data: RequestData,
    /// Provider-specific parameters
    pub provider_params: HashMap<String, serde_json::Value>,
    /// Routing preferences
    pub routing: RoutingPreferences,
    /// Caching preferences
    pub caching: CachingPreferences,
}

/// Request type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestType {
    /// Chat completion requests
    ChatCompletion,
    /// Text completion requests
    Completion,
    /// Text embedding requests
    Embedding,
    /// Image generation requests
    ImageGeneration,
    /// Image editing requests
    ImageEdit,
    /// Image variation requests
    ImageVariation,
    /// Audio transcription requests
    AudioTranscription,
    /// Audio translation requests
    AudioTranslation,
    /// Text-to-speech requests
    AudioSpeech,
    /// Content moderation requests
    Moderation,
    /// Fine-tuning requests
    FineTuning,
    /// File management requests
    Files,
    /// Assistant API requests
    Assistants,
    /// Thread management requests
    Threads,
    /// Batch processing requests
    Batches,
    /// Vector store requests
    VectorStores,
    /// Document reranking requests
    Rerank,
    /// Real-time API requests
    Realtime,
}

/// Request data union
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RequestData {
    /// Chat completion request data
    #[serde(rename = "chat_completion")]
    ChatCompletion(Box<ChatCompletionRequest>),
    /// Text completion request data
    #[serde(rename = "completion")]
    Completion(CompletionRequest),
    /// Embedding request data
    #[serde(rename = "embedding")]
    Embedding(EmbeddingRequest),
    /// Image generation request data
    #[serde(rename = "image_generation")]
    ImageGeneration(ImageGenerationRequest),
    /// Audio transcription request data
    #[serde(rename = "audio_transcription")]
    AudioTranscription(AudioTranscriptionRequest),
    /// Moderation request data
    #[serde(rename = "moderation")]
    Moderation(ModerationRequest),
    /// Rerank request data
    #[serde(rename = "rerank")]
    Rerank(RerankRequest),
}

/// Completion request (legacy)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    /// Model to use
    pub model: String,
    /// Prompt text
    pub prompt: Option<String>,
    /// Maximum tokens
    pub max_tokens: Option<u32>,
    /// Temperature
    pub temperature: Option<f32>,
    /// Top-p sampling
    pub top_p: Option<f32>,
    /// Number of completions
    pub n: Option<u32>,
    /// Stream response
    pub stream: Option<bool>,
    /// Stop sequences
    pub stop: Option<Vec<String>>,
    /// Presence penalty
    pub presence_penalty: Option<f32>,
    /// Frequency penalty
    pub frequency_penalty: Option<f32>,
    /// Logit bias
    pub logit_bias: Option<HashMap<String, f32>>,
    /// User identifier
    pub user: Option<String>,
    /// Suffix
    pub suffix: Option<String>,
    /// Echo prompt
    pub echo: Option<bool>,
    /// Best of
    pub best_of: Option<u32>,
    /// Logprobs
    pub logprobs: Option<u32>,
}

/// Embedding request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingRequest {
    /// Model to use
    pub model: String,
    /// Input text(s)
    pub input: EmbeddingInput,
    /// Encoding format
    pub encoding_format: Option<String>,
    /// Dimensions
    pub dimensions: Option<u32>,
    /// User identifier
    pub user: Option<String>,
}

/// Embedding input
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EmbeddingInput {
    /// Single text string
    String(String),
    /// Array of text strings
    Array(Vec<String>),
    /// Array of token IDs
    Tokens(Vec<u32>),
    /// Array of token ID arrays
    TokenArrays(Vec<Vec<u32>>),
}

/// Image generation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageGenerationRequest {
    /// Model to use
    pub model: Option<String>,
    /// Prompt
    pub prompt: String,
    /// Number of images
    pub n: Option<u32>,
    /// Image size
    pub size: Option<String>,
    /// Response format
    pub response_format: Option<String>,
    /// Quality
    pub quality: Option<String>,
    /// Style
    pub style: Option<String>,
    /// User identifier
    pub user: Option<String>,
}

/// Audio transcription request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioTranscriptionRequest {
    /// Model to use
    pub model: String,
    /// Audio file (base64 encoded)
    pub file: String,
    /// Language
    pub language: Option<String>,
    /// Prompt
    pub prompt: Option<String>,
    /// Response format
    pub response_format: Option<String>,
    /// Temperature
    pub temperature: Option<f32>,
    /// Timestamp granularities
    pub timestamp_granularities: Option<Vec<String>>,
}

/// Moderation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModerationRequest {
    /// Model to use
    pub model: Option<String>,
    /// Input text(s)
    pub input: ModerationInput,
}

/// Moderation input
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ModerationInput {
    /// Single text string
    String(String),
    /// Array of text strings
    Array(Vec<String>),
}

/// Rerank request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankRequest {
    /// Model to use
    pub model: String,
    /// Query
    pub query: String,
    /// Documents to rerank
    pub documents: Vec<RerankDocument>,
    /// Top K results
    pub top_k: Option<u32>,
    /// Return documents
    pub return_documents: Option<bool>,
    /// Maximum chunks per document
    pub max_chunks_per_doc: Option<u32>,
}

/// Rerank document
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RerankDocument {
    /// Document as plain text
    String(String),
    /// Document as object with text field
    Object {
        /// Document text content
        text: String,
    },
}

/// Routing preferences
#[derive(Debug, Clone, Default)]
pub struct RoutingPreferences {
    /// Preferred providers (in order)
    pub preferred_providers: Vec<String>,
    /// Excluded providers
    pub excluded_providers: Vec<String>,
    /// Routing strategy override
    pub strategy_override: Option<String>,
    /// Tags for tag-based routing
    pub tags: Vec<String>,
    /// Region preference
    pub region: Option<String>,
    /// Cost optimization preference
    pub optimize_cost: bool,
    /// Latency optimization preference
    pub optimize_latency: bool,
}

/// Caching preferences
#[derive(Debug, Clone, Default)]
pub struct CachingPreferences {
    /// Enable caching for this request
    pub enabled: bool,
    /// Cache TTL override
    pub ttl_seconds: Option<u64>,
    /// Cache key prefix
    pub key_prefix: Option<String>,
    /// Enable semantic caching
    pub semantic_cache: bool,
    /// Semantic similarity threshold
    pub similarity_threshold: Option<f32>,
    /// Cache tags
    pub tags: Vec<String>,
}

impl GatewayRequest {
    /// Create a new gateway request
    pub fn new(request_type: RequestType, data: RequestData, context: RequestContext) -> Self {
        Self {
            metadata: Metadata::new(),
            context,
            request_type,
            data,
            provider_params: HashMap::new(),
            routing: RoutingPreferences::default(),
            caching: CachingPreferences::default(),
        }
    }

    /// Get the model name from the request
    pub fn model(&self) -> Option<&str> {
        match &self.data {
            RequestData::ChatCompletion(req) => Some(&req.model),
            RequestData::Completion(req) => Some(&req.model),
            RequestData::Embedding(req) => Some(&req.model),
            RequestData::ImageGeneration(req) => req.model.as_deref(),
            RequestData::AudioTranscription(req) => Some(&req.model),
            RequestData::Moderation(req) => req.model.as_deref(),
            RequestData::Rerank(req) => Some(&req.model),
        }
    }

    /// Check if the request is streaming
    pub fn is_streaming(&self) -> bool {
        match &self.data {
            RequestData::ChatCompletion(req) => req.stream.unwrap_or(false),
            RequestData::Completion(req) => req.stream.unwrap_or(false),
            _ => false,
        }
    }

    /// Get estimated token count for the request
    pub fn estimated_tokens(&self) -> Option<u32> {
        // This would be implemented with actual token counting logic
        // For now, return None
        None
    }

    /// Set provider parameter
    pub fn set_provider_param<K: Into<String>, V: Into<serde_json::Value>>(
        &mut self,
        key: K,
        value: V,
    ) {
        self.provider_params.insert(key.into(), value.into());
    }

    /// Get provider parameter
    pub fn get_provider_param(&self, key: &str) -> Option<&serde_json::Value> {
        self.provider_params.get(key)
    }

    /// Set routing preferences
    pub fn with_routing(mut self, routing: RoutingPreferences) -> Self {
        self.routing = routing;
        self
    }

    /// Set caching preferences
    pub fn with_caching(mut self, caching: CachingPreferences) -> Self {
        self.caching = caching;
        self
    }

    /// Add preferred provider
    pub fn add_preferred_provider<S: Into<String>>(mut self, provider: S) -> Self {
        self.routing.preferred_providers.push(provider.into());
        self
    }

    /// Exclude provider
    pub fn exclude_provider<S: Into<String>>(mut self, provider: S) -> Self {
        self.routing.excluded_providers.push(provider.into());
        self
    }

    /// Enable caching
    pub fn enable_caching(mut self, ttl_seconds: Option<u64>) -> Self {
        self.caching.enabled = true;
        self.caching.ttl_seconds = ttl_seconds;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gateway_request_creation() {
        let context = RequestContext::new();
        let chat_request = ChatCompletionRequest::default();
        let data = RequestData::ChatCompletion(Box::new(chat_request));

        let gateway_request = GatewayRequest::new(RequestType::ChatCompletion, data, context);

        assert!(matches!(
            gateway_request.request_type,
            RequestType::ChatCompletion
        ));
        assert!(matches!(
            gateway_request.data,
            RequestData::ChatCompletion(_)
        ));
    }

    #[test]
    fn test_model_extraction() {
        let context = RequestContext::new();
        let chat_request = ChatCompletionRequest {
            model: "gpt-4".to_string(),
            ..Default::default()
        };
        let data = RequestData::ChatCompletion(Box::new(chat_request));

        let gateway_request = GatewayRequest::new(RequestType::ChatCompletion, data, context);

        assert_eq!(gateway_request.model(), Some("gpt-4"));
    }

    #[test]
    fn test_streaming_detection() {
        let context = RequestContext::new();
        let chat_request = ChatCompletionRequest {
            stream: Some(true),
            ..Default::default()
        };
        let data = RequestData::ChatCompletion(Box::new(chat_request));

        let gateway_request = GatewayRequest::new(RequestType::ChatCompletion, data, context);

        assert!(gateway_request.is_streaming());
    }

    // ==================== RequestType Tests ====================

    #[test]
    fn test_request_type_serialization() {
        assert_eq!(
            serde_json::to_string(&RequestType::ChatCompletion).unwrap(),
            "\"chat_completion\""
        );
        assert_eq!(
            serde_json::to_string(&RequestType::Embedding).unwrap(),
            "\"embedding\""
        );
        assert_eq!(
            serde_json::to_string(&RequestType::ImageGeneration).unwrap(),
            "\"image_generation\""
        );
    }

    #[test]
    fn test_request_type_deserialization() {
        let chat: RequestType = serde_json::from_str("\"chat_completion\"").unwrap();
        assert!(matches!(chat, RequestType::ChatCompletion));

        let embed: RequestType = serde_json::from_str("\"embedding\"").unwrap();
        assert!(matches!(embed, RequestType::Embedding));
    }

    #[test]
    fn test_request_type_all_variants() {
        let types = vec![
            RequestType::ChatCompletion,
            RequestType::Completion,
            RequestType::Embedding,
            RequestType::ImageGeneration,
            RequestType::ImageEdit,
            RequestType::ImageVariation,
            RequestType::AudioTranscription,
            RequestType::AudioTranslation,
            RequestType::AudioSpeech,
            RequestType::Moderation,
            RequestType::FineTuning,
            RequestType::Files,
            RequestType::Assistants,
            RequestType::Threads,
            RequestType::Batches,
            RequestType::VectorStores,
            RequestType::Rerank,
            RequestType::Realtime,
        ];
        assert_eq!(types.len(), 18);
    }

    // ==================== RoutingPreferences Tests ====================

    #[test]
    fn test_routing_preferences_default() {
        let prefs = RoutingPreferences::default();
        assert!(prefs.preferred_providers.is_empty());
        assert!(prefs.excluded_providers.is_empty());
        assert!(prefs.strategy_override.is_none());
        assert!(prefs.tags.is_empty());
        assert!(prefs.region.is_none());
        assert!(!prefs.optimize_cost);
        assert!(!prefs.optimize_latency);
    }

    #[test]
    fn test_routing_preferences_structure() {
        let prefs = RoutingPreferences {
            preferred_providers: vec!["openai".to_string(), "anthropic".to_string()],
            excluded_providers: vec!["azure".to_string()],
            strategy_override: Some("least_latency".to_string()),
            tags: vec!["prod".to_string()],
            region: Some("us-east-1".to_string()),
            optimize_cost: true,
            optimize_latency: false,
        };
        assert_eq!(prefs.preferred_providers.len(), 2);
        assert_eq!(prefs.excluded_providers.len(), 1);
        assert!(prefs.optimize_cost);
    }

    // ==================== CachingPreferences Tests ====================

    #[test]
    fn test_caching_preferences_default() {
        let prefs = CachingPreferences::default();
        assert!(!prefs.enabled);
        assert!(prefs.ttl_seconds.is_none());
        assert!(prefs.key_prefix.is_none());
        assert!(!prefs.semantic_cache);
        assert!(prefs.similarity_threshold.is_none());
        assert!(prefs.tags.is_empty());
    }

    #[test]
    fn test_caching_preferences_structure() {
        let prefs = CachingPreferences {
            enabled: true,
            ttl_seconds: Some(3600),
            key_prefix: Some("cache:v1:".to_string()),
            semantic_cache: true,
            similarity_threshold: Some(0.95),
            tags: vec!["user:123".to_string()],
        };
        assert!(prefs.enabled);
        assert_eq!(prefs.ttl_seconds, Some(3600));
        assert!(prefs.semantic_cache);
    }

    // ==================== GatewayRequest Methods Tests ====================

    #[test]
    fn test_gateway_request_provider_params() {
        let context = RequestContext::new();
        let chat_request = ChatCompletionRequest::default();
        let data = RequestData::ChatCompletion(Box::new(chat_request));
        let mut request = GatewayRequest::new(RequestType::ChatCompletion, data, context);

        request.set_provider_param("temperature", serde_json::json!(0.7));
        request.set_provider_param("max_tokens", serde_json::json!(1000));

        assert_eq!(
            request.get_provider_param("temperature"),
            Some(&serde_json::json!(0.7))
        );
        assert_eq!(
            request.get_provider_param("max_tokens"),
            Some(&serde_json::json!(1000))
        );
        assert!(request.get_provider_param("nonexistent").is_none());
    }

    #[test]
    fn test_gateway_request_with_routing() {
        let context = RequestContext::new();
        let chat_request = ChatCompletionRequest::default();
        let data = RequestData::ChatCompletion(Box::new(chat_request));
        let routing = RoutingPreferences {
            preferred_providers: vec!["openai".to_string()],
            optimize_latency: true,
            ..Default::default()
        };

        let request = GatewayRequest::new(RequestType::ChatCompletion, data, context)
            .with_routing(routing);

        assert_eq!(request.routing.preferred_providers.len(), 1);
        assert!(request.routing.optimize_latency);
    }

    #[test]
    fn test_gateway_request_with_caching() {
        let context = RequestContext::new();
        let chat_request = ChatCompletionRequest::default();
        let data = RequestData::ChatCompletion(Box::new(chat_request));
        let caching = CachingPreferences {
            enabled: true,
            semantic_cache: true,
            ..Default::default()
        };

        let request = GatewayRequest::new(RequestType::ChatCompletion, data, context)
            .with_caching(caching);

        assert!(request.caching.enabled);
        assert!(request.caching.semantic_cache);
    }

    #[test]
    fn test_gateway_request_add_preferred_provider() {
        let context = RequestContext::new();
        let chat_request = ChatCompletionRequest::default();
        let data = RequestData::ChatCompletion(Box::new(chat_request));

        let request = GatewayRequest::new(RequestType::ChatCompletion, data, context)
            .add_preferred_provider("openai")
            .add_preferred_provider("anthropic");

        assert_eq!(request.routing.preferred_providers.len(), 2);
        assert!(request.routing.preferred_providers.contains(&"openai".to_string()));
    }

    #[test]
    fn test_gateway_request_exclude_provider() {
        let context = RequestContext::new();
        let chat_request = ChatCompletionRequest::default();
        let data = RequestData::ChatCompletion(Box::new(chat_request));

        let request = GatewayRequest::new(RequestType::ChatCompletion, data, context)
            .exclude_provider("azure");

        assert_eq!(request.routing.excluded_providers.len(), 1);
        assert!(request.routing.excluded_providers.contains(&"azure".to_string()));
    }

    #[test]
    fn test_gateway_request_enable_caching() {
        let context = RequestContext::new();
        let chat_request = ChatCompletionRequest::default();
        let data = RequestData::ChatCompletion(Box::new(chat_request));

        let request = GatewayRequest::new(RequestType::ChatCompletion, data, context)
            .enable_caching(Some(7200));

        assert!(request.caching.enabled);
        assert_eq!(request.caching.ttl_seconds, Some(7200));
    }

    #[test]
    fn test_gateway_request_estimated_tokens() {
        let context = RequestContext::new();
        let chat_request = ChatCompletionRequest::default();
        let data = RequestData::ChatCompletion(Box::new(chat_request));
        let request = GatewayRequest::new(RequestType::ChatCompletion, data, context);

        assert!(request.estimated_tokens().is_none());
    }

    // ==================== CompletionRequest Tests ====================

    #[test]
    fn test_completion_request_structure() {
        let request = CompletionRequest {
            model: "gpt-3.5-turbo-instruct".to_string(),
            prompt: Some("Complete this:".to_string()),
            max_tokens: Some(100),
            temperature: Some(0.7),
            top_p: Some(0.9),
            n: Some(1),
            stream: Some(false),
            stop: Some(vec!["\n".to_string()]),
            presence_penalty: Some(0.0),
            frequency_penalty: Some(0.0),
            logit_bias: None,
            user: Some("user123".to_string()),
            suffix: None,
            echo: Some(false),
            best_of: Some(1),
            logprobs: None,
        };
        assert_eq!(request.model, "gpt-3.5-turbo-instruct");
        assert_eq!(request.max_tokens, Some(100));
    }

    // ==================== EmbeddingInput Tests ====================

    #[test]
    fn test_embedding_input_string() {
        let input = EmbeddingInput::String("Hello world".to_string());
        let json = serde_json::to_value(&input).unwrap();
        assert_eq!(json, "Hello world");
    }

    #[test]
    fn test_embedding_input_array() {
        let input = EmbeddingInput::Array(vec!["Hello".to_string(), "World".to_string()]);
        let json = serde_json::to_value(&input).unwrap();
        assert!(json.is_array());
        assert_eq!(json.as_array().unwrap().len(), 2);
    }

    // ==================== ModerationInput Tests ====================

    #[test]
    fn test_moderation_input_string() {
        let input = ModerationInput::String("Check this text".to_string());
        let json = serde_json::to_value(&input).unwrap();
        assert_eq!(json, "Check this text");
    }

    #[test]
    fn test_moderation_input_array() {
        let input = ModerationInput::Array(vec!["Text 1".to_string(), "Text 2".to_string()]);
        let json = serde_json::to_value(&input).unwrap();
        assert!(json.is_array());
    }

    // ==================== RerankDocument Tests ====================

    #[test]
    fn test_rerank_document_string() {
        let doc = RerankDocument::String("Document content".to_string());
        let json = serde_json::to_value(&doc).unwrap();
        assert_eq!(json, "Document content");
    }

    #[test]
    fn test_rerank_document_object() {
        let doc = RerankDocument::Object {
            text: "Document with object".to_string(),
        };
        let json = serde_json::to_value(&doc).unwrap();
        assert_eq!(json["text"], "Document with object");
    }

    // ==================== Model Extraction for Different Types ====================

    #[test]
    fn test_model_extraction_completion() {
        let context = RequestContext::new();
        let request = CompletionRequest {
            model: "text-davinci-003".to_string(),
            prompt: None,
            max_tokens: None,
            temperature: None,
            top_p: None,
            n: None,
            stream: None,
            stop: None,
            presence_penalty: None,
            frequency_penalty: None,
            logit_bias: None,
            user: None,
            suffix: None,
            echo: None,
            best_of: None,
            logprobs: None,
        };
        let data = RequestData::Completion(request);
        let gateway_request = GatewayRequest::new(RequestType::Completion, data, context);
        assert_eq!(gateway_request.model(), Some("text-davinci-003"));
    }

    #[test]
    fn test_model_extraction_embedding() {
        let context = RequestContext::new();
        let request = EmbeddingRequest {
            model: "text-embedding-ada-002".to_string(),
            input: EmbeddingInput::String("test".to_string()),
            encoding_format: None,
            dimensions: None,
            user: None,
        };
        let data = RequestData::Embedding(request);
        let gateway_request = GatewayRequest::new(RequestType::Embedding, data, context);
        assert_eq!(gateway_request.model(), Some("text-embedding-ada-002"));
    }

    #[test]
    fn test_model_extraction_rerank() {
        let context = RequestContext::new();
        let request = RerankRequest {
            model: "rerank-english-v2.0".to_string(),
            query: "test query".to_string(),
            documents: vec![],
            top_k: None,
            return_documents: None,
            max_chunks_per_doc: None,
        };
        let data = RequestData::Rerank(request);
        let gateway_request = GatewayRequest::new(RequestType::Rerank, data, context);
        assert_eq!(gateway_request.model(), Some("rerank-english-v2.0"));
    }

    #[test]
    fn test_is_streaming_completion() {
        let context = RequestContext::new();
        let request = CompletionRequest {
            model: "model".to_string(),
            stream: Some(true),
            prompt: None,
            max_tokens: None,
            temperature: None,
            top_p: None,
            n: None,
            stop: None,
            presence_penalty: None,
            frequency_penalty: None,
            logit_bias: None,
            user: None,
            suffix: None,
            echo: None,
            best_of: None,
            logprobs: None,
        };
        let data = RequestData::Completion(request);
        let gateway_request = GatewayRequest::new(RequestType::Completion, data, context);
        assert!(gateway_request.is_streaming());
    }

    #[test]
    fn test_is_streaming_embedding() {
        let context = RequestContext::new();
        let request = EmbeddingRequest {
            model: "model".to_string(),
            input: EmbeddingInput::String("test".to_string()),
            encoding_format: None,
            dimensions: None,
            user: None,
        };
        let data = RequestData::Embedding(request);
        let gateway_request = GatewayRequest::new(RequestType::Embedding, data, context);
        assert!(!gateway_request.is_streaming());
    }
}
