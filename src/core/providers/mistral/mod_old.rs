//! Mistral AI Provider
//!
//! Mistral AI model integration for LiteLLM.
//! This provider supports Mistral's API for various model sizes and capabilities.
//!
//! ## Features
//! - Support for all Mistral models (Tiny, Small, Medium, Large)
//! - Function calling support
//! - JSON mode response format
//! - Streaming support
//! - Embeddings support
//!
//! ## Documentation
//! - API Docs: https://docs.mistral.ai/api/

use async_trait::async_trait;
use futures::Stream;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use tracing::{debug, info};

pub mod chat;
pub mod common_utils;
pub mod embedding;

// Re-export main components
pub use chat::{MistralChatHandler, MistralChatTransformation};
pub use common_utils::{MistralClient, MistralConfig, MistralError, MistralUtils};
pub use embedding::MistralEmbeddingHandler;

// Cost calculation removed - integrated in provider implementation
use crate::core::traits::{ErrorMapper, ProviderConfig, provider::LLMProvider};
use crate::core::types::{
    common::{HealthStatus, ModelInfo, ProviderCapability, RequestContext},
    requests::{ChatRequest, EmbeddingRequest, ImageGenerationRequest},
    responses::{ChatChunk, ChatResponse, EmbeddingResponse, ImageGenerationResponse},
};

// Static capabilities for Mistral provider
const MISTRAL_CAPABILITIES: &[ProviderCapability] = &[
    ProviderCapability::ChatCompletion,
    ProviderCapability::ChatCompletionStream,
    ProviderCapability::ToolCalling,
    ProviderCapability::Embeddings,
];

/// Mistral provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MistralProviderConfig {
    /// API key for authentication
    pub api_key: String,
    /// API base URL (defaults to https://api.mistral.ai/v1)
    pub api_base: Option<String>,
    /// Request timeout in seconds
    pub timeout: Option<u64>,
    /// Maximum retries for failed requests
    pub max_retries: Option<u32>,
    /// Custom headers
    pub headers: Option<HashMap<String, String>>,
    /// Supported models
    pub supported_models: Vec<String>,
    /// Provider metadata
    pub metadata: HashMap<String, String>,
    /// Whether to use enhanced cost calculation features
    pub enable_cost_tracking: bool,
}

impl Default for MistralProviderConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            api_base: Some("https://api.mistral.ai/v1".to_string()),
            timeout: Some(30),
            max_retries: Some(3),
            headers: None,
            supported_models: vec![
                // Open models
                "open-mistral-7b".to_string(),
                "open-mixtral-8x7b".to_string(),
                "open-mixtral-8x22b".to_string(),
                // Commercial models
                "mistral-tiny".to_string(),
                "mistral-small".to_string(),
                "mistral-small-latest".to_string(),
                "mistral-medium".to_string(),
                "mistral-medium-latest".to_string(),
                "mistral-large".to_string(),
                "mistral-large-latest".to_string(),
                // Specialized models
                "codestral-latest".to_string(),
                "mistral-embed".to_string(),
            ],
            metadata: HashMap::new(),
            enable_cost_tracking: true,
        }
    }
}

/// Mistral provider implementation
#[derive(Debug)]
pub struct MistralProvider {
    config: Arc<MistralProviderConfig>,
    client: Arc<MistralClient>,
    chat_handler: Arc<MistralChatHandler>,
    embedding_handler: Arc<MistralEmbeddingHandler>,
    // Cost calculation integrated internally
    models: Vec<ModelInfo>,
}

