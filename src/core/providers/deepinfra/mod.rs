//! DeepInfra Provider
//!
//! DeepInfra model hosting platform integration

pub mod chat;
pub mod chat_handler;
pub mod rerank;

// Re-export main components
pub use chat::DeepInfraChatTransformation;
pub use rerank::DeepInfraRerankTransformation;

use async_trait::async_trait;
use futures::Stream;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use thiserror::Error;

use crate::core::providers::base_provider::{BaseHttpClient, BaseProviderConfig};
use crate::core::traits::{
    ProviderConfig, error_mapper::trait_def::ErrorMapper,
    provider::llm_provider::trait_definition::LLMProvider,
};
use crate::core::types::errors::ProviderErrorTrait;
use crate::core::types::{
    common::{HealthStatus, ModelInfo, ProviderCapability, RequestContext},
    requests::{ChatRequest, EmbeddingRequest, ImageGenerationRequest},
    responses::{ChatChunk, ChatResponse, EmbeddingResponse, ImageGenerationResponse},
};

/// DeepInfra configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepInfraConfig {
    /// API key for DeepInfra
    pub api_key: Option<String>,
    /// API base URL (default: https://api.deepinfra.com)
    pub api_base: Option<String>,
    /// Timeout in seconds
    pub timeout: u64,
    /// Max retries
    pub max_retries: u32,
}

impl Default for DeepInfraConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            api_base: Some("https://api.deepinfra.com".to_string()),
            timeout: 60,
            max_retries: 3,
        }
    }
}

impl DeepInfraConfig {
    /// Create configuration from environment variables
    pub fn from_env() -> Result<Self, DeepInfraError> {
        let api_key = std::env::var("DEEPINFRA_API_KEY").ok();

        let api_base = std::env::var("DEEPINFRA_API_BASE")
            .unwrap_or_else(|_| "https://api.deepinfra.com".to_string());

        Ok(Self {
            api_key,
            api_base: Some(api_base),
            timeout: 60,
            max_retries: 3,
        })
    }

    /// Get effective API key
    pub fn get_effective_api_key(&self) -> Option<&String> {
        self.api_key.as_ref()
    }

    /// Get effective API base URL
    pub fn get_effective_api_base(&self) -> &str {
        self.api_base
            .as_deref()
            .unwrap_or("https://api.deepinfra.com")
    }
}

/// DeepInfra error types
#[derive(Debug, Error)]
pub enum DeepInfraError {
    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("API error: {status} - {message}")]
    Api { status: u16, message: String },

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    #[error("Feature not implemented: {0}")]
    NotImplemented(String),

    #[error("Model not found: {0}")]
    ModelNotFound(String),
}

/// Implement ProviderError trait for DeepInfraError
impl ProviderErrorTrait for DeepInfraError {
    fn error_type(&self) -> &'static str {
        match self {
            Self::Configuration(_) => "configuration",
            Self::Authentication(_) => "authentication",
            Self::Network(_) => "network",
            Self::Api { .. } => "api_error",
            Self::Serialization(_) => "serialization",
            Self::Validation(_) => "validation",
            Self::RateLimit(_) => "rate_limit",
            Self::NotImplemented(_) => "not_implemented",
            Self::ModelNotFound(_) => "model_not_found",
        }
    }

    fn is_retryable(&self) -> bool {
        match self {
            Self::Network(_) => true,
            Self::RateLimit(_) => true,
            Self::Api { status, .. } if *status >= 500 => true,
            Self::Api { status, .. } if *status == 429 => true,
            _ => false,
        }
    }

    fn retry_delay(&self) -> Option<u64> {
        match self {
            Self::Network(_) => Some(5),
            Self::RateLimit(_) => Some(60),
            Self::Api { status, .. } if *status == 429 => Some(30),
            Self::Api { status, .. } if *status >= 500 => Some(10),
            _ if self.is_retryable() => Some(15),
            _ => None,
        }
    }

    fn http_status(&self) -> u16 {
        match self {
            Self::Api { status, .. } => *status,
            Self::Authentication(_) => 401,
            Self::Configuration(_) => 400,
            Self::Validation(_) => 400,
            Self::RateLimit(_) => 429,
            Self::ModelNotFound(_) => 404,
            Self::NotImplemented(_) => 501,
            _ => 500,
        }
    }

    fn not_supported(feature: &str) -> Self {
        Self::Configuration(format!("Feature not supported: {}", feature))
    }

    fn authentication_failed(reason: &str) -> Self {
        Self::Authentication(reason.to_string())
    }

    fn rate_limited(_retry_after: Option<u64>) -> Self {
        Self::RateLimit("Rate limit exceeded".to_string())
    }

    fn network_error(details: &str) -> Self {
        Self::Network(details.to_string())
    }

    fn parsing_error(details: &str) -> Self {
        Self::Serialization(details.to_string())
    }

    fn not_implemented(feature: &str) -> Self {
        Self::NotImplemented(feature.to_string())
    }
}

