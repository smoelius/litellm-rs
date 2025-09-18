//! Gemini Provider Implementation
//!
//! Implementation

use async_trait::async_trait;
use futures::Stream;
use serde_json::Value;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;

use crate::core::providers::base::GlobalPoolManager;
use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::{LLMProvider, ProviderConfig};
use crate::core::types::{
    common::{HealthStatus, ModelInfo, ProviderCapability, RequestContext},
    requests::{ChatRequest, EmbeddingRequest, ImageGenerationRequest},
    responses::{ChatChunk, ChatResponse, EmbeddingResponse, ImageGenerationResponse},
};

use super::client::GeminiClient;
use super::config::GeminiConfig;
use super::error::{GeminiError, GeminiErrorMapper, gemini_model_error, gemini_validation_error};
use super::models::{ModelFeature, get_gemini_registry};
use super::streaming::GeminiStream;

/// Gemini Provider - Unified implementation
#[derive(Debug)]
pub struct GeminiProvider {
    config: GeminiConfig,
    client: GeminiClient,
    pool_manager: Arc<GlobalPoolManager>,
    supported_models: Vec<ModelInfo>,
}

impl GeminiProvider {
    /// Create
    pub fn new(config: GeminiConfig) -> Result<Self, ProviderError> {
        // Configuration
        config
            .validate()
            .map_err(|e| ProviderError::configuration("gemini", e))?;

        // Create
        let client = GeminiClient::new(config.clone())?;

        // Get
        let pool_manager = Arc::new(GlobalPoolManager::new()?);

        // Get
        let registry = get_gemini_registry();
        let supported_models = registry
            .list_models()
            .into_iter()
            .map(|spec| spec.model_info.clone())
            .collect();

        Ok(Self {
            config,
            client,
            pool_manager,
            supported_models,
        })
    }

    /// Request
    fn validate_request(&self, request: &ChatRequest) -> Result<(), ProviderError> {
        let registry = get_gemini_registry();

        // Check
        let model_spec = registry
            .get_model_spec(&request.model)
            .ok_or_else(|| gemini_model_error(format!("Unsupported model: {}", request.model)))?;

        // Check
        if request.messages.is_empty() {
            return Err(gemini_validation_error("Messages cannot be empty"));
        }

        // Check
        if let Some(max_tokens) = request.max_tokens {
            if max_tokens > model_spec.limits.max_output_tokens {
                return Err(gemini_validation_error(format!(
                    "max_tokens ({}) exceeds model limit ({})",
                    max_tokens, model_spec.limits.max_output_tokens
                )));
            }
        }

        // Check
        if let Some(temperature) = request.temperature {
            if !(0.0..=2.0).contains(&temperature) {
                return Err(gemini_validation_error(
                    "temperature must be between 0.0 and 2.0",
                ));
            }
        }

        // Check
        if let Some(top_p) = request.top_p {
            if !(0.0..=1.0).contains(&top_p) {
                return Err(gemini_validation_error("top_p must be between 0.0 and 1.0"));
            }
        }

        // Check
        if request.tools.is_some() && !model_spec.features.contains(&ModelFeature::ToolCalling) {
            return Err(gemini_validation_error(format!(
                "Model {} does not support tool calling",
                request.model
            )));
        }

        Ok(())
    }

    /// Get
    pub fn calculate_cost(
        &self,
        model: &str,
        input_tokens: u32,
        output_tokens: u32,
    ) -> Option<f64> {
        super::models::CostCalculator::calculate_cost(model, input_tokens, output_tokens)
    }
}

#[async_trait]
impl LLMProvider for GeminiProvider {
    type Config = GeminiConfig;
    type Error = GeminiError;
    type ErrorMapper = GeminiErrorMapper;