impl MistralProvider {
    /// Create a new Mistral provider instance
    pub fn new(config: MistralProviderConfig) -> Result<Self, MistralError> {
        let mistral_config = MistralConfig::from_provider_config(&config)?;
        let client = MistralClient::new(mistral_config.clone())?;
        let chat_handler = MistralChatHandler::new(mistral_config.clone())?;
        let embedding_handler = MistralEmbeddingHandler::new(mistral_config.clone())?;
        // Cost calculation integrated in provider implementation

        let models = vec![
            ModelInfo {
                id: "mistral-tiny".to_string(),
                name: "Mistral Tiny".to_string(),
                provider: "mistral".to_string(),
                max_context_length: 32000,
                max_output_length: None,
                supports_streaming: true,
                supports_tools: true,
                supports_multimodal: false,
                input_cost_per_1k_tokens: Some(0.0002),
                output_cost_per_1k_tokens: Some(0.0006),
                currency: "USD".to_string(),
                capabilities: vec![],
                created_at: None,
                updated_at: None,
                metadata: HashMap::new(),
            },
            ModelInfo {
                id: "mistral-small".to_string(),
                name: "Mistral Small".to_string(),
                provider: "mistral".to_string(),
                max_context_length: 32000,
                max_output_length: None,
                supports_streaming: true,
                supports_tools: true,
                supports_multimodal: false,
                input_cost_per_1k_tokens: Some(0.001),
                output_cost_per_1k_tokens: Some(0.003),
                currency: "USD".to_string(),
                capabilities: vec![],
                created_at: None,
                updated_at: None,
                metadata: HashMap::new(),
            },
        ];

        Ok(Self {
            config: Arc::new(config),
            client: Arc::new(client),
            chat_handler: Arc::new(chat_handler),
            embedding_handler: Arc::new(embedding_handler),
            models,
        })
    }

    /// Get the API base URL
    pub fn get_api_base(&self) -> String {
        self.config
            .api_base
            .clone()
            .unwrap_or_else(|| "https://api.mistral.ai/v1".to_string())
    }

    /// Check if a model is supported
    pub fn is_model_supported(&self, model: &str) -> bool {
        self.config
            .supported_models
            .iter()
            .any(|m| m == model || model.contains(m) || m.contains(model))
    }

    /// Check if model is an embedding model
    pub fn is_embedding_model(&self, model: &str) -> bool {
        model.contains("embed")
    }

    /// Get provider capabilities
    pub fn get_capabilities(&self) -> &'static [ProviderCapability] {
        MISTRAL_CAPABILITIES
    }
}

#[async_trait]
impl LLMProvider for MistralProvider {
    type Config = MistralProviderConfig;
    type Error = MistralError;
    type ErrorMapper = MistralErrorMapper;