/// DeepInfra provider
#[derive(Debug, Clone)]
pub struct DeepInfraProvider {
    config: DeepInfraConfig,
    base_client: BaseHttpClient,
}

impl DeepInfraProvider {
    /// Create new DeepInfra provider
    pub fn new(config: DeepInfraConfig) -> Result<Self, DeepInfraError> {
        // Create base provider config
        let base_config = BaseProviderConfig {
            api_key: config.api_key.clone(),
            api_base: config.api_base.clone(),
            timeout: Some(config.timeout),
            max_retries: Some(config.max_retries),
            headers: None,
            organization: None,
            api_version: None,
        };

        // Create base HTTP client
        let base_client = BaseHttpClient::new(base_config)
            .map_err(|e| DeepInfraError::Configuration(e.to_string()))?;

        Ok(Self {
            config,
            base_client,
        })
    }

    /// Build request headers
    fn build_headers(&self) -> Result<HeaderMap, DeepInfraError> {
        let mut headers = HeaderMap::new();

        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        if let Some(api_key) = &self.config.api_key {
            let auth_value = HeaderValue::from_str(&format!("Bearer {}", api_key))
                .map_err(|e| DeepInfraError::Configuration(format!("Invalid API key: {}", e)))?;
            headers.insert(AUTHORIZATION, auth_value);
        }

        Ok(headers)
    }

    /// Make a chat completion request
    async fn send_chat_request(
        &self,
        request: ChatRequest,
        stream: bool,
    ) -> Result<reqwest::Response, DeepInfraError> {
        let url = format!(
            "{}/v1/chat/completions",
            self.config.get_effective_api_base()
        );
        let headers = self.build_headers()?;

        // Transform request to DeepInfra format
        let mut body = serde_json::json!({
            "model": request.model,
            "messages": request.messages,
            "stream": stream,
        });

        if let Some(temperature) = request.temperature {
            body["temperature"] = serde_json::json!(temperature);
        }
        if let Some(max_tokens) = request.max_tokens {
            body["max_tokens"] = serde_json::json!(max_tokens);
        }
        if let Some(top_p) = request.top_p {
            body["top_p"] = serde_json::json!(top_p);
        }

        // Send request using base HTTP client
        let response = self
            .base_client
            .inner()
            .post(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await
            .map_err(|e| DeepInfraError::Network(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(DeepInfraError::Api {
                status,
                message: error_body,
            });
        }

        Ok(response)
    }
}

/// DeepInfra error mapper
#[derive(Debug)]
pub struct DeepInfraErrorMapper;

impl ErrorMapper<DeepInfraError> for DeepInfraErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> DeepInfraError {
        match status_code {
            401 => DeepInfraError::authentication_failed(&format!(
                "Invalid API key: {}",
                response_body
            )),
            403 => DeepInfraError::authentication_failed(&format!(
                "Permission denied: {}",
                response_body
            )),
            404 => DeepInfraError::ModelNotFound(format!("Model not found: {}", response_body)),
            429 => DeepInfraError::rate_limited(None),
            500..=599 => DeepInfraError::Api {
                status: status_code,
                message: format!("Server error: {}", response_body),
            },
            _ => DeepInfraError::Api {
                status: status_code,
                message: format!("HTTP {}: {}", status_code, response_body),
            },
        }
    }
}

#[async_trait]
impl LLMProvider for DeepInfraProvider {
    type Config = DeepInfraConfig;
    type Error = DeepInfraError;
    type ErrorMapper = DeepInfraErrorMapper;

    fn name(&self) -> &'static str {
        "deepinfra"
    }

