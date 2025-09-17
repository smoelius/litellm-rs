//! Meta Llama Provider - Consolidated Version
//!
//! A streamlined implementation combining all Llama provider functionality
//! into a single module following Rust best practices.

use async_trait::async_trait;
use futures::{Stream, StreamExt};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::fmt::Debug;
use std::pin::Pin;
use tracing::warn;

use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::error_mapper::ErrorMapper;
use crate::core::traits::provider::{LLMProvider, ProviderConfig as ProviderConfigTrait};
use crate::core::types::{
    common::{HealthStatus, ModelInfo, ProviderCapability, RequestContext},
    requests::{
        ChatMessage, ChatRequest, EmbeddingRequest, ImageGenerationRequest, MessageContent,
        MessageRole,
    },
    responses::{
        ChatChoice, ChatChunk, ChatResponse, EmbeddingResponse, FinishReason,
        ImageGenerationResponse, Usage,
    },
};

// ============================================================================
// Configuration
// ============================================================================

/// Meta Llama provider configuration
#[derive(Clone, Deserialize, Serialize)]
pub struct LlamaConfig {
    /// API key for authentication
    pub api_key: String,

    /// Base URL (defaults to https://api.llama.com/compat/v1)
    #[serde(default = "default_base_url")]
    pub base_url: String,

    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout: u64,

    /// Max retries for failed requests
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// Custom headers
    #[serde(default)]
    pub custom_headers: HashMap<String, String>,
}

fn default_base_url() -> String {
    "https://api.llama.com/compat/v1".to_string()
}

fn default_timeout() -> u64 {
    60
}

fn default_max_retries() -> u32 {
    3
}

impl Default for LlamaConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            base_url: default_base_url(),
            timeout: default_timeout(),
            max_retries: default_max_retries(),
            custom_headers: HashMap::new(),
        }
    }
}

impl ProviderConfigTrait for LlamaConfig {
    fn validate(&self) -> Result<(), String> {
        if self.api_key.is_empty() {
            return Err("API key is required".to_string());
        }
        Ok(())
    }

    fn api_key(&self) -> Option<&str> {
        Some(&self.api_key)
    }

    fn api_base(&self) -> Option<&str> {
        Some(&self.base_url)
    }

    fn timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.timeout)
    }

    fn max_retries(&self) -> u32 {
        self.max_retries
    }
}

impl Debug for LlamaConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LlamaConfig")
            .field("api_key", &"<redacted>")
            .field("base_url", &self.base_url)
            .field("timeout", &self.timeout)
            .field("max_retries", &self.max_retries)
            .finish()
    }
}

// ============================================================================
// Provider Implementation
// ============================================================================

/// Meta Llama provider implementation
#[derive(Debug)]
pub struct MetaLlamaProvider {
    config: LlamaConfig,
    client: Client,
    capabilities: Vec<ProviderCapability>,
    models: Vec<ModelInfo>,
}

