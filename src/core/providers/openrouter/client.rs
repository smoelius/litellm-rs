//! Implementation

use async_trait::async_trait;
use futures::Stream;
use reqwest::Client;
use std::pin::Pin;
use std::time::Duration;
use tracing::{warn, error, debug};

use crate::core::traits::{ErrorMapper, ProviderConfig, provider::LLMProvider};
use crate::core::types::{
    common::{HealthStatus, ModelInfo, ProviderCapability, RequestContext},
    requests::{ChatRequest, EmbeddingRequest, ImageGenerationRequest},
    responses::{ChatChunk, ChatResponse, EmbeddingResponse, ImageGenerationResponse},
};

use super::config::OpenRouterConfig;
use super::error::OpenRouterError;
use super::models::get_openrouter_registry;

use serde_json::Value;
use std::collections::HashMap;

/// OpenRouter-specific error mapper implementation
#[derive(Debug)]
pub struct OpenRouterErrorMapper;

impl ErrorMapper<OpenRouterError> for OpenRouterErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> OpenRouterError {
        match status_code {
            400 => OpenRouterError::InvalidRequest(format!("Bad request: {}", response_body)),
            401 => OpenRouterError::Authentication("Invalid API key".to_string()),
            403 => {
                OpenRouterError::Authentication("Forbidden: insufficient permissions".to_string())
            }
            404 => OpenRouterError::UnsupportedModel("Model not found".to_string()),
            429 => OpenRouterError::RateLimit("Rate limit exceeded".to_string()),
            500 => OpenRouterError::ApiError {
                status_code: 500,
                message: "Internal server error".to_string(),
            },
            502 => OpenRouterError::Network("Bad gateway".to_string()),
            503 => OpenRouterError::Network("Service unavailable".to_string()),
            _ => OpenRouterError::ApiError {
                status_code,
                message: format!("HTTP error {}: {}", status_code, response_body),
            },
        }
    }

    fn map_json_error(&self, error_response: &Value) -> OpenRouterError {
        if let Some(error) = error_response.get("error") {
            let error_code = error.get("code").and_then(|c| c.as_i64()).unwrap_or(0);
            let error_message = error
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error");
            let error_type = error
                .get("type")
                .and_then(|t| t.as_str())
                .unwrap_or("unknown");

            match error_type {
                "invalid_request_error" => {
                    OpenRouterError::InvalidRequest(error_message.to_string())
                }
                "authentication_error" => {
                    OpenRouterError::Authentication("Authentication failed".to_string())
                }
                "permission_error" => {
                    OpenRouterError::Authentication("Permission denied".to_string())
                }
                "rate_limit_error" => OpenRouterError::RateLimit("Rate limit exceeded".to_string()),
                "api_error" => OpenRouterError::ApiError {
                    status_code: error_code as u16,
                    message: error_message.to_string(),
                },
                _ => OpenRouterError::Other(format!("{}: {}", error_type, error_message)),
            }
        } else {
            OpenRouterError::Parsing("Unknown error response format".to_string())
        }
    }

    fn map_network_error(&self, error: &dyn std::error::Error) -> OpenRouterError {
        OpenRouterError::Network(format!("Network error: {}", error))
    }
}
use super::transformer::{
    OpenRouterRequestTransformer, OpenRouterResponseTransformer, create_openrouter_headers,
};

/// OpenRouter Provider implementation
#[derive(Debug, Clone)]
pub struct OpenRouterProvider {
    /// HTTP客户端
    client: Client,
    /// Configuration
    config: OpenRouterConfig,
    /// API基础URL
    base_url: String,
    /// Model
    models: Vec<ModelInfo>,
}