    fn capabilities(&self) -> &'static [ProviderCapability] {
        static CAPABILITIES: &[ProviderCapability] = &[
            ProviderCapability::ChatCompletion,
            ProviderCapability::ChatCompletionStream,
        ];
        CAPABILITIES
    }

    fn models(&self) -> &[ModelInfo] {
        // Return empty slice for now to avoid ModelInfo structure errors
        &[]
    }

    fn get_supported_openai_params(&self, _model: &str) -> &'static [&'static str] {
        &["temperature", "max_tokens", "top_p", "stream"]
    }

    async fn map_openai_params(
        &self,
        params: std::collections::HashMap<String, serde_json::Value>,
        _model: &str,
    ) -> Result<std::collections::HashMap<String, serde_json::Value>, Self::Error> {
        // DeepInfra API is largely OpenAI-compatible
        Ok(params)
    }

    async fn transform_request(
        &self,
        request: ChatRequest,
        _context: RequestContext,
    ) -> Result<serde_json::Value, Self::Error> {
        use serde_json::json;

        let mut body = json!({
            "model": request.model,
            "messages": request.messages,
        });

        if let Some(temperature) = request.temperature {
            body["temperature"] = json!(temperature);
        }

        if let Some(max_tokens) = request.max_tokens {
            body["max_tokens"] = json!(max_tokens);
        }

        if let Some(top_p) = request.top_p {
            body["top_p"] = json!(top_p);
        }

        if request.stream {
            body["stream"] = json!(true);
        }

        Ok(body)
    }

    async fn transform_response(
        &self,
        raw_response: &[u8],
        model: &str,
        request_id: &str,
    ) -> Result<ChatResponse, Self::Error> {
        let response_text = std::str::from_utf8(raw_response)
            .map_err(|e| DeepInfraError::Serialization(format!("Invalid UTF-8: {}", e)))?;

        let _response_json: serde_json::Value = serde_json::from_str(response_text)
            .map_err(|e| DeepInfraError::Serialization(format!("Invalid JSON: {}", e)))?;

        // Basic response structure - would need full implementation
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let chat_response = ChatResponse {
            id: request_id.to_string(),
            object: "chat.completion".to_string(),
            created: timestamp as i64,
            model: model.to_string(),
            choices: vec![], // TODO: Parse actual choices
            usage: None,     // TODO: Parse usage
            system_fingerprint: None,
        };

        Ok(chat_response)
    }

    fn get_error_mapper(&self) -> Self::ErrorMapper {
        DeepInfraErrorMapper
    }

    async fn calculate_cost(
        &self,
        model: &str,
        input_tokens: u32,
        output_tokens: u32,
    ) -> Result<f64, Self::Error> {
        // Basic cost calculation for DeepInfra models
        let cost = match model {
            "meta-llama/Llama-2-70b-chat-hf" => {
                (input_tokens as f64 * 0.0007 + output_tokens as f64 * 0.0009) / 1000.0
            }
            _ => 0.0,
        };
        Ok(cost)
    }

    fn supports_model(&self, model: &str) -> bool {
        // DeepInfra supports many open source models
        model.contains("llama")
            || model.contains("mistral")
            || model.contains("mixtral")
            || model.contains("falcon")
    }

    async fn health_check(&self) -> HealthStatus {
        if self.config.api_key.is_some() {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unhealthy
        }
    }

    async fn chat_completion(
        &self,
        request: ChatRequest,
        _context: RequestContext,
    ) -> Result<ChatResponse, Self::Error> {
        let response = self.send_chat_request(request.clone(), false).await?;

        // Parse response
        let response_json: serde_json::Value = response.json().await.map_err(|e| {
            DeepInfraError::Serialization(format!("Failed to parse response: {}", e))
        })?;

        // Transform to ChatResponse
        let chat_response: ChatResponse = serde_json::from_value(response_json).map_err(|e| {
            DeepInfraError::Serialization(format!("Failed to parse ChatResponse: {}", e))
        })?;

        Ok(chat_response)
    }

    async fn chat_completion_stream(
        &self,
        _request: ChatRequest,
        _context: RequestContext,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatChunk, Self::Error>> + Send>>, Self::Error>
    {
        // For now, return not implemented since streaming requires SSE parsing
        // which would need to be implemented separately
        Err(DeepInfraError::NotImplemented(
            "Streaming not yet implemented".to_string(),
        ))
    }

    async fn embeddings(
        &self,
        _request: EmbeddingRequest,
        _context: RequestContext,
    ) -> Result<EmbeddingResponse, Self::Error> {
        Err(DeepInfraError::not_supported("Embeddings not supported"))
    }

    async fn image_generation(
        &self,
        _request: ImageGenerationRequest,
        _context: RequestContext,
    ) -> Result<ImageGenerationResponse, Self::Error> {
        Err(DeepInfraError::not_supported(
            "Image generation not supported",
        ))
    }
}