    fn name(&self) -> &'static str {
        "mistral"
    }
    async fn chat_completion(
        &self,
        request: ChatRequest,
        _context: RequestContext,
    ) -> Result<ChatResponse, Self::Error> {
        debug!("Mistral chat completion request: model={}", request.model);

        // Like Python LiteLLM, we don't validate models locally
        // Mistral API will handle invalid models

        // Transform request to Mistral format
        let mistral_request = self.chat_handler.transform_request(request)?;

        // Make API call
        let response = self
            .client
            .chat_completion(
                mistral_request,
                Some(&self.config.api_key),
                self.config.api_base.as_deref(),
                self.config.headers.clone(),
            )
            .await?;

        // Transform response back to standard format
        let chat_response = self.chat_handler.transform_response(response)?;

        info!(
            "Mistral chat completion successful: model={}",
            chat_response.model
        );
        Ok(chat_response)
    }

    async fn chat_completion_stream(
        &self,
        request: ChatRequest,
        _context: RequestContext,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatChunk, Self::Error>> + Send>>, Self::Error>
    {
        debug!("Mistral streaming chat request: model={}", request.model);

        // Like Python LiteLLM, we don't validate models locally
        // Mistral API will handle invalid models

        // Clone Arc references for the stream to own
        let client = Arc::clone(&self.client);
        let config = Arc::clone(&self.config);
        let chat_handler = Arc::clone(&self.chat_handler);

        // Transform request and enable streaming
        let mut mistral_request = chat_handler.transform_request(request)?;
        mistral_request["stream"] = serde_json::json!(true);

        // Get stream from client using owned data
        let api_key = Some(config.api_key.clone());
        let api_base = config.api_base.clone();
        let headers = config.headers.clone();

        let stream = client
            .chat_completion_stream(
                mistral_request,
                api_key.as_deref(),
                api_base.as_deref(),
                headers,
            )
            .await?;

        // Convert JsonValue stream to ChatChunk stream
        use futures::stream::StreamExt;
        let chunk_stream = stream.map(|result| {
            result.and_then(|json| {
                // Convert JsonValue to ChatChunk
                Ok(crate::core::types::responses::ChatChunk {
                    id: json
                        .get("id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    object: "chat.completion.chunk".to_string(),
                    created: json.get("created").and_then(|v| v.as_i64()).unwrap_or(0),
                    model: json
                        .get("model")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    choices: vec![],
                    usage: None,
                    system_fingerprint: None,
                })
            })
        });

        Ok(Box::pin(chunk_stream))
    }

    async fn embeddings(
        &self,
        request: EmbeddingRequest,
        _context: RequestContext,
    ) -> Result<EmbeddingResponse, Self::Error> {
        debug!("Mistral embedding request: model={}", request.model);

        // Validate model support
        if !self.is_embedding_model(&request.model) {
            return Err(MistralError::InvalidRequest(format!(
                "Model {} is not an embedding model. Use mistral-embed instead.",
                request.model
            )));
        }

        // Transform request to Mistral format
        let mistral_request = self.embedding_handler.transform_request(request)?;

        // Make API call
        let response = self
            .client
            .embedding(
                mistral_request,
                Some(&self.config.api_key),
                self.config.api_base.as_deref(),
                self.config.headers.clone(),
            )
            .await?;

        // Transform response back to standard format
        let embedding_response = self.embedding_handler.transform_response(response)?;

        info!("Mistral embedding successful");
        Ok(embedding_response)
    }

    async fn image_generation(
        &self,
        _request: ImageGenerationRequest,
        _context: RequestContext,
    ) -> Result<ImageGenerationResponse, Self::Error> {
        // Mistral doesn't support image generation
        Err(MistralError::not_supported("image generation"))
    }

    async fn health_check(&self) -> HealthStatus {
        // Perform a simple API call to check health
        match self.client.check_health().await {
            Ok(status) => status,
            Err(_) => HealthStatus::Unhealthy,
        }
    }

    fn capabilities(&self) -> &'static [ProviderCapability] {
        MISTRAL_CAPABILITIES
    }

    fn models(&self) -> &[ModelInfo] {
        &self.models
    }

    fn get_supported_openai_params(&self, _model: &str) -> &'static [&'static str] {
        &[
            "messages",
            "model",
            "max_tokens",
            "temperature",
            "top_p",
            "stream",
            "stop",
            "random_seed",
            "response_format",
            "tools",
            "tool_choice",
            "safe_prompt",
        ]
    }

    async fn map_openai_params(
        &self,
        mut params: HashMap<String, Value>,
        _model: &str,
    ) -> Result<HashMap<String, Value>, Self::Error> {
        // Mistral uses random_seed instead of seed
        if let Some(seed) = params.remove("seed") {
            params.insert("random_seed".to_string(), seed);
        }

        // Add safe_prompt by default
        params.insert("safe_prompt".to_string(), serde_json::json!(true));

        Ok(params)
    }

    async fn transform_request(
        &self,
        request: ChatRequest,
        _context: RequestContext,
    ) -> Result<Value, Self::Error> {
        self.chat_handler
            .transform_request(request)
            .map_err(MistralError::from)
    }

    async fn transform_response(
        &self,
        raw_response: &[u8],
        _model: &str,
        _request_id: &str,
    ) -> Result<ChatResponse, Self::Error> {
        let response: Value = serde_json::from_slice(raw_response)?;
        self.chat_handler
            .transform_response(response)
            .map_err(MistralError::from)
    }

    fn get_error_mapper(&self) -> Self::ErrorMapper {
        MistralErrorMapper
    }

    async fn calculate_cost(
        &self,
        model: &str,
        input_tokens: u32,
        output_tokens: u32,
    ) -> Result<f64, Self::Error> {
        // Basic cost calculation for Mistral models (per 1M tokens)
        let cost = match model {
            "open-mistral-7b" => {
                (input_tokens as f64 * 0.00025 + output_tokens as f64 * 0.00025) / 1000.0
            }
            "open-mixtral-8x7b" => {
                (input_tokens as f64 * 0.0007 + output_tokens as f64 * 0.0007) / 1000.0
            }
            "open-mixtral-8x22b" => {
                (input_tokens as f64 * 0.002 + output_tokens as f64 * 0.006) / 1000.0
            }
            "mistral-small-latest" => {
                (input_tokens as f64 * 0.001 + output_tokens as f64 * 0.003) / 1000.0
            }
            "mistral-medium-latest" => {
                (input_tokens as f64 * 0.0027 + output_tokens as f64 * 0.0081) / 1000.0
            }
            "mistral-large-latest" => {
                (input_tokens as f64 * 0.004 + output_tokens as f64 * 0.012) / 1000.0
            }
            _ => 0.0, // Default cost for unknown models
        };
        debug!(
            "Mistral cost calculation: model={}, input_tokens={}, output_tokens={}, total_cost=${:.6}",
            model, input_tokens, output_tokens, cost
        );
        Ok(cost)
    }
}