impl MetaLlamaProvider {
    /// Create a new Meta Llama provider
    pub fn new(config: LlamaConfig) -> Result<Self, ProviderError> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout))
            .build()
            .map_err(|e| ProviderError::Configuration {
                provider: "meta_llama",
                message: format!("Failed to create HTTP client: {}", e),
            })?;

        let capabilities = vec![
            ProviderCapability::ChatCompletion,
            ProviderCapability::ChatCompletionStream,
            ProviderCapability::ToolCalling,
        ];

        let models = vec![
            ModelInfo {
                id: "llama-3.1-8b".to_string(),
                name: "Llama 3.1 8B".to_string(),
                provider: "meta".to_string(),
                max_context_length: 128000,
                max_output_length: None,
                supports_streaming: true,
                supports_tools: true,
                supports_multimodal: false,
                input_cost_per_1k_tokens: Some(0.0001),
                output_cost_per_1k_tokens: Some(0.0002),
                currency: "USD".to_string(),
                capabilities: vec![],
                created_at: None,
                updated_at: None,
                metadata: HashMap::new(),
            },
            ModelInfo {
                id: "llama-3.1-70b".to_string(),
                name: "Llama 3.1 70B".to_string(),
                provider: "meta".to_string(),
                max_context_length: 128000,
                max_output_length: None,
                supports_streaming: true,
                supports_tools: true,
                supports_multimodal: false,
                input_cost_per_1k_tokens: Some(0.0005),
                output_cost_per_1k_tokens: Some(0.001),
                currency: "USD".to_string(),
                capabilities: vec![],
                created_at: None,
                updated_at: None,
                metadata: HashMap::new(),
            },
            ModelInfo {
                id: "llama-3.1-405b".to_string(),
                name: "Llama 3.1 405B".to_string(),
                provider: "meta".to_string(),
                max_context_length: 128000,
                max_output_length: None,
                supports_streaming: true,
                supports_tools: true,
                supports_multimodal: false,
                input_cost_per_1k_tokens: Some(0.002),
                output_cost_per_1k_tokens: Some(0.004),
                currency: "USD".to_string(),
                capabilities: vec![],
                created_at: None,
                updated_at: None,
                metadata: HashMap::new(),
            },
            ModelInfo {
                id: "llama-3.2-1b".to_string(),
                name: "Llama 3.2 1B".to_string(),
                provider: "meta".to_string(),
                max_context_length: 128000,
                max_output_length: None,
                supports_streaming: true,
                supports_tools: true,
                supports_multimodal: false,
                input_cost_per_1k_tokens: Some(0.00005),
                output_cost_per_1k_tokens: Some(0.0001),
                currency: "USD".to_string(),
                capabilities: vec![],
                created_at: None,
                updated_at: None,
                metadata: HashMap::new(),
            },
            ModelInfo {
                id: "llama-3.2-3b".to_string(),
                name: "Llama 3.2 3B".to_string(),
                provider: "meta".to_string(),
                max_context_length: 128000,
                max_output_length: None,
                supports_streaming: true,
                supports_tools: true,
                supports_multimodal: false,
                input_cost_per_1k_tokens: Some(0.00008),
                output_cost_per_1k_tokens: Some(0.00015),
                currency: "USD".to_string(),
                capabilities: vec![],
                created_at: None,
                updated_at: None,
                metadata: HashMap::new(),
            },
        ];

        Ok(Self {
            config,
            client,
            capabilities,
            models,
        })
    }

    /// Transform request to Llama API format
    fn transform_request(&self, request: ChatRequest) -> Value {
        // Handle system messages - Llama expects them in a specific format
        let messages: Vec<Value> = request
            .messages
            .iter()
            .map(|msg| {
                let mut message = json!({
                    "role": self.transform_role(&msg.role),
                    "content": self.transform_content(&msg.content),
                });

                if let Some(name) = &msg.name {
                    message["name"] = json!(name);
                }

                if let Some(tool_calls) = &msg.tool_calls {
                    message["tool_calls"] = json!(tool_calls);
                }

                message
            })
            .collect();

        let mut body = json!({
            "model": request.model,
            "messages": messages,
            "stream": request.stream,
        });

        // Add optional parameters
        if let Some(temp) = request.temperature {
            body["temperature"] = json!(temp);
        }
        if let Some(max_tokens) = request.max_tokens {
            body["max_tokens"] = json!(max_tokens);
        }
        if let Some(top_p) = request.top_p {
            body["top_p"] = json!(top_p);
        }
        if let Some(tools) = request.tools {
            body["tools"] = json!(tools);
        }
        if let Some(tool_choice) = request.tool_choice {
            body["tool_choice"] = json!(tool_choice);
        }

        body
    }

    fn transform_role(&self, role: &MessageRole) -> String {
        match role {
            MessageRole::System => "system",
            MessageRole::User => "user",
            MessageRole::Assistant => "assistant",
            MessageRole::Tool => "tool",
            MessageRole::Function => "function",
        }
        .to_string()
    }

    fn transform_content(&self, content: &Option<MessageContent>) -> Value {
        match content {
            Some(MessageContent::Text(text)) => json!(text),
            Some(MessageContent::Parts(parts)) => json!(parts),
            None => json!(null),
        }
    }

    /// Transform Llama response to standard format
    fn transform_response(&self, llama_response: Value) -> Result<ChatResponse, ProviderError> {
        let response: LlamaResponse =
            serde_json::from_value(llama_response).map_err(|e| ProviderError::ResponseParsing {
                provider: "meta_llama",
                message: format!("Failed to parse response: {}", e),
            })?;

        Ok(ChatResponse {
            id: response.id,
            object: "chat.completion".to_string(),
            created: response.created as i64,
            model: response.model,
            choices: response
                .choices
                .into_iter()
                .map(|choice| {
                    ChatChoice {
                        index: choice.index as u32,
                        message: ChatMessage {
                            role: self.parse_role(&choice.message.role),
                            content: Some(MessageContent::Text(choice.message.content.clone())),
                            name: choice.message.name,
                            tool_calls: None, // TODO: Convert Value to ToolCall
                            tool_call_id: None,
                            function_call: None,
                        },
                        finish_reason: self.parse_finish_reason(&choice.finish_reason),
                        logprobs: None,
                    }
                })
                .collect(),
            usage: Some(Usage {
                prompt_tokens: response.usage.prompt_tokens,
                completion_tokens: response.usage.completion_tokens,
                total_tokens: response.usage.total_tokens,
                prompt_tokens_details: None,
                completion_tokens_details: None,
            }),
            system_fingerprint: response.system_fingerprint,
        })
    }

    fn parse_role(&self, role: &str) -> MessageRole {
        match role {
            "system" => MessageRole::System,
            "user" => MessageRole::User,
            "assistant" => MessageRole::Assistant,
            "tool" => MessageRole::Tool,
            "function" => MessageRole::Function,
            _ => MessageRole::User,
        }
    }

    fn parse_finish_reason(&self, reason: &Option<String>) -> Option<FinishReason> {
        reason.as_ref().and_then(|r| match r.as_str() {
            "stop" => Some(FinishReason::Stop),
            "length" => Some(FinishReason::Length),
            "tool_calls" => Some(FinishReason::ToolCalls),
            "content_filter" => Some(FinishReason::ContentFilter),
            _ => None,
        })
    }

    /// Execute HTTP request with retries
    async fn execute_request(&self, endpoint: &str, body: Value) -> Result<Value, ProviderError> {
        let url = format!("{}/{}", self.config.base_url, endpoint);

        let mut retries = 0;
        loop {
            let response = self
                .client
                .post(&url)
                .header("Authorization", format!("Bearer {}", self.config.api_key))
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await
                .map_err(|e| ProviderError::Network {
                    provider: "meta_llama",
                    message: format!("Request failed: {}", e),
                })?;

            if response.status().is_success() {
                return response
                    .json()
                    .await
                    .map_err(|e| ProviderError::ResponseParsing {
                        provider: "meta_llama",
                        message: format!("Failed to parse response: {}", e),
                    });
            }

            // Handle rate limiting and retries
            if response.status() == 429 && retries < self.config.max_retries {
                retries += 1;
                let delay = std::time::Duration::from_secs(2_u64.pow(retries));
                warn!("Rate limited, retrying in {:?}", delay);
                tokio::time::sleep(delay).await;
                continue;
            }

            // Handle other errors
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();

            return Err(match status.as_u16() {
                401 => ProviderError::Authentication {
                    provider: "meta_llama",
                    message: "Invalid API key".to_string(),
                },
                402 => ProviderError::QuotaExceeded {
                    provider: "meta_llama",
                    message: "Quota exceeded".to_string(),
                },
                429 => ProviderError::RateLimit {
                    provider: "meta_llama",
                    message: "Rate limit exceeded".to_string(),
                    retry_after: None,
                    rpm_limit: None,
                    tpm_limit: None,
                    current_usage: None,
                },
                _ => ProviderError::Other {
                    provider: "meta_llama",
                    message: format!("Request failed with status {}: {}", status, error_text),
                },
            });
        }
    }
}