impl ProviderConfig for DeepInfraConfig {
    fn validate(&self) -> Result<(), String> {
        if self.api_key.is_none() {
            return Err("DeepInfra API key is required".to_string());
        }
        Ok(())
    }

    fn api_key(&self) -> Option<&str> {
        self.api_key.as_deref()
    }

    fn api_base(&self) -> Option<&str> {
        self.api_base.as_deref()
    }

    fn timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.timeout)
    }

    fn max_retries(&self) -> u32 {
        self.max_retries
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::traits::error_mapper::trait_def::ErrorMapper;

    // DeepInfraConfig tests
    #[test]
    fn test_deepinfra_config_default() {
        let config = DeepInfraConfig::default();
        assert!(config.api_key.is_none());
        assert_eq!(config.api_base, Some("https://api.deepinfra.com".to_string()));
        assert_eq!(config.timeout, 60);
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_deepinfra_config_with_api_key() {
        let config = DeepInfraConfig {
            api_key: Some("test-key".to_string()),
            ..Default::default()
        };
        assert_eq!(config.get_effective_api_key(), Some(&"test-key".to_string()));
    }

    #[test]
    fn test_deepinfra_config_get_effective_api_base() {
        let config = DeepInfraConfig::default();
        assert_eq!(config.get_effective_api_base(), "https://api.deepinfra.com");

        let config_custom = DeepInfraConfig {
            api_base: Some("https://custom.api.com".to_string()),
            ..Default::default()
        };
        assert_eq!(config_custom.get_effective_api_base(), "https://custom.api.com");

        let config_none = DeepInfraConfig {
            api_base: None,
            ..Default::default()
        };
        assert_eq!(config_none.get_effective_api_base(), "https://api.deepinfra.com");
    }

    #[test]
    fn test_deepinfra_config_validate_missing_key() {
        let config = DeepInfraConfig::default();
        let result = config.validate();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "DeepInfra API key is required");
    }

    #[test]
    fn test_deepinfra_config_validate_with_key() {
        let config = DeepInfraConfig {
            api_key: Some("test-key".to_string()),
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_deepinfra_config_provider_trait() {
        let config = DeepInfraConfig {
            api_key: Some("my-key".to_string()),
            api_base: Some("https://api.example.com".to_string()),
            timeout: 120,
            max_retries: 5,
        };

        assert_eq!(config.api_key(), Some("my-key"));
        assert_eq!(config.api_base(), Some("https://api.example.com"));
        assert_eq!(config.timeout(), std::time::Duration::from_secs(120));
        assert_eq!(config.max_retries(), 5);
    }

    // DeepInfraError tests
    #[test]
    fn test_deepinfra_error_display() {
        let err = DeepInfraError::Configuration("missing config".to_string());
        assert_eq!(err.to_string(), "Configuration error: missing config");

        let err = DeepInfraError::Authentication("bad key".to_string());
        assert_eq!(err.to_string(), "Authentication error: bad key");

        let err = DeepInfraError::Network("timeout".to_string());
        assert_eq!(err.to_string(), "Network error: timeout");

        let err = DeepInfraError::Api { status: 500, message: "server error".to_string() };
        assert_eq!(err.to_string(), "API error: 500 - server error");

        let err = DeepInfraError::Serialization("parse error".to_string());
        assert_eq!(err.to_string(), "Serialization error: parse error");

        let err = DeepInfraError::Validation("invalid input".to_string());
        assert_eq!(err.to_string(), "Validation error: invalid input");

        let err = DeepInfraError::RateLimit("too many requests".to_string());
        assert_eq!(err.to_string(), "Rate limit exceeded: too many requests");

        let err = DeepInfraError::NotImplemented("streaming".to_string());
        assert_eq!(err.to_string(), "Feature not implemented: streaming");

        let err = DeepInfraError::ModelNotFound("gpt-5".to_string());
        assert_eq!(err.to_string(), "Model not found: gpt-5");
    }

    #[test]
    fn test_deepinfra_error_type() {
        assert_eq!(DeepInfraError::Configuration("".to_string()).error_type(), "configuration");
        assert_eq!(DeepInfraError::Authentication("".to_string()).error_type(), "authentication");
        assert_eq!(DeepInfraError::Network("".to_string()).error_type(), "network");
        assert_eq!(DeepInfraError::Api { status: 500, message: "".to_string() }.error_type(), "api_error");
        assert_eq!(DeepInfraError::Serialization("".to_string()).error_type(), "serialization");
        assert_eq!(DeepInfraError::Validation("".to_string()).error_type(), "validation");
        assert_eq!(DeepInfraError::RateLimit("".to_string()).error_type(), "rate_limit");
        assert_eq!(DeepInfraError::NotImplemented("".to_string()).error_type(), "not_implemented");
        assert_eq!(DeepInfraError::ModelNotFound("".to_string()).error_type(), "model_not_found");
    }

    #[test]
    fn test_deepinfra_error_is_retryable() {
        assert!(DeepInfraError::Network("".to_string()).is_retryable());
        assert!(DeepInfraError::RateLimit("".to_string()).is_retryable());
        assert!(DeepInfraError::Api { status: 500, message: "".to_string() }.is_retryable());
        assert!(DeepInfraError::Api { status: 503, message: "".to_string() }.is_retryable());
        assert!(DeepInfraError::Api { status: 429, message: "".to_string() }.is_retryable());

        assert!(!DeepInfraError::Configuration("".to_string()).is_retryable());
        assert!(!DeepInfraError::Authentication("".to_string()).is_retryable());
        assert!(!DeepInfraError::Api { status: 400, message: "".to_string() }.is_retryable());
        assert!(!DeepInfraError::Api { status: 404, message: "".to_string() }.is_retryable());
        assert!(!DeepInfraError::Validation("".to_string()).is_retryable());
    }

    #[test]
    fn test_deepinfra_error_retry_delay() {
        assert_eq!(DeepInfraError::Network("".to_string()).retry_delay(), Some(5));
        assert_eq!(DeepInfraError::RateLimit("".to_string()).retry_delay(), Some(60));
        assert_eq!(DeepInfraError::Api { status: 429, message: "".to_string() }.retry_delay(), Some(30));
        assert_eq!(DeepInfraError::Api { status: 500, message: "".to_string() }.retry_delay(), Some(10));
        assert_eq!(DeepInfraError::Api { status: 503, message: "".to_string() }.retry_delay(), Some(10));
        assert_eq!(DeepInfraError::Configuration("".to_string()).retry_delay(), None);
        assert_eq!(DeepInfraError::Authentication("".to_string()).retry_delay(), None);
    }

    #[test]
    fn test_deepinfra_error_http_status() {
        assert_eq!(DeepInfraError::Api { status: 503, message: "".to_string() }.http_status(), 503);
        assert_eq!(DeepInfraError::Authentication("".to_string()).http_status(), 401);
        assert_eq!(DeepInfraError::Configuration("".to_string()).http_status(), 400);
        assert_eq!(DeepInfraError::Validation("".to_string()).http_status(), 400);
        assert_eq!(DeepInfraError::RateLimit("".to_string()).http_status(), 429);
        assert_eq!(DeepInfraError::ModelNotFound("".to_string()).http_status(), 404);
        assert_eq!(DeepInfraError::NotImplemented("".to_string()).http_status(), 501);
        assert_eq!(DeepInfraError::Network("".to_string()).http_status(), 500);
        assert_eq!(DeepInfraError::Serialization("".to_string()).http_status(), 500);
    }

    #[test]
    fn test_deepinfra_error_factory_methods() {
        let err = DeepInfraError::not_supported("vision");
        assert!(matches!(err, DeepInfraError::Configuration(_)));

        let err = DeepInfraError::authentication_failed("bad token");
        assert!(matches!(err, DeepInfraError::Authentication(_)));

        let err = DeepInfraError::rate_limited(Some(30));
        assert!(matches!(err, DeepInfraError::RateLimit(_)));

        let err = DeepInfraError::network_error("timeout");
        assert!(matches!(err, DeepInfraError::Network(_)));

        let err = DeepInfraError::parsing_error("invalid json");
        assert!(matches!(err, DeepInfraError::Serialization(_)));

        let err = DeepInfraError::not_implemented("streaming");
        assert!(matches!(err, DeepInfraError::NotImplemented(_)));
    }

    // DeepInfraErrorMapper tests
    #[test]
    fn test_deepinfra_error_mapper_401() {
        let mapper = DeepInfraErrorMapper;
        let err = mapper.map_http_error(401, "invalid key");
        assert!(matches!(err, DeepInfraError::Authentication(_)));
    }

    #[test]
    fn test_deepinfra_error_mapper_403() {
        let mapper = DeepInfraErrorMapper;
        let err = mapper.map_http_error(403, "forbidden");
        assert!(matches!(err, DeepInfraError::Authentication(_)));
    }

    #[test]
    fn test_deepinfra_error_mapper_404() {
        let mapper = DeepInfraErrorMapper;
        let err = mapper.map_http_error(404, "not found");
        assert!(matches!(err, DeepInfraError::ModelNotFound(_)));
    }

    #[test]
    fn test_deepinfra_error_mapper_429() {
        let mapper = DeepInfraErrorMapper;
        let err = mapper.map_http_error(429, "rate limited");
        assert!(matches!(err, DeepInfraError::RateLimit(_)));
    }

    #[test]
    fn test_deepinfra_error_mapper_500() {
        let mapper = DeepInfraErrorMapper;
        let err = mapper.map_http_error(500, "server error");
        assert!(matches!(err, DeepInfraError::Api { status: 500, .. }));
    }

    #[test]
    fn test_deepinfra_error_mapper_503() {
        let mapper = DeepInfraErrorMapper;
        let err = mapper.map_http_error(503, "service unavailable");
        assert!(matches!(err, DeepInfraError::Api { status: 503, .. }));
    }

    #[test]
    fn test_deepinfra_error_mapper_unknown() {
        let mapper = DeepInfraErrorMapper;
        let err = mapper.map_http_error(418, "teapot");
        assert!(matches!(err, DeepInfraError::Api { status: 418, .. }));
    }

    // DeepInfraProvider tests
    #[test]
    fn test_deepinfra_provider_supports_model() {
        let config = DeepInfraConfig {
            api_key: Some("test".to_string()),
            ..Default::default()
        };
        let provider = DeepInfraProvider::new(config).unwrap();

        assert!(provider.supports_model("meta-llama/Llama-2-70b"));
        assert!(provider.supports_model("mistralai/Mixtral-8x7B"));
        assert!(provider.supports_model("tiiuae/falcon-40b"));
        assert!(!provider.supports_model("gpt-4"));
        assert!(!provider.supports_model("claude-3"));
    }

    #[test]
    fn test_deepinfra_provider_name() {
        let config = DeepInfraConfig {
            api_key: Some("test".to_string()),
            ..Default::default()
        };
        let provider = DeepInfraProvider::new(config).unwrap();
        assert_eq!(provider.name(), "deepinfra");
    }

    #[test]
    fn test_deepinfra_provider_capabilities() {
        let config = DeepInfraConfig {
            api_key: Some("test".to_string()),
            ..Default::default()
        };
        let provider = DeepInfraProvider::new(config).unwrap();
        let capabilities = provider.capabilities();

        assert!(capabilities.contains(&ProviderCapability::ChatCompletion));
        assert!(capabilities.contains(&ProviderCapability::ChatCompletionStream));
    }

    #[test]
    fn test_deepinfra_provider_get_supported_openai_params() {
        let config = DeepInfraConfig {
            api_key: Some("test".to_string()),
            ..Default::default()
        };
        let provider = DeepInfraProvider::new(config).unwrap();
        let params = provider.get_supported_openai_params("any-model");

        assert!(params.contains(&"temperature"));
        assert!(params.contains(&"max_tokens"));
        assert!(params.contains(&"top_p"));
        assert!(params.contains(&"stream"));
    }

    #[tokio::test]
    async fn test_deepinfra_provider_calculate_cost() {
        let config = DeepInfraConfig {
            api_key: Some("test".to_string()),
            ..Default::default()
        };
        let provider = DeepInfraProvider::new(config).unwrap();

        let cost = provider.calculate_cost("meta-llama/Llama-2-70b-chat-hf", 1000, 1000).await.unwrap();
        assert!(cost > 0.0);

        let cost_unknown = provider.calculate_cost("unknown-model", 1000, 1000).await.unwrap();
        assert_eq!(cost_unknown, 0.0);
    }

    #[tokio::test]
    async fn test_deepinfra_provider_health_check_with_key() {
        let config = DeepInfraConfig {
            api_key: Some("test".to_string()),
            ..Default::default()
        };
        let provider = DeepInfraProvider::new(config).unwrap();
        let status = provider.health_check().await;
        assert_eq!(status, HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_deepinfra_provider_health_check_without_key() {
        let config = DeepInfraConfig {
            api_key: None,
            ..Default::default()
        };
        let provider = DeepInfraProvider::new(config).unwrap();
        let status = provider.health_check().await;
        assert_eq!(status, HealthStatus::Unhealthy);
    }
}
