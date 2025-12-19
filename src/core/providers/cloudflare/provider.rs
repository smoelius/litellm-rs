//! Main Cloudflare Workers AI Provider Implementation
//!
//! Implements the LLMProvider trait for Cloudflare's Workers AI models.

use async_trait::async_trait;
use futures::Stream;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use tracing::debug;

use super::config::CloudflareConfig;
use super::error::{CloudflareError, CloudflareErrorMapper};
use super::model_info::{calculate_cost, get_available_models, get_model_info};
use crate::core::providers::base::{GlobalPoolManager, HttpMethod};
use crate::core::traits::{ProviderConfig as _, provider::llm_provider::trait_definition::LLMProvider};
use crate::core::types::{
    common::{HealthStatus, ModelInfo, ProviderCapability, RequestContext},
    requests::{ChatRequest, EmbeddingRequest},
    responses::{ChatChunk, ChatResponse, EmbeddingResponse, FinishReason},
};

/// Static capabilities for Cloudflare provider
const CLOUDFLARE_CAPABILITIES: &[ProviderCapability] = &[
    ProviderCapability::ChatCompletion,
    ProviderCapability::ChatCompletionStream,
];

/// Cloudflare Workers AI provider implementation
#[derive(Debug, Clone)]
pub struct CloudflareProvider {
    config: CloudflareConfig,
    pool_manager: Arc<GlobalPoolManager>,
    models: Vec<ModelInfo>,
}

impl CloudflareProvider {
    /// Create a new Cloudflare provider instance
    pub async fn new(config: CloudflareConfig) -> Result<Self, CloudflareError> {
        // Validate configuration
        config
            .validate()
            .map_err(CloudflareError::ConfigurationError)?;

        // Create pool manager
        let pool_manager = Arc::new(GlobalPoolManager::new().map_err(|e| {
            CloudflareError::ConfigurationError(format!("Failed to create pool manager: {}", e))
        })?);

        // Build model list from static configuration
        let models = get_available_models()
            .iter()
            .filter_map(|id| get_model_info(id))
            .map(|info| {
                let capabilities = vec![
                    ProviderCapability::ChatCompletion,
                    ProviderCapability::ChatCompletionStream,
                ];

                ModelInfo {
                    id: format!("cloudflare/{}", info.model_id),
                    name: info.display_name.to_string(),
                    provider: "cloudflare".to_string(),
                    max_context_length: info.context_length,
                    max_output_length: Some(info.max_output_tokens),
                    supports_streaming: info.supports_streaming,
                    supports_tools: info.supports_tools,
                    supports_multimodal: info.supports_vision,
                    input_cost_per_1k_tokens: Some(info.input_cost_per_million / 1000.0),
                    output_cost_per_1k_tokens: Some(info.output_cost_per_million / 1000.0),
                    currency: "USD".to_string(),
                    capabilities,
                    created_at: None,
                    updated_at: None,
                    metadata: HashMap::new(),
                }
            })
            .collect();

        Ok(Self {
            config,
            pool_manager,
            models,
        })
    }

    /// Create provider with account ID and token
    pub async fn with_credentials(
        account_id: impl Into<String>,
        api_token: impl Into<String>,
    ) -> Result<Self, CloudflareError> {
        let config = CloudflareConfig {
            account_id: Some(account_id.into()),
            api_token: Some(api_token.into()),
            ..Default::default()
        };
        Self::new(config).await
    }

    /// Execute an HTTP request
    async fn execute_request(
        &self,
        endpoint: &str,
        body: serde_json::Value,
    ) -> Result<serde_json::Value, CloudflareError> {
        let account_id = self.config.get_account_id().ok_or_else(|| {
            CloudflareError::ConfigurationError("Account ID is required".to_string())
        })?;

        let url = format!(
            "{}/accounts/{}/ai/run/{}",
            self.config.get_api_base(),
            account_id,
            endpoint
        );

        let mut headers = Vec::new();
        if let Some(api_token) = self.config.get_api_token() {
            headers.push(("Authorization".to_string(), format!("Bearer {}", api_token)));
        }
        headers.push(("Content-Type".to_string(), "application/json".to_string()));

        let response = self
            .pool_manager
            .execute_request(&url, HttpMethod::POST, headers, Some(body))
            .await
            .map_err(|e| CloudflareError::NetworkError(e.to_string()))?;

        let response_bytes = response
            .bytes()
            .await
            .map_err(|e| CloudflareError::NetworkError(e.to_string()))?;

        serde_json::from_slice(&response_bytes)
            .map_err(|e| CloudflareError::ApiError(format!("Failed to parse response: {}", e)))
    }

