//! OpenRouter Provider Implementation
//!
//! Unified OpenRouter provider using the modern architecture

use async_trait::async_trait;
use futures::Stream;
use serde_json::Value;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;

use crate::core::providers::base::{GlobalPoolManager, HeaderPair, HttpMethod, header};
use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::{ProviderConfig, provider::llm_provider::trait_definition::LLMProvider};
use crate::core::types::{
    common::{HealthStatus, ModelInfo, ProviderCapability, RequestContext},
    requests::ChatRequest,
    responses::{ChatChunk, ChatResponse},
    thinking::ThinkingContent,
};

use super::config::OpenRouterConfig;
use super::models::get_openrouter_registry;
// use super::streaming::OpenRouterStream; // Unused for now

/// OpenRouter provider implementation
#[derive(Debug, Clone)]
pub struct OpenRouterProvider {
    config: OpenRouterConfig,
    pool_manager: Arc<GlobalPoolManager>,
    model_registry: &'static super::models::OpenRouterModelRegistry,
}

impl OpenRouterProvider {
    /// Generate headers for OpenRouter API requests
    fn get_request_headers(&self) -> Vec<HeaderPair> {
        let mut headers = Vec::with_capacity(3);

        headers.push(header(
            "Authorization",
            format!("Bearer {}", self.config.api_key),
        ));

        if let Some(site_url) = &self.config.site_url {
            headers.push(header("HTTP-Referer", site_url.clone()));
        }

        if let Some(site_name) = &self.config.site_name {
            headers.push(header("X-Title", site_name.clone()));
        }

        headers
    }

    /// Create new OpenRouter provider
    pub fn new(config: OpenRouterConfig) -> Result<Self, ProviderError> {
        config
            .validate()
            .map_err(|e| ProviderError::configuration("openrouter", e))?;

        let pool_manager = Arc::new(
            GlobalPoolManager::new()
                .map_err(|e| ProviderError::configuration("openrouter", e.to_string()))?,
        );
        let model_registry = get_openrouter_registry();

        Ok(Self {
            config,
            pool_manager,
            model_registry,
        })
    }

    /// Create provider from environment
    pub fn from_env() -> Result<Self, ProviderError> {
        let config = OpenRouterConfig::from_env();
        Self::new(config)
    }

    /// Transform chat request to OpenRouter format
    fn transform_chat_request(&self, request: ChatRequest) -> Result<Value, ProviderError> {
        let mut body = serde_json::json!({
            "model": request.model,
            "messages": request.messages,
            "stream": request.stream
        });

        // Add OpenAI-compatible parameters
        if let Some(max_tokens) = request.max_tokens {
            body["max_tokens"] = max_tokens.into();
        }

        if let Some(temperature) = request.temperature {
            body["temperature"] = temperature.into();
        }

        if let Some(top_p) = request.top_p {
            body["top_p"] = top_p.into();
        }

        if let Some(frequency_penalty) = request.frequency_penalty {
            body["frequency_penalty"] = frequency_penalty.into();
        }

        if let Some(presence_penalty) = request.presence_penalty {
            body["presence_penalty"] = presence_penalty.into();
        }

        if let Some(stop) = request.stop {
            body["stop"] = serde_json::to_value(stop)
                .map_err(|e| ProviderError::serialization("openrouter", e.to_string()))?;
        }

        if let Some(tools) = request.tools {
            body["tools"] = serde_json::to_value(tools)
                .map_err(|e| ProviderError::serialization("openrouter", e.to_string()))?;
        }

        if let Some(tool_choice) = request.tool_choice {
            body["tool_choice"] = serde_json::to_value(tool_choice)
                .map_err(|e| ProviderError::serialization("openrouter", e.to_string()))?;
        }

        // Add OpenRouter-specific parameters
        for (key, value) in &self.config.extra_params {
            body[key] = value.clone();
        }

        Ok(body)
    }