impl OpenRouterProvider {
    /// Create
    pub async fn new(config: OpenRouterConfig) -> Result<Self, OpenRouterError> {
        // Configuration
        config
            .validate()
            .map_err(OpenRouterError::Configuration)?;

        // Get
        let api_key = if config.api_key.is_empty() {
            std::env::var("OPENROUTER_API_KEY").map_err(|_| {
                OpenRouterError::Configuration("OpenRouter API key not found".to_string())
            })?
        } else {
            config.api_key.clone()
        };
        let api_key = api_key.trim().to_string();

        // Create
        let headers = create_openrouter_headers(
            &api_key,
            config.site_url.as_deref(),
            config.site_name.as_deref(),
        );

        // Create
        let mut header_map = reqwest::header::HeaderMap::new();
        for (key, value) in &headers {
            let header_name =
                reqwest::header::HeaderName::from_bytes(key.as_bytes()).map_err(|e| {
                    OpenRouterError::Configuration(format!("Invalid header key '{}': {}", key, e))
                })?;

            // 确保 header value 没有非法字符
            let clean_value = value.trim();
            let header_value =
                reqwest::header::HeaderValue::from_str(clean_value).map_err(|e| {
                    error!(
                        provider = "openrouter",
                        header_key = %key,
                        header_value = %clean_value,
                        error = %e,
                        "Failed to parse HTTP header value"
                    );
                    OpenRouterError::Configuration(format!(
                        "Invalid header value for '{}': {}",
                        key, e
                    ))
                })?;
            header_map.insert(header_name, header_value);
        }

        // 添加自定义头 - OpenRouterConfig doesn't have headers field, so skip this
        // TODO: Add support for custom headers if needed
        /*
        for (key, value) in &config.headers {
            let header_name =
                reqwest::header::HeaderName::from_bytes(key.as_bytes()).map_err(|e| {
                    OpenRouterError::Configuration(format!(
                        "Invalid custom header key '{}': {}",
                        key, e
                    ))
                })?;
            let header_value = reqwest::header::HeaderValue::from_str(value).map_err(|e| {
                OpenRouterError::Configuration(format!(
                    "Invalid custom header value for '{}': {}",
                    key, e
                ))
            })?;
            header_map.insert(header_name, header_value);
        }
        */

        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .default_headers(header_map)
            .build()
            .map_err(|e| {
                OpenRouterError::Network(format!("Failed to create HTTP client: {}", e))
            })?;

        let base_url = config.base_url.clone();

        // Get
        let models = get_openrouter_registry().get_all_models();

        Ok(Self {
            client,
            config,
            base_url,
            models,
        })
    }

    /// Default
    pub async fn with_api_key(api_key: impl Into<String>) -> Result<Self, OpenRouterError> {
        let config = OpenRouterConfig {
            api_key: api_key.into(),
            ..Default::default()
        };
        Self::new(config).await
    }

    /// Request
    async fn execute_request<T: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
        body: serde_json::Value,
    ) -> Result<T, OpenRouterError> {
        let url = format!("{}/{}", self.base_url, endpoint);

        let response = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    OpenRouterError::Timeout(format!("Request to {} timed out", url))
                } else if e.is_connect() {
                    OpenRouterError::Network(format!("Connection failed to {}: {}", url, e))
                } else {
                    OpenRouterError::Network(format!("Request failed: {}", e))
                }
            })?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(OpenRouterResponseTransformer::parse_error(
                &error_text,
                status.as_u16(),
            ));
        }

        let response_text = response
            .text()
            .await
            .map_err(|e| OpenRouterError::Network(format!("Failed to read response: {}", e)))?;

        debug!(
            provider = "openrouter",
            response_text = %response_text,
            "Raw HTTP response received"
        );

        serde_json::from_str(&response_text)
            .map_err(|e| OpenRouterError::Parsing(format!("Failed to parse response: {}", e)))
    }
}

#[async_trait]
impl LLMProvider for OpenRouterProvider {
    type Config = OpenRouterConfig;
    type Error = OpenRouterError;
    type ErrorMapper = OpenRouterErrorMapper;