    fn name(&self) -> &'static str {
        "gemini"
    }

    fn capabilities(&self) -> &'static [ProviderCapability] {
        &[
            ProviderCapability::ChatCompletion,
            ProviderCapability::ChatCompletionStream,
            ProviderCapability::ToolCalling,
            // ProviderCapability::Vision, // TODO: Add to enum
        ]
    }

    fn models(&self) -> &[ModelInfo] {
        &self.supported_models
    }

    fn supports_model(&self, model: &str) -> bool {
        get_gemini_registry().get_model_spec(model).is_some()
    }

    fn supports_tools(&self) -> bool {
        true // Gemini supports tool calling
    }

    fn supports_streaming(&self) -> bool {
        true // Streaming support
    }

    fn supports_image_generation(&self) -> bool {
        false // Gemini currently does not support image generation
    }

    fn supports_embeddings(&self) -> bool {
        false // TODO: Can be supported through dedicated embedding models
    }

    fn supports_vision(&self) -> bool {
        true // Gemini supports vision understanding
    }

    fn get_supported_openai_params(&self, _model: &str) -> &'static [&'static str] {
        &[
            "temperature",
            "max_tokens",
            "top_p",
            "stop",
            "stream",
            "tools",
            "tool_choice",
        ]
    }

    async fn map_openai_params(
        &self,
        params: HashMap<String, Value>,
        _model: &str,
    ) -> Result<HashMap<String, Value>, Self::Error> {
        let mut mapped = HashMap::new();

        for (key, value) in params {
            match key.as_str() {
                // Directly mapped parameters
                "temperature" | "top_p" | "stop" | "stream" => {
                    mapped.insert(key, value);
                }
                "max_tokens" => {
                    mapped.insert("max_output_tokens".to_string(), value);
                }
                // Handle tools
                "tools" | "tool_choice" => {
                    mapped.insert(key, value);
                }
                // Ignore unsupported parameters
                "frequency_penalty" | "presence_penalty" | "logit_bias" => {
                    // Gemini doesn't support these parameters, skip
                }
                // Keep other parameters as-is
                _ => {
                    mapped.insert(key, value);
                }
            }
        }

        Ok(mapped)
    }

    async fn transform_request(
        &self,
        request: ChatRequest,
        _context: RequestContext,
    ) -> Result<Value, Self::Error> {
        // Request
        self.validate_request(&request)?;

        // Use client's transformation method
        let transformed = self.client.transform_chat_request(&request)?;
        Ok(transformed)
    }

    async fn transform_response(
        &self,
        raw_response: &[u8],
        model: &str,
        _request_id: &str,
    ) -> Result<ChatResponse, Self::Error> {
        let response_text = String::from_utf8_lossy(raw_response);
        let response_json: Value = serde_json::from_str(&response_text).map_err(|e| {
            ProviderError::serialization("gemini", format!("Failed to parse response: {}", e))
        })?;

        // Error
        if response_json.get("error").is_some() {
            return Err(GeminiErrorMapper::from_api_response(&response_json));
        }

        // Request
        let dummy_request = ChatRequest {
            model: model.to_string(),
            messages: vec![],
            temperature: None,
            max_tokens: None,
            max_completion_tokens: None,
            top_p: None,
            n: None,
            stream: false,
            stop: None,
            presence_penalty: None,
            frequency_penalty: None,
            logit_bias: None,
            logprobs: None,
            top_logprobs: None,
            user: None,
            tools: None,
            tool_choice: None,
            parallel_tool_calls: None,
            response_format: None,
            seed: None,
            functions: None,
            function_call: None,
            extra_params: std::collections::HashMap::new(),
        };

        self.client
            .transform_chat_response(response_json, &dummy_request)
    }

    fn get_error_mapper(&self) -> Self::ErrorMapper {
        GeminiErrorMapper
    }

    async fn chat_completion(
        &self,
        request: ChatRequest,
        _context: RequestContext,
    ) -> Result<ChatResponse, Self::Error> {
        // Request
        self.validate_request(&request)?;

        // Request
        self.client.chat(request).await
    }

    async fn chat_completion_stream(
        &self,
        request: ChatRequest,
        _context: RequestContext,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatChunk, Self::Error>> + Send>>, Self::Error>
    {
        // Request
        self.validate_request(&request)?;

        // Request
        let response = self.client.chat_stream(request.clone()).await?;

        // Create stream
        let stream = GeminiStream::from_response(response, request.model);

        Ok(Box::pin(stream))
    }

    async fn embeddings(
        &self,
        _request: EmbeddingRequest,
        _context: RequestContext,
    ) -> Result<EmbeddingResponse, Self::Error> {
        Err(ProviderError::NotSupported {
            provider: "gemini",
            feature: "embeddings: not yet implemented for Gemini provider".to_string(),
        })
    }

    async fn image_generation(
        &self,
        _request: ImageGenerationRequest,
        _context: RequestContext,
    ) -> Result<ImageGenerationResponse, Self::Error> {
        Err(ProviderError::NotSupported {
            provider: "gemini",
            feature: "image_generation: not supported by Gemini provider".to_string(),
        })
    }

    async fn health_check(&self) -> HealthStatus {
        // Health check request
        let test_request = ChatRequest {
            model: "gemini-1.0-pro".to_string(),
            messages: vec![crate::core::types::ChatMessage {
                role: crate::core::types::MessageRole::User,
                content: Some(crate::core::types::MessageContent::Text("Hi".to_string())),
                name: None,
                tool_calls: None,
                tool_call_id: None,
                function_call: None,
            }],
            temperature: Some(0.1),
            max_tokens: Some(5),
            max_completion_tokens: None,
            top_p: None,
            n: None,
            stream: false,
            stop: None,
            presence_penalty: None,
            frequency_penalty: None,
            logit_bias: None,
            logprobs: None,
            top_logprobs: None,
            user: None,
            tools: None,
            tool_choice: None,
            parallel_tool_calls: None,
            response_format: None,
            seed: None,
            functions: None,
            function_call: None,
            extra_params: std::collections::HashMap::new(),
        };

        match self.client.chat(test_request).await {
            Ok(_) => HealthStatus::Healthy,
            Err(e) => match &e {
                ProviderError::Authentication { .. } => HealthStatus::Unhealthy,
                ProviderError::RateLimit { .. } => HealthStatus::Degraded,
                ProviderError::Network { .. } => HealthStatus::Degraded,
                _ => HealthStatus::Unhealthy,
            },
        }
    }

    async fn calculate_cost(
        &self,
        model: &str,
        input_tokens: u32,
        output_tokens: u32,
    ) -> Result<f64, Self::Error> {
        Ok(
            super::models::CostCalculator::calculate_cost(model, input_tokens, output_tokens)
                .unwrap_or(0.0),
        )
    }
}