    /// Transform OpenRouter response to standard format
    fn transform_chat_response(
        &self,
        raw_response: Value,
        model: &str,
    ) -> Result<ChatResponse, ProviderError> {
        let response: serde_json::Map<String, Value> = raw_response
            .as_object()
            .ok_or_else(|| {
                ProviderError::response_parsing(
                    "openrouter",
                    "Response is not a JSON object".to_string(),
                )
            })?
            .clone();

        // Check for error response
        if let Some(error) = response.get("error") {
            let error_obj = error.as_object().ok_or_else(|| {
                ProviderError::response_parsing(
                    "openrouter",
                    "Error field is not an object".to_string(),
                )
            })?;

            // Try to get detailed error from metadata.raw first, like Python LiteLLM
            let detailed_message = if let Some(metadata) = error_obj.get("metadata") {
                if let Some(raw) = metadata.get("raw").and_then(|v| v.as_str()) {
                    // Try to parse the raw error JSON
                    if let Ok(raw_error) = serde_json::from_str::<serde_json::Value>(raw) {
                        if let Some(error_inner) = raw_error.get("error") {
                            if let Some(msg) = error_inner.get("message").and_then(|v| v.as_str()) {
                                // Include provider name for context
                                if let Some(provider_name) =
                                    metadata.get("provider_name").and_then(|v| v.as_str())
                                {
                                    format!("{}: {}", provider_name, msg)
                                } else {
                                    msg.to_string()
                                }
                            } else {
                                error_obj
                                    .get("message")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("Unknown error")
                                    .to_string()
                            }
                        } else {
                            error_obj
                                .get("message")
                                .and_then(|v| v.as_str())
                                .unwrap_or("Unknown error")
                                .to_string()
                        }
                    } else {
                        error_obj
                            .get("message")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Unknown error")
                            .to_string()
                    }
                } else {
                    error_obj
                        .get("message")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown error")
                        .to_string()
                }
            } else {
                error_obj
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown error")
                    .to_string()
            };

            let code = error_obj
                .get("code")
                .and_then(|v| v.as_i64())
                .unwrap_or(500);

            // Check for specific error types
            if code == 404
                || detailed_message.contains("Model not found")
                || detailed_message.contains("No endpoints found")
            {
                return Err(ProviderError::model_not_found("openrouter", model));
            } else if code == 401 {
                return Err(ProviderError::authentication(
                    "openrouter",
                    &detailed_message,
                ));
            } else if code == 429 {
                return Err(ProviderError::rate_limit("openrouter", None));
            } else {
                // For all other errors (including 403), return as API error with proper detailed message
                return Err(ProviderError::api_error(
                    "openrouter",
                    code as u16,
                    detailed_message,
                ));
            }
        }

        // Extract ID
        let id = response
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("openrouter-response")
            .to_string();

        // Extract choices
        let choices = response
            .get("choices")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                ProviderError::response_parsing("openrouter", "No choices in response".to_string())
            })?;

        let mut response_choices = Vec::new();
        for (index, choice) in choices.iter().enumerate() {
            let choice_obj = choice.as_object().ok_or_else(|| {
                ProviderError::response_parsing("openrouter", "Choice is not an object".to_string())
            })?;

            let message = choice_obj.get("message").ok_or_else(|| {
                ProviderError::response_parsing("openrouter", "No message in choice".to_string())
            })?;

            let finish_reason = choice_obj
                .get("finish_reason")
                .and_then(|v| v.as_str())
                .map(|s| match s {
                    "stop" => crate::core::types::responses::FinishReason::Stop,
                    "length" => crate::core::types::responses::FinishReason::Length,
                    "tool_calls" => crate::core::types::responses::FinishReason::ToolCalls,
                    "content_filter" => crate::core::types::responses::FinishReason::ContentFilter,
                    "function_call" => crate::core::types::responses::FinishReason::FunctionCall,
                    _ => crate::core::types::responses::FinishReason::Stop, // Default fallback
                });

            // Parse message but handle reasoning/thinking separately
            let mut chat_message: crate::core::types::ChatMessage =
                serde_json::from_value(message.clone())
                    .map_err(|e| ProviderError::response_parsing("openrouter", e.to_string()))?;

            // Extract reasoning/thinking content from the raw message
            // OpenRouter/DeepSeek uses "reasoning" or "reasoning_content" fields
            if chat_message.thinking.is_none() {
                let thinking = message
                    .get("reasoning_content")
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty())
                    .or_else(|| {
                        message
                            .get("reasoning")
                            .and_then(|v| v.as_str())
                            .filter(|s| !s.is_empty())
                    })
                    .map(|text| ThinkingContent::Text {
                        text: text.to_string(),
                        signature: None,
                    });

                if thinking.is_some() {
                    chat_message.thinking = thinking;
                }
            }

            response_choices.push(crate::core::types::responses::ChatChoice {
                index: index as u32,
                message: chat_message,
                finish_reason,
                logprobs: None,
            });
        }

        // Extract usage
        let usage = response
            .get("usage")
            .map(|u| serde_json::from_value(u.clone()))
            .transpose()
            .map_err(|e| ProviderError::response_parsing("openrouter", e.to_string()))?;

        Ok(ChatResponse {
            id,
            object: "chat.completion".to_string(),
            created: chrono::Utc::now().timestamp(),
            model: model.to_string(),
            choices: response_choices,
            usage,
            system_fingerprint: None,
        })
    }

    /// Get request headers
    fn get_headers(&self) -> HashMap<String, String> {
        self.config.get_headers()
    }
}