    /// Transform OpenAI-style request to Cloudflare format
    fn transform_to_cloudflare_format(&self, request: &ChatRequest) -> serde_json::Value {
        // Cloudflare uses a simpler format
        let mut messages = Vec::new();
        for msg in &request.messages {
            let mut message = serde_json::json!({
                "role": msg.role.to_string().to_lowercase(),
            });

            if let Some(ref content) = msg.content {
                message["content"] = serde_json::json!(content.to_string());
            }

            messages.push(message);
        }

        let mut body = serde_json::json!({
            "messages": messages,
        });

        // Add optional parameters
        if let Some(temperature) = request.temperature {
            body["temperature"] = serde_json::json!(temperature);
        }

        if let Some(max_tokens) = request.max_tokens {
            body["max_tokens"] = serde_json::json!(max_tokens);
        }

        if let Some(top_p) = request.top_p {
            body["top_p"] = serde_json::json!(top_p);
        }

        if request.stream {
            body["stream"] = serde_json::json!(true);
        }

        body
    }
}

#[async_trait]
impl LLMProvider for CloudflareProvider {
    type Config = CloudflareConfig;
    type Error = CloudflareError;
    type ErrorMapper = CloudflareErrorMapper;

    fn name(&self) -> &'static str {
        "cloudflare"
    }

    fn capabilities(&self) -> &'static [ProviderCapability] {
        CLOUDFLARE_CAPABILITIES
    }

    fn models(&self) -> &[ModelInfo] {
        &self.models
    }

    fn get_supported_openai_params(&self, _model: &str) -> &'static [&'static str] {
        &[
            "temperature",
            "top_p",
            "max_tokens",
            "stream",
            "stop",
            "frequency_penalty",
            "presence_penalty",
            "n",
            "seed",
        ]
    }

    async fn map_openai_params(
        &self,
        params: HashMap<String, serde_json::Value>,
        _model: &str,
    ) -> Result<HashMap<String, serde_json::Value>, Self::Error> {
        // Cloudflare supports a subset of OpenAI parameters directly
        Ok(params)
    }

    async fn transform_request(
        &self,
        request: ChatRequest,
        _context: RequestContext,
    ) -> Result<serde_json::Value, Self::Error> {
        Ok(self.transform_to_cloudflare_format(&request))
    }

    async fn transform_response(
        &self,
        raw_response: &[u8],
        model: &str,
        request_id: &str,
    ) -> Result<ChatResponse, Self::Error> {
        let cloudflare_response: serde_json::Value = serde_json::from_slice(raw_response)
            .map_err(|e| CloudflareError::ApiError(format!("Failed to parse response: {}", e)))?;

        // Transform Cloudflare response to OpenAI format
        let content = cloudflare_response["result"]["response"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(ChatResponse {
            id: request_id.to_string(),
            object: "chat.completion".to_string(),
            created: chrono::Utc::now().timestamp(),
            model: model.to_string(),
            choices: vec![crate::core::types::responses::ChatChoice {
                index: 0,
                message: crate::core::types::requests::ChatMessage {
                    role: crate::core::types::requests::MessageRole::Assistant,
                    content: Some(crate::core::types::requests::MessageContent::Text(content)),
                thinking: None,
                    name: None,
                    function_call: None,
                    tool_calls: None,
                    tool_call_id: None,
                },
                finish_reason: Some(FinishReason::Stop),
                logprobs: None,
            }],
            usage: None, // Cloudflare doesn't provide usage stats
            system_fingerprint: None,
        })
    }

    fn get_error_mapper(&self) -> Self::ErrorMapper {
        CloudflareErrorMapper
    }

    async fn chat_completion(
        &self,
        request: ChatRequest,
        _context: RequestContext,
    ) -> Result<ChatResponse, Self::Error> {
        debug!("Cloudflare chat request: model={}", request.model);

        // Remove cloudflare/ prefix if present
        let model = request
            .model
            .strip_prefix("cloudflare/")
            .unwrap_or(&request.model);

        // Transform request
        let cloudflare_request = self.transform_to_cloudflare_format(&request);

        // Execute request
        let response = self.execute_request(model, cloudflare_request).await?;

        // Transform response
        let content = response["result"]["response"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let request_id = uuid::Uuid::new_v4().to_string();

        Ok(ChatResponse {
            id: request_id.clone(),
            object: "chat.completion".to_string(),
            created: chrono::Utc::now().timestamp(),
            model: request.model.clone(),
            choices: vec![crate::core::types::responses::ChatChoice {
                index: 0,
                message: crate::core::types::requests::ChatMessage {
                    role: crate::core::types::requests::MessageRole::Assistant,
                    content: Some(crate::core::types::requests::MessageContent::Text(content)),
                thinking: None,
                    name: None,
                    function_call: None,
                    tool_calls: None,
                    tool_call_id: None,
                },
                finish_reason: Some(FinishReason::Stop),
                logprobs: None,
            }],
            usage: None,
            system_fingerprint: None,
        })
    }

    async fn chat_completion_stream(
        &self,
        mut request: ChatRequest,
        _context: RequestContext,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatChunk, Self::Error>> + Send>>, Self::Error>
    {
        debug!("Cloudflare streaming request: model={}", request.model);

        // Set streaming flag
        request.stream = true;

        // TODO: Implement proper SSE streaming for Cloudflare
        // For now, return an error as streaming implementation needs more work
        Err(CloudflareError::InvalidRequestError(
            "Streaming is not yet fully implemented for Cloudflare provider".to_string(),
        ))
    }

    async fn embeddings(
        &self,
        _request: EmbeddingRequest,
        _context: RequestContext,
    ) -> Result<EmbeddingResponse, Self::Error> {
        // Cloudflare also supports embeddings models, but we'll implement text generation first
        Err(CloudflareError::InvalidRequestError(
            "Embeddings are not yet implemented for Cloudflare provider".to_string(),
        ))
    }

    async fn health_check(&self) -> HealthStatus {
        // Simple health check - try to list models
        let account_id = match self.config.get_account_id() {
            Some(id) => id,
            None => return HealthStatus::Unhealthy,
        };

        let url = format!(
            "{}/accounts/{}/ai/models/search",
            self.config.get_api_base(),
            account_id
        );

        let mut headers = Vec::new();
        if let Some(api_token) = self.config.get_api_token() {
            headers.push(("Authorization".to_string(), format!("Bearer {}", api_token)));
        }

        match self
            .pool_manager
            .execute_request(&url, HttpMethod::GET, headers, None::<serde_json::Value>)
            .await
        {
            Ok(_) => HealthStatus::Healthy,
            Err(_) => HealthStatus::Unhealthy,
        }
    }

    async fn calculate_cost(
        &self,
        model: &str,
        input_tokens: u32,
        output_tokens: u32,
    ) -> Result<f64, Self::Error> {
        calculate_cost(model, input_tokens, output_tokens)
            .ok_or_else(|| CloudflareError::ModelNotFoundError(format!("Unknown model: {}", model)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_provider_creation() {
        let config = CloudflareConfig {
            account_id: Some("test_account".to_string()),
            api_token: Some("test_token".to_string()),
            ..Default::default()
        };

        let provider = CloudflareProvider::new(config).await;
        assert!(provider.is_ok());

        let provider = provider.unwrap();
        assert_eq!(provider.name(), "cloudflare");
        assert!(!provider.models().is_empty());
    }

    #[tokio::test]
    async fn test_provider_without_credentials() {
        let config = CloudflareConfig {
            account_id: None,
            api_token: None,
            ..Default::default()
        };

        let provider = CloudflareProvider::new(config).await;
        assert!(provider.is_err());
    }

    #[test]
    fn test_capabilities() {
        assert!(CLOUDFLARE_CAPABILITIES.contains(&ProviderCapability::ChatCompletion));
        assert!(CLOUDFLARE_CAPABILITIES.contains(&ProviderCapability::ChatCompletionStream));
    }

    #[test]
    fn test_transform_request() {
        use crate::core::types::requests::{ChatMessage, MessageContent, MessageRole};

        let config = CloudflareConfig {
            account_id: Some("test".to_string()),
            api_token: Some("test".to_string()),
            ..Default::default()
        };

        let runtime = tokio::runtime::Runtime::new().unwrap();
        let provider = runtime.block_on(CloudflareProvider::new(config)).unwrap();

        let request = ChatRequest {
            model: "@cf/meta/llama-3-8b-instruct".to_string(),
            messages: vec![ChatMessage {
                role: MessageRole::User,
                content: Some(MessageContent::Text("Hello".to_string())),
                thinking: None,
                name: None,
                function_call: None,
                tool_calls: None,
                tool_call_id: None,
            thinking: None,
            }],
            temperature: Some(0.7),
            max_tokens: Some(100),
            stream: false,
            ..Default::default()
        };

        let transformed = provider.transform_to_cloudflare_format(&request);
        assert!(transformed["messages"].is_array());
        let temp_value = transformed["temperature"].as_f64().unwrap();
        assert!(
            (temp_value - 0.7).abs() < 1e-6,
            "Expected 0.7, got {}",
            temp_value
        );
        assert_eq!(transformed["max_tokens"], 100);
    }
}