// ============================================================================
// Error Mapper
// ============================================================================

pub struct LlamaErrorMapper;

impl ErrorMapper<ProviderError> for LlamaErrorMapper {
    fn map_http_error(&self, status: u16, body: &str) -> ProviderError {
        match status {
            401 => ProviderError::Authentication {
                provider: "meta_llama",
                message: body.to_string(),
            },
            402 => ProviderError::QuotaExceeded {
                provider: "meta_llama",
                message: body.to_string(),
            },
            429 => ProviderError::RateLimit {
                provider: "meta_llama",
                message: body.to_string(),
                retry_after: None,
                rpm_limit: None,
                tpm_limit: None,
                current_usage: None,
            },
            404 => ProviderError::ModelNotFound {
                provider: "meta_llama",
                model: body.to_string(),
            },
            _ => ProviderError::Other {
                provider: "meta_llama",
                message: format!("HTTP {}: {}", status, body),
            },
        }
    }
}

// ============================================================================
// LLMProvider Trait Implementation
// ============================================================================

#[async_trait]
impl LLMProvider for MetaLlamaProvider {
    type Config = LlamaConfig;
    type Error = ProviderError;
    type ErrorMapper = LlamaErrorMapper;

    fn name(&self) -> &'static str {
        "meta_llama"
    }

    fn capabilities(&self) -> &'static [ProviderCapability] {
        &[
            ProviderCapability::ChatCompletion,
            ProviderCapability::ChatCompletionStream,
            ProviderCapability::ToolCalling,
        ]
    }

    fn models(&self) -> &[ModelInfo] {
        &self.models
    }

    fn get_supported_openai_params(&self, _model: &str) -> &'static [&'static str] {
        &[
            "temperature",
            "max_tokens",
            "top_p",
            "frequency_penalty",
            "presence_penalty",
            "stop",
            "tools",
            "tool_choice",
            "response_format",
            "seed",
            "user",
        ]
    }

    async fn map_openai_params(
        &self,
        params: HashMap<String, Value>,
        _model: &str,
    ) -> Result<HashMap<String, Value>, Self::Error> {
        Ok(params) // Llama uses OpenAI-compatible format
    }

    async fn transform_request(
        &self,
        request: ChatRequest,
        _context: RequestContext,
    ) -> Result<Value, Self::Error> {
        Ok(self.transform_request(request))
    }

    async fn transform_response(
        &self,
        raw_response: &[u8],
        _model: &str,
        _request_id: &str,
    ) -> Result<ChatResponse, Self::Error> {
        let value: Value =
            serde_json::from_slice(raw_response).map_err(|e| ProviderError::ResponseParsing {
                provider: "meta_llama",
                message: format!("Failed to parse response: {}", e),
            })?;
        self.transform_response(value)
    }

    fn get_error_mapper(&self) -> Self::ErrorMapper {
        LlamaErrorMapper
    }

    async fn health_check(&self) -> HealthStatus {
        // Try to list models to check health
        let response = self
            .client
            .get(format!("{}/models", self.config.base_url))
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .send()
            .await;

        match response {
            Ok(res) if res.status().is_success() => HealthStatus::Healthy,
            Ok(res) if res.status() == 401 => HealthStatus::Unhealthy,
            _ => HealthStatus::Unhealthy,
        }
    }

    async fn chat_completion(
        &self,
        request: ChatRequest,
        _context: RequestContext,
    ) -> Result<ChatResponse, ProviderError> {
        let body = self.transform_request(request);
        let response = self.execute_request("chat/completions", body).await?;
        self.transform_response(response)
    }

    async fn chat_completion_stream(
        &self,
        request: ChatRequest,
        _context: RequestContext,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatChunk, ProviderError>> + Send>>, ProviderError>
    {
        let mut modified_request = request;
        modified_request.stream = true;

        let body = self.transform_request(modified_request);
        let url = format!("{}/chat/completions", self.config.base_url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::Network {
                provider: "meta_llama",
                message: format!("Stream request failed: {}", e),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(ProviderError::Other {
                provider: "meta_llama",
                message: format!("Stream failed with status {}: {}", status, error_text),
            });
        }

        // Create stream from response
        let stream = response.bytes_stream().map(move |chunk| {
            match chunk {
                Ok(bytes) => {
                    // Parse SSE data
                    let data = String::from_utf8_lossy(&bytes);
                    if let Some(json_str) = data.strip_prefix("data: ") {
                        if json_str.trim() == "[DONE]" {
                            return Ok(ChatChunk {
                                id: String::new(),
                                object: "chat.completion.chunk".to_string(),
                                created: 0,
                                model: String::new(),
                                choices: vec![],
                                usage: None,
                                system_fingerprint: None,
                            });
                        }

                        match serde_json::from_str::<ChatChunk>(json_str) {
                            Ok(chunk) => Ok(chunk),
                            Err(e) => Err(ProviderError::ResponseParsing {
                                provider: "meta_llama",
                                message: format!("Failed to parse chunk: {}", e),
                            }),
                        }
                    } else {
                        Ok(ChatChunk {
                            id: String::new(),
                            object: "chat.completion.chunk".to_string(),
                            created: 0,
                            model: String::new(),
                            choices: vec![],
                            usage: None,
                            system_fingerprint: None,
                        })
                    }
                }
                Err(e) => Err(ProviderError::Network {
                    provider: "meta_llama",
                    message: format!("Stream error: {}", e),
                }),
            }
        });

        Ok(Box::pin(stream))
    }

    async fn embeddings(
        &self,
        _request: EmbeddingRequest,
        _context: RequestContext,
    ) -> Result<EmbeddingResponse, Self::Error> {
        Err(ProviderError::NotImplemented {
            provider: "meta_llama",
            feature: "embeddings".to_string(),
        })
    }

    async fn image_generation(
        &self,
        _request: ImageGenerationRequest,
        _context: RequestContext,
    ) -> Result<ImageGenerationResponse, Self::Error> {
        Err(ProviderError::NotImplemented {
            provider: "meta_llama",
            feature: "image_generation".to_string(),
        })
    }

    async fn calculate_cost(
        &self,
        model: &str,
        input_tokens: u32,
        output_tokens: u32,
    ) -> Result<f64, Self::Error> {
        // Simple cost calculation - adjust rates as needed
        let (input_rate, output_rate) = match model {
            "llama-3.1-8b" => (0.0001, 0.0002),
            "llama-3.1-70b" => (0.0005, 0.001),
            "llama-3.1-405b" => (0.002, 0.004),
            _ => (0.0001, 0.0002),
        };

        let cost = (input_tokens as f64 / 1000.0) * input_rate
            + (output_tokens as f64 / 1000.0) * output_rate;
        Ok(cost)
    }
}

// ============================================================================
// Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
struct LlamaResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<LlamaChoice>,
    usage: LlamaUsage,
    system_fingerprint: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LlamaChoice {
    index: i32,
    message: LlamaMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LlamaMessage {
    role: String,
    content: String,
    name: Option<String>,
    tool_calls: Option<Vec<Value>>,
}

#[derive(Debug, Deserialize)]
struct LlamaUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = LlamaConfig::default();
        assert_eq!(config.base_url, "https://api.llama.com/compat/v1");
        assert_eq!(config.timeout, 60);
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_transform_role() {
        let config = LlamaConfig::default();
        let provider = MetaLlamaProvider::new(config).unwrap();

        assert_eq!(provider.transform_role(&MessageRole::System), "system");
        assert_eq!(provider.transform_role(&MessageRole::User), "user");
        assert_eq!(
            provider.transform_role(&MessageRole::Assistant),
            "assistant"
        );
    }

    #[tokio::test]
    async fn test_provider_creation() {
        let config = LlamaConfig {
            api_key: "test_key".to_string(),
            ..Default::default()
        };

        let provider = MetaLlamaProvider::new(config);
        assert!(provider.is_ok());

        let provider = provider.unwrap();
        assert_eq!(provider.name(), "meta_llama");
        assert_eq!(provider.capabilities().len(), 3);
    }
}