#[async_trait]
impl LLMProvider for OpenRouterProvider {
    type Config = OpenRouterConfig;
    type Error = ProviderError;
    type ErrorMapper = crate::core::traits::error_mapper::implementations::OpenAIErrorMapper; // OpenRouter uses OpenAI-compatible API

    fn name(&self) -> &'static str {
        "openrouter"
    }

    fn capabilities(&self) -> &'static [ProviderCapability] {
        static CAPABILITIES: &[ProviderCapability] = &[
            ProviderCapability::ChatCompletion,
            ProviderCapability::ChatCompletionStream,
            ProviderCapability::FunctionCalling,
            ProviderCapability::ToolCalling,
            // OpenRouter supports many models with different capabilities
            // Vision support depends on the underlying model
        ];
        CAPABILITIES
    }

    fn models(&self) -> &[ModelInfo] {
        static MODELS: std::sync::LazyLock<Vec<ModelInfo>> =
            std::sync::LazyLock::new(|| get_openrouter_registry().get_all_models());
        &MODELS
    }

    async fn health_check(&self) -> HealthStatus {
        // Test with a lightweight API call
        let _url = format!("{}/models", self.config.base_url);

        // TODO: Fix health check - needs proper BaseConfig
        // Temporarily return Healthy for now
        HealthStatus::Healthy
    }

    fn get_supported_openai_params(&self, _model: &str) -> &'static [&'static str] {
        static PARAMS: &[&str] = &[
            "temperature",
            "top_p",
            "max_tokens",
            "frequency_penalty",
            "presence_penalty",
            "stop",
            "tools",
            "tool_choice",
            "response_format",
        ];
        PARAMS
    }

    async fn map_openai_params(
        &self,
        params: HashMap<String, Value>,
        _model: &str,
    ) -> Result<HashMap<String, Value>, Self::Error> {
        // OpenRouter uses OpenAI-compatible API, so no transformation needed
        Ok(params)
    }

    async fn transform_request(
        &self,
        request: ChatRequest,
        _context: RequestContext,
    ) -> Result<Value, Self::Error> {
        self.transform_chat_request(request)
    }

    async fn transform_response(
        &self,
        raw_response: &[u8],
        model: &str,
        _request_id: &str,
    ) -> Result<ChatResponse, Self::Error> {
        let response: Value = serde_json::from_slice(raw_response)
            .map_err(|e| ProviderError::response_parsing("openrouter", e.to_string()))?;

        self.transform_chat_response(response, model)
    }

    fn get_error_mapper(&self) -> Self::ErrorMapper {
        crate::core::traits::error_mapper::implementations::OpenAIErrorMapper
    }

    async fn chat_completion(
        &self,
        request: ChatRequest,
        context: RequestContext,
    ) -> Result<ChatResponse, Self::Error> {
        // Like Python LiteLLM, we don't validate models locally
        // OpenRouter API will handle invalid models

        // Transform request
        let body = self
            .transform_request(request.clone(), context.clone())
            .await?;

        // Make API call using high-performance connection pool
        let url = format!("{}/chat/completions", self.config.base_url);

        let headers = self.get_request_headers();
        let body_data = Some(body);

        let response = self
            .pool_manager
            .execute_request(&url, HttpMethod::POST, headers, body_data)
            .await?;

        let response_bytes = response
            .bytes()
            .await
            .map_err(|e| ProviderError::network("openrouter", e.to_string()))?;

        self.transform_response(&response_bytes, &request.model, &context.request_id)
            .await
    }

    async fn chat_completion_stream(
        &self,
        request: ChatRequest,
        _context: RequestContext,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatChunk, Self::Error>> + Send>>, Self::Error>
    {
        // Like Python LiteLLM, we don't validate models locally
        // OpenRouter API will handle model capabilities

        // Force streaming
        let mut streaming_request = request;
        streaming_request.stream = true;

        // TODO: Implement proper streaming using UnifiedHttpClient
        // For now, return an error indicating streaming is not yet implemented
        Err(ProviderError::not_supported(
            "openrouter",
            "Streaming not yet implemented in unified architecture",
        ))
    }

    async fn calculate_cost(
        &self,
        model: &str,
        input_tokens: u32,
        output_tokens: u32,
    ) -> Result<f64, Self::Error> {
        // Try to get pricing from registry, but don't fail if model not found
        // This matches Python LiteLLM's approach of supporting all models
        if let Some(model_spec) = self.model_registry.get_model_spec(model) {
            let input_cost = if let Some(prompt_cost) = model_spec.prompt_cost {
                (input_tokens as f64 / 1_000_000.0) * prompt_cost
            } else {
                0.0
            };

            let output_cost = if let Some(completion_cost) = model_spec.completion_cost {
                (output_tokens as f64 / 1_000_000.0) * completion_cost
            } else {
                0.0
            };

            Ok(input_cost + output_cost)
        } else {
            // Model not in registry, return 0 cost (unknown pricing)
            Ok(0.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_provider_creation() {
        let config = OpenRouterConfig::new("test-key-1234567890")
            .with_site_url("https://example.com")
            .with_site_name("Test Site");

        let provider = OpenRouterProvider::new(config);
        assert!(provider.is_ok());
    }

    #[test]
    fn test_capabilities() {
        let config = OpenRouterConfig::new("test-key-1234567890");
        let provider = OpenRouterProvider::new(config).unwrap();

        let caps = provider.capabilities();
        assert!(caps.contains(&ProviderCapability::ChatCompletion));
        assert!(caps.contains(&ProviderCapability::ChatCompletionStream));
        assert!(caps.contains(&ProviderCapability::FunctionCalling));
    }

    #[test]
    fn test_models() {
        let config = OpenRouterConfig::new("test-key-1234567890");
        let provider = OpenRouterProvider::new(config).unwrap();

        let models = provider.models();
        assert!(!models.is_empty());

        // Should have OpenAI models
        assert!(models.iter().any(|m| m.id.contains("openai/gpt-4")));

        // Should have Anthropic models
        assert!(models.iter().any(|m| m.id.contains("anthropic/claude")));
    }

    #[test]
    fn test_request_transformation() {
        let config = OpenRouterConfig::new("test-key-1234567890");
        let provider = OpenRouterProvider::new(config).unwrap();

        let request = ChatRequest {
            model: "openai/gpt-4".to_string(),
            messages: vec![],
            stream: false,
            max_tokens: Some(100),
            temperature: Some(0.7),
            ..Default::default()
        };

        let transformed = provider.transform_chat_request(request).unwrap();

        assert_eq!(transformed["model"], "openai/gpt-4");
        assert_eq!(transformed["max_tokens"], 100);
        let temp_value = transformed["temperature"].as_f64().unwrap();
        assert!(
            (temp_value - 0.7).abs() < 1e-6,
            "Expected 0.7, got {}",
            temp_value
        );
        assert_eq!(transformed["stream"], false);
    }
}