    fn name(&self) -> &'static str {
        "openrouter"
    }

    fn capabilities(&self) -> &'static [ProviderCapability] {
        static CAPABILITIES: &[ProviderCapability] = &[
            ProviderCapability::ChatCompletion,
            ProviderCapability::ChatCompletionStream,
            ProviderCapability::FunctionCalling,
        ];
        CAPABILITIES
    }

    fn models(&self) -> &[ModelInfo] {
        &self.models
    }

    async fn health_check(&self) -> HealthStatus {
        // Check
        match self
            .execute_request::<serde_json::Value>("models", serde_json::json!({}))
            .await
        {
            Ok(_) => HealthStatus::Healthy,
            Err(_) => HealthStatus::Unhealthy,
        }
    }

    async fn chat_completion(
        &self,
        request: ChatRequest,
        _context: RequestContext,
    ) -> Result<ChatResponse, Self::Error> {
        // Request
        // TODO: Convert HashMap extra_params to OpenRouterExtraParams
        let openai_request = OpenRouterRequestTransformer::transform_request(
            request, None, // Using None for now - will implement proper conversion later
        )?;

        // Request
        let body = serde_json::to_value(openai_request)?;
        debug!(
            provider = "openrouter",
            request_body = %serde_json::to_string_pretty(&body).unwrap_or_default(),
            "Sending request to OpenRouter API"
        );

        // Request
        let response: crate::core::providers::openai::models::OpenAIChatResponse =
            self.execute_request("chat/completions", body).await?;

        // Debug log the raw response
        debug!(
            provider = "openrouter",
            response = ?response,
            "Raw response received from OpenRouter"
        );

        // Response
        OpenRouterResponseTransformer::transform_response(response)
    }

    async fn chat_completion_stream(
        &self,
        _request: ChatRequest,
        _context: RequestContext,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatChunk, Self::Error>> + Send>>, Self::Error>
    {
        // Response
        // Handle
        Err(OpenRouterError::UnsupportedFeature(
            "Streaming not yet implemented".to_string(),
        ))
    }

    async fn embeddings(
        &self,
        _request: EmbeddingRequest,
        _context: RequestContext,
    ) -> Result<EmbeddingResponse, Self::Error> {
        // OpenRouter可能不支持所有模型的embeddings
        Err(OpenRouterError::UnsupportedFeature(
            "Embeddings not supported via OpenRouter".to_string(),
        ))
    }

    async fn image_generation(
        &self,
        _request: ImageGenerationRequest,
        _context: RequestContext,
    ) -> Result<ImageGenerationResponse, Self::Error> {
        // OpenRouter可能支持某些图像生成模型
        Err(OpenRouterError::UnsupportedFeature(
            "Image generation not yet implemented".to_string(),
        ))
    }

    async fn calculate_cost(
        &self,
        _model: &str,
        input_tokens: u32,
        output_tokens: u32,
    ) -> Result<f64, Self::Error> {
        // Default
        // Get
        let input_cost = (input_tokens as f64 / 1000.0) * 0.001;
        let output_cost = (output_tokens as f64 / 1000.0) * 0.002;
        Ok(input_cost + output_cost)
    }

    fn get_supported_openai_params(&self, _model: &str) -> &'static [&'static str] {
        // OpenRouter支持标准的OpenAIparameter，加上自己的扩展
        static SUPPORTED_PARAMS: &[&str] = &[
            "messages",
            "model",
            "max_tokens",
            "temperature",
            "top_p",
            "n",
            "stream",
            "stop",
            "presence_penalty",
            "frequency_penalty",
            "logit_bias",
            "user",
            "functions",
            "function_call",
            "tools",
            "tool_choice",
            "response_format",
            // OpenRouter specific_params
            "transforms",
            "models",
            "route",
            "provider",
        ];
        SUPPORTED_PARAMS
    }

    async fn map_openai_params(
        &self,
        params: HashMap<String, serde_json::Value>,
        _model: &str,
    ) -> Result<HashMap<String, serde_json::Value>, Self::Error> {
        let mut mapped_params = HashMap::new();

        for (key, value) in params {
            match key.as_str() {
                // 标准OpenAIparameter直接映射
                "messages" | "model" | "max_tokens" | "temperature" | "top_p" | "n" | "stream"
                | "stop" | "presence_penalty" | "frequency_penalty" | "logit_bias" | "user"
                | "functions" | "function_call" | "tools" | "tool_choice" | "response_format" => {
                    mapped_params.insert(key, value);
                }

                // OpenRouterspecific_params
                "transforms" | "models" | "route" | "provider" => {
                    mapped_params.insert(key, value);
                }

                // 忽略不支持的parameter
                _ => {
                    warn!(
                        provider = "openrouter",
                        parameter = %key,
                        "Ignoring unsupported parameter for OpenRouter"
                    );
                }
            }
        }

        Ok(mapped_params)
    }

    async fn transform_request(
        &self,
        request: ChatRequest,
        _context: RequestContext,
    ) -> Result<serde_json::Value, Self::Error> {
        // Request
        // TODO: Convert HashMap extra_params to OpenRouterExtraParams
        let openai_request = OpenRouterRequestTransformer::transform_request(
            request, None, // Using None for now - will implement proper conversion later
        )?;

        // 序列化为JSON值
        serde_json::to_value(openai_request)
            .map_err(|e| OpenRouterError::Parsing(format!("Failed to serialize request: {}", e)))
    }

    async fn transform_response(
        &self,
        raw_response: &[u8],
        _model: &str,
        _request_id: &str,
    ) -> Result<ChatResponse, Self::Error> {
        // Response
        let response_text = std::str::from_utf8(raw_response)
            .map_err(|e| OpenRouterError::Parsing(format!("Invalid UTF-8 response: {}", e)))?;

        let openai_response: crate::core::providers::openai::models::OpenAIChatResponse =
            serde_json::from_str(response_text).map_err(|e| {
                OpenRouterError::Parsing(format!("Failed to parse response: {}", e))
            })?;

        // Response
        OpenRouterResponseTransformer::transform_response(openai_response)
    }

    fn get_error_mapper(&self) -> Self::ErrorMapper {
        OpenRouterErrorMapper
    }
}

// Provider trait implementation removed - OpenRouterProvider is now included through the Provider enum variants