// GeminiError is a type alias for ProviderError, so we don't need to implement traits for it
// The error mapping is handled by GeminiErrorMapper in error.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_creation() {
        let config = GeminiConfig::new_google_ai("test-api-key-12345678901234567890");
        let provider = GeminiProvider::new(config);
        assert!(provider.is_ok());
    }

    #[test]
    fn test_provider_capabilities() {
        let config = GeminiConfig::new_google_ai("test-api-key-12345678901234567890");
        let provider = GeminiProvider::new(config).unwrap();

        assert_eq!(provider.name(), "gemini");
        assert!(provider.supports_streaming());
        assert!(provider.supports_tools());
        assert!(provider.supports_vision());
        assert!(!provider.supports_embeddings());
        assert!(!provider.supports_image_generation());
    }

    #[test]
    fn test_model_support() {
        let config = GeminiConfig::new_google_ai("test-api-key-12345678901234567890");
        let provider = GeminiProvider::new(config).unwrap();

        assert!(provider.supports_model("gemini-pro"));
        assert!(provider.supports_model("gemini-1.5-flash"));
        assert!(!provider.supports_model("gpt-4"));
    }

    #[test]
    fn test_request_validation() {
        let config = GeminiConfig::new_google_ai("test-api-key-12345678901234567890");
        let provider = GeminiProvider::new(config).unwrap();

        // Empty messages should fail
        let empty_request = ChatRequest {
            model: "gemini-pro".to_string(),
            messages: vec![],
            temperature: None,
            max_tokens: None,
            max_completion_tokens: None,
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            stop: None,
            stream: false,
            tools: None,
            tool_choice: None,
            parallel_tool_calls: None,
            response_format: None,
            user: None,
            seed: None,
            n: None,
            logit_bias: None,
            functions: None,
            function_call: None,
            logprobs: None,
            top_logprobs: None,
            extra_params: std::collections::HashMap::new(),
        };

        assert!(provider.validate_request(&empty_request).is_err());

        // Invalid temperature should fail
        let invalid_temp_request = ChatRequest {
            model: "gemini-pro".to_string(),
            messages: vec![crate::core::types::requests::ChatMessage {
                role: crate::core::types::requests::MessageRole::User,
                content: Some(crate::core::types::requests::MessageContent::Text(
                    "Hello".to_string(),
                )),
                name: None,
                tool_calls: None,
                tool_call_id: None,
                function_call: None,
            }],
            temperature: Some(3.0), // Out of range
            max_tokens: None,
            max_completion_tokens: None,
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            stop: None,
            stream: false,
            tools: None,
            tool_choice: None,
            parallel_tool_calls: None,
            response_format: None,
            user: None,
            seed: None,
            n: None,
            logit_bias: None,
            functions: None,
            function_call: None,
            logprobs: None,
            top_logprobs: None,
            extra_params: std::collections::HashMap::new(),
        };

        assert!(provider.validate_request(&invalid_temp_request).is_err());
    }
}