impl ProviderConfig for MistralProviderConfig {
    fn validate(&self) -> Result<(), String> {
        if self.api_key.is_empty() {
            return Err("API key is required for Mistral provider".to_string());
        }

        if let Some(timeout) = self.timeout {
            if timeout == 0 {
                return Err("Timeout must be greater than 0".to_string());
            }
        }

        Ok(())
    }

    fn api_key(&self) -> Option<&str> {
        if self.api_key.is_empty() {
            None
        } else {
            Some(&self.api_key)
        }
    }

    fn api_base(&self) -> Option<&str> {
        self.api_base.as_deref()
    }

    fn timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.timeout.unwrap_or(30))
    }

    fn max_retries(&self) -> u32 {
        self.max_retries.unwrap_or(3)
    }
}

/// Mistral ErrorMapper implementation
#[derive(Debug, Clone)]
pub struct MistralErrorMapper;

impl ErrorMapper<MistralError> for MistralErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> MistralError {
        match status_code {
            400 => MistralError::InvalidRequest(format!("Bad request: {}", response_body)),
            401 => MistralError::Authentication("Invalid Mistral API key".to_string()),
            403 => MistralError::Authentication("Forbidden: insufficient permissions".to_string()),
            404 => MistralError::ModelNotFound("Model not found".to_string()),
            429 => MistralError::RateLimit("Rate limit exceeded".to_string()),
            500..=599 => MistralError::ApiRequest(format!("Server error: {}", response_body)),
            _ => MistralError::ApiRequest(format!("HTTP error {}: {}", status_code, response_body)),
        }
    }

    fn map_json_error(&self, error_response: &serde_json::Value) -> MistralError {
        if let Some(error) = error_response.get("error") {
            let error_message = error
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error");

            if let Some(error_type) = error.get("type").and_then(|t| t.as_str()) {
                match error_type {
                    "invalid_api_key" | "authentication_error" => {
                        MistralError::Authentication(error_message.to_string())
                    }
                    "model_not_found" => MistralError::ModelNotFound(error_message.to_string()),
                    "rate_limit_exceeded" => MistralError::RateLimit(error_message.to_string()),
                    "invalid_request_error" => {
                        MistralError::InvalidRequest(error_message.to_string())
                    }
                    _ => MistralError::ApiRequest(format!("{}: {}", error_type, error_message)),
                }
            } else {
                MistralError::ApiRequest(error_message.to_string())
            }
        } else {
            MistralError::Serialization("Unknown error response format".to_string())
        }
    }

    fn map_network_error(&self, error: &dyn std::error::Error) -> MistralError {
        MistralError::Network(format!("Network error: {}", error))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = MistralProviderConfig::default();
        assert_eq!(config.api_base.unwrap(), "https://api.mistral.ai/v1");
        assert_eq!(config.timeout.unwrap(), 30);
        assert!(!config.supported_models.is_empty());
    }

    #[test]
    fn test_model_support() {
        let config = MistralProviderConfig::default();
        let provider = MistralProvider::new(config).unwrap();

        assert!(provider.is_model_supported("mistral-tiny"));
        assert!(provider.is_model_supported("mistral-large"));
        assert!(provider.is_model_supported("open-mixtral-8x7b"));
        assert!(!provider.is_model_supported("gpt-4"));
    }

    #[test]
    fn test_embedding_model_detection() {
        let config = MistralProviderConfig::default();
        let provider = MistralProvider::new(config).unwrap();

        assert!(provider.is_embedding_model("mistral-embed"));
        assert!(!provider.is_embedding_model("mistral-tiny"));
        assert!(!provider.is_embedding_model("mistral-large"));
    }

    #[test]
    fn test_capabilities() {
        let config = MistralProviderConfig::default();
        let provider = MistralProvider::new(config).unwrap();
        let capabilities = provider.get_capabilities();

        assert!(capabilities.contains(&ProviderCapability::ChatCompletion));
        assert!(capabilities.contains(&ProviderCapability::ChatCompletionStream));
        assert!(capabilities.contains(&ProviderCapability::ToolCalling));
        assert!(capabilities.contains(&ProviderCapability::Embeddings));
        assert!(!capabilities.contains(&ProviderCapability::ImageGeneration));
    }
}
