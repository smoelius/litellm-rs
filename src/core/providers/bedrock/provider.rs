//! Main Bedrock Provider Implementation
//!
//! Contains the BedrockProvider struct and its LLMProvider trait implementation.

use async_trait::async_trait;
use futures::Stream;
use serde_json::Value;
use std::collections::HashMap;
use std::pin::Pin;
use tracing::debug;

use super::client::BedrockClient;
use super::config::BedrockConfig;
use super::error::{BedrockError, BedrockErrorMapper};
use super::model_config::{BedrockModelFamily, get_model_config};
use super::utils::{CostCalculator, validate_region};
use crate::core::traits::ProviderConfig as _;

use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::provider::LLMProvider;
use crate::core::types::{
    ChatMessage, FinishReason, MessageContent, MessageRole,
    common::{HealthStatus, ModelInfo, ProviderCapability, RequestContext},
    requests::{ChatRequest, EmbeddingRequest},
    responses::{ChatChunk, ChatResponse, EmbeddingResponse},
};

/// Static capabilities for Bedrock provider
const BEDROCK_CAPABILITIES: &[ProviderCapability] = &[
    ProviderCapability::ChatCompletion,
    ProviderCapability::ChatCompletionStream,
    ProviderCapability::FunctionCalling,
    ProviderCapability::Embeddings,
];

/// AWS Bedrock provider implementation
#[derive(Debug)]
pub struct BedrockProvider {
    client: BedrockClient,
    models: Vec<ModelInfo>,
}

impl BedrockProvider {
    /// Create a new Bedrock provider instance
    pub async fn new(config: BedrockConfig) -> Result<Self, BedrockError> {
        // Validate configuration
        config
            .validate()
            .map_err(|e| ProviderError::configuration("bedrock", e))?;

        // Validate AWS region
        validate_region(&config.aws_region)?;

        // Create Bedrock client
        let client = BedrockClient::new(config)?;

        // Define supported models using cost calculator data
        let mut models = Vec::new();
        let available_models = CostCalculator::get_all_models();

        for model_id in available_models {
            if let Some(pricing) = CostCalculator::get_model_pricing(model_id) {
                if let Ok(model_config) = get_model_config(model_id) {
                    models.push(ModelInfo {
                        id: model_id.to_string(),
                        name: format!(
                            "{} (Bedrock)",
                            model_id.split('.').next_back().unwrap_or(model_id)
                        ),
                        provider: "bedrock".to_string(),
                        max_context_length: model_config.max_context_length,
                        max_output_length: model_config.max_output_length,
                        supports_streaming: model_config.supports_streaming,
                        supports_tools: model_config.supports_function_calling,
                        supports_multimodal: model_config.supports_multimodal,
                        input_cost_per_1k_tokens: Some(pricing.input_cost_per_1k),
                        output_cost_per_1k_tokens: Some(pricing.output_cost_per_1k),
                        currency: pricing.currency.to_string(),
                        capabilities: vec![],
                        created_at: None,
                        updated_at: None,
                        metadata: HashMap::new(),
                    });
                }
            }
        }

        Ok(Self { client, models })
    }

    /// Generate images using Bedrock image models
    pub async fn generate_image(
        &self,
        request: &crate::core::types::requests::ImageGenerationRequest,
    ) -> Result<crate::core::types::responses::ImageGenerationResponse, BedrockError> {
        super::images::execute_image_generation(&self.client, request).await
    }

    /// Access the Agents client
    pub fn agents_client(&self) -> super::agents::AgentClient<'_> {
        super::agents::AgentClient::new(&self.client)
    }

    /// Access the Knowledge Bases client
    pub fn knowledge_bases_client(&self) -> super::knowledge_bases::KnowledgeBaseClient<'_> {
        super::knowledge_bases::KnowledgeBaseClient::new(&self.client)
    }

    /// Access the Batch processing client
    pub fn batch_client(&self) -> super::batch::BatchClient<'_> {
        super::batch::BatchClient::new(&self.client)
    }

    /// Access the Guardrails client
    pub fn guardrails_client(&self) -> super::guardrails::GuardrailClient<'_> {
        super::guardrails::GuardrailClient::new(&self.client)
    }

    /// Check if model is an embedding model
    fn is_embedding_model(&self, model: &str) -> bool {
        model.contains("embed")
    }

    /// Convert messages to a single prompt string for models that require it
    fn messages_to_prompt(&self, messages: &[ChatMessage]) -> Result<String, BedrockError> {
        let mut prompt = String::new();

        for message in messages {
            let content = match &message.content {
                Some(MessageContent::Text(text)) => text.clone(),
                Some(MessageContent::Parts(parts)) => {
                    // Extract text from parts
                    parts
                        .iter()
                        .filter_map(|part| {
                            if let crate::core::types::requests::ContentPart::Text { text } = part {
                                Some(text.clone())
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(" ")
                }
                None => continue,
            };

            match message.role {
                MessageRole::System => prompt.push_str(&format!("System: {}\n\n", content)),
                MessageRole::User => prompt.push_str(&format!("Human: {}\n\n", content)),
                MessageRole::Assistant => prompt.push_str(&format!("Assistant: {}\n\n", content)),
                MessageRole::Function | MessageRole::Tool => {
                    prompt.push_str(&format!("Tool: {}\n\n", content));
                }
            }
        }

        // Add Assistant prompt at the end for completion
        prompt.push_str("Assistant:");

        Ok(prompt)
    }
}

#[async_trait]
impl LLMProvider for BedrockProvider {
    type Config = BedrockConfig;
    type Error = BedrockError;
    type ErrorMapper = BedrockErrorMapper;

    fn name(&self) -> &'static str {
        "bedrock"
    }

    fn capabilities(&self) -> &'static [ProviderCapability] {
        BEDROCK_CAPABILITIES
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
            "tools",
            "tool_choice",
        ]
    }

    async fn map_openai_params(
        &self,
        params: HashMap<String, Value>,
        _model: &str,
    ) -> Result<HashMap<String, Value>, Self::Error> {
        // Bedrock has some differences from OpenAI format
        let mut mapped = HashMap::new();

        for (key, value) in params {
            match key.as_str() {
                // Map OpenAI parameters to Bedrock format
                "max_tokens" => mapped.insert("max_tokens_to_sample".to_string(), value),
                "temperature" | "top_p" | "stream" | "stop" => mapped.insert(key, value),
                // Skip unsupported parameters
                _ => None,
            };
        }

        Ok(mapped)
    }

    async fn transform_request(
        &self,
        request: ChatRequest,
        _context: RequestContext,
    ) -> Result<Value, Self::Error> {
        // Get model configuration
        let model_config = get_model_config(&request.model)?;

        // Route based on model family
        match model_config.family {
            BedrockModelFamily::Claude => {
                // Claude models on Bedrock use anthropic messages format
                let mut body = serde_json::json!({
                    "messages": request.messages,
                    "max_tokens": request.max_tokens.unwrap_or(4096),
                    "anthropic_version": "bedrock-2023-05-20"
                });

                if let Some(temp) = request.temperature {
                    body["temperature"] =
                        Value::Number(serde_json::Number::from_f64(temp.into()).unwrap());
                }

                if let Some(top_p) = request.top_p {
                    body["top_p"] =
                        Value::Number(serde_json::Number::from_f64(top_p.into()).unwrap());
                }

                Ok(body)
            }
            BedrockModelFamily::TitanText => {
                // Titan models use different format
                let prompt = self.messages_to_prompt(&request.messages)?;
                let mut body = serde_json::json!({
                    "inputText": prompt,
                    "textGenerationConfig": {
                        "maxTokenCount": request.max_tokens.unwrap_or(4096),
                    }
                });

                if let Some(temp) = request.temperature {
                    body["textGenerationConfig"]["temperature"] =
                        Value::Number(serde_json::Number::from_f64(temp.into()).unwrap());
                }

                if let Some(top_p) = request.top_p {
                    body["textGenerationConfig"]["topP"] =
                        Value::Number(serde_json::Number::from_f64(top_p.into()).unwrap());
                }

                Ok(body)
            }
            BedrockModelFamily::Nova => {
                // Nova models use converse API format similar to Claude
                let mut body = serde_json::json!({
                    "messages": request.messages,
                    "max_tokens": request.max_tokens.unwrap_or(4096),
                });

                if let Some(temp) = request.temperature {
                    body["temperature"] =
                        Value::Number(serde_json::Number::from_f64(temp.into()).unwrap());
                }

                Ok(body)
            }
            BedrockModelFamily::Llama => {
                // Meta Llama models use similar format to Claude
                let mut body = serde_json::json!({
                    "messages": request.messages,
                    "max_tokens": request.max_tokens.unwrap_or(4096),
                });

                if let Some(temp) = request.temperature {
                    body["temperature"] =
                        Value::Number(serde_json::Number::from_f64(temp.into()).unwrap());
                }

                Ok(body)
            }
            BedrockModelFamily::Mistral => {
                // Mistral models use their own format
                let prompt = self.messages_to_prompt(&request.messages)?;
                let mut body = serde_json::json!({
                    "prompt": prompt,
                    "max_tokens": request.max_tokens.unwrap_or(4096),
                });

                if let Some(temp) = request.temperature {
                    body["temperature"] =
                        Value::Number(serde_json::Number::from_f64(temp.into()).unwrap());
                }

                Ok(body)
            }
            BedrockModelFamily::AI21 => {
                // AI21 models use their own format
                let prompt = self.messages_to_prompt(&request.messages)?;
                let mut body = serde_json::json!({
                    "prompt": prompt,
                    "maxTokens": request.max_tokens.unwrap_or(4096),
                });

                if let Some(temp) = request.temperature {
                    body["temperature"] =
                        Value::Number(serde_json::Number::from_f64(temp.into()).unwrap());
                }

                Ok(body)
            }
            BedrockModelFamily::Cohere => {
                // Cohere models use their own format
                let prompt = self.messages_to_prompt(&request.messages)?;
                let mut body = serde_json::json!({
                    "prompt": prompt,
                    "max_tokens": request.max_tokens.unwrap_or(4096),
                });

                if let Some(temp) = request.temperature {
                    body["temperature"] =
                        Value::Number(serde_json::Number::from_f64(temp.into()).unwrap());
                }

                Ok(body)
            }
            BedrockModelFamily::DeepSeek => {
                // DeepSeek models use their own format
                let prompt = self.messages_to_prompt(&request.messages)?;
                let mut body = serde_json::json!({
                    "prompt": prompt,
                    "max_tokens": request.max_tokens.unwrap_or(4096),
                });

                if let Some(temp) = request.temperature {
                    body["temperature"] =
                        Value::Number(serde_json::Number::from_f64(temp.into()).unwrap());
                }

                Ok(body)
            }
            BedrockModelFamily::TitanEmbedding
            | BedrockModelFamily::TitanImage
            | BedrockModelFamily::StabilityAI => {
                // These are not chat models
                Err(ProviderError::invalid_request(
                    "bedrock",
                    format!(
                        "Model family {:?} is not supported for chat completion",
                        model_config.family
                    ),
                ))
            }
        }
    }

    async fn transform_response(
        &self,
        raw_response: &[u8],
        model: &str,
        _request_id: &str,
    ) -> Result<ChatResponse, Self::Error> {
        use crate::core::types::responses::{ChatChoice, Usage};

        let response: Value = serde_json::from_slice(raw_response)
            .map_err(|e| ProviderError::response_parsing("bedrock", e.to_string()))?;

        // Get model configuration
        let model_config = get_model_config(model)?;

        let choices = match model_config.family {
            BedrockModelFamily::Claude => {
                // Claude response format
                let content = response
                    .get("content")
                    .and_then(|c| c.as_array())
                    .and_then(|arr| arr.first())
                    .and_then(|item| item.get("text"))
                    .and_then(|text| text.as_str())
                    .unwrap_or("")
                    .to_string();

                vec![ChatChoice {
                    index: 0,
                    message: ChatMessage {
                        role: MessageRole::Assistant,
                        content: Some(MessageContent::Text(content)),
                        name: None,
                        function_call: None,
                        tool_calls: None,
                        tool_call_id: None,
                    },
                    finish_reason: Some(FinishReason::Stop),
                    logprobs: None,
                }]
            }
            BedrockModelFamily::TitanText => {
                // Titan response format
                let content = response
                    .get("results")
                    .and_then(|r| r.as_array())
                    .and_then(|arr| arr.first())
                    .and_then(|item| item.get("outputText"))
                    .and_then(|text| text.as_str())
                    .unwrap_or("")
                    .to_string();

                vec![ChatChoice {
                    index: 0,
                    message: ChatMessage {
                        role: MessageRole::Assistant,
                        content: Some(MessageContent::Text(content)),
                        name: None,
                        function_call: None,
                        tool_calls: None,
                        tool_call_id: None,
                    },
                    finish_reason: Some(FinishReason::Stop),
                    logprobs: None,
                }]
            }
            BedrockModelFamily::Nova | BedrockModelFamily::Llama => {
                // Nova and Llama use similar format to Claude
                let content = response
                    .get("content")
                    .and_then(|c| c.as_array())
                    .and_then(|arr| arr.first())
                    .and_then(|item| item.get("text"))
                    .and_then(|text| text.as_str())
                    .unwrap_or("")
                    .to_string();

                vec![ChatChoice {
                    index: 0,
                    message: ChatMessage {
                        role: MessageRole::Assistant,
                        content: Some(MessageContent::Text(content)),
                        name: None,
                        function_call: None,
                        tool_calls: None,
                        tool_call_id: None,
                    },
                    finish_reason: Some(FinishReason::Stop),
                    logprobs: None,
                }]
            }
            BedrockModelFamily::Mistral => {
                // Mistral response format
                let content = response
                    .get("outputs")
                    .and_then(|o| o.as_array())
                    .and_then(|arr| arr.first())
                    .and_then(|item| item.get("text"))
                    .and_then(|text| text.as_str())
                    .unwrap_or("")
                    .to_string();

                vec![ChatChoice {
                    index: 0,
                    message: ChatMessage {
                        role: MessageRole::Assistant,
                        content: Some(MessageContent::Text(content)),
                        name: None,
                        function_call: None,
                        tool_calls: None,
                        tool_call_id: None,
                    },
                    finish_reason: Some(FinishReason::Stop),
                    logprobs: None,
                }]
            }
            BedrockModelFamily::AI21 => {
                // AI21 response format
                let content = response
                    .get("completions")
                    .and_then(|c| c.as_array())
                    .and_then(|arr| arr.first())
                    .and_then(|item| item.get("data"))
                    .and_then(|data| data.get("text"))
                    .and_then(|text| text.as_str())
                    .unwrap_or("")
                    .to_string();

                vec![ChatChoice {
                    index: 0,
                    message: ChatMessage {
                        role: MessageRole::Assistant,
                        content: Some(MessageContent::Text(content)),
                        name: None,
                        function_call: None,
                        tool_calls: None,
                        tool_call_id: None,
                    },
                    finish_reason: Some(FinishReason::Stop),
                    logprobs: None,
                }]
            }
            BedrockModelFamily::Cohere => {
                // Cohere response format
                let content = response
                    .get("text")
                    .and_then(|text| text.as_str())
                    .unwrap_or("")
                    .to_string();

                vec![ChatChoice {
                    index: 0,
                    message: ChatMessage {
                        role: MessageRole::Assistant,
                        content: Some(MessageContent::Text(content)),
                        name: None,
                        function_call: None,
                        tool_calls: None,
                        tool_call_id: None,
                    },
                    finish_reason: Some(FinishReason::Stop),
                    logprobs: None,
                }]
            }
            BedrockModelFamily::DeepSeek => {
                // DeepSeek response format
                let content = response
                    .get("completion")
                    .and_then(|text| text.as_str())
                    .unwrap_or("")
                    .to_string();

                vec![ChatChoice {
                    index: 0,
                    message: ChatMessage {
                        role: MessageRole::Assistant,
                        content: Some(MessageContent::Text(content)),
                        name: None,
                        function_call: None,
                        tool_calls: None,
                        tool_call_id: None,
                    },
                    finish_reason: Some(FinishReason::Stop),
                    logprobs: None,
                }]
            }
            _ => {
                // Unsupported model family
                return Err(ProviderError::invalid_request(
                    "bedrock",
                    format!(
                        "Model family {:?} is not supported for response parsing",
                        model_config.family
                    ),
                ));
            }
        };

        // Extract usage information based on model family
        let usage = match model_config.family {
            BedrockModelFamily::Claude | BedrockModelFamily::Nova | BedrockModelFamily::Llama => {
                response.get("usage").map(|u| Usage {
                    prompt_tokens: u.get("input_tokens").and_then(|t| t.as_u64()).unwrap_or(0)
                        as u32,
                    completion_tokens: u.get("output_tokens").and_then(|t| t.as_u64()).unwrap_or(0)
                        as u32,
                    total_tokens: 0, // Will be calculated below
                    prompt_tokens_details: None,
                    completion_tokens_details: None,
                })
            }
            BedrockModelFamily::TitanText => {
                response.get("inputTextTokenCount").and_then(|input| {
                    response.get("results").and_then(|results| {
                        results.as_array().and_then(|arr| {
                            arr.first().and_then(|r| {
                                r.get("tokenCount").map(|output| Usage {
                                    prompt_tokens: input.as_u64().unwrap_or(0) as u32,
                                    completion_tokens: output.as_u64().unwrap_or(0) as u32,
                                    total_tokens: 0, // Will be calculated below
                                    prompt_tokens_details: None,
                                    completion_tokens_details: None,
                                })
                            })
                        })
                    })
                })
            }
            _ => None,
        };

        let mut final_usage = usage;
        if let Some(ref mut usage) = final_usage {
            usage.total_tokens = usage.prompt_tokens + usage.completion_tokens;
        }

        Ok(ChatResponse {
            id: format!("bedrock-{}", uuid::Uuid::new_v4()),
            object: "chat.completion".to_string(),
            created: chrono::Utc::now().timestamp(),
            model: model.to_string(),
            choices,
            usage: final_usage,
            system_fingerprint: None,
        })
    }

    async fn chat_completion(
        &self,
        request: ChatRequest,
        _context: RequestContext,
    ) -> Result<ChatResponse, Self::Error> {
        debug!("Bedrock chat request: model={}", request.model);

        // Check if it's an embedding model
        if self.is_embedding_model(&request.model) {
            return Err(ProviderError::invalid_request(
                "bedrock",
                "Use embeddings endpoint for embedding models".to_string(),
            ));
        }

        // Use the chat module's routing logic
        let response_value = super::chat::route_chat_request(&self.client, &request).await?;

        // Convert the response to bytes for transform_response
        let response_bytes = serde_json::to_vec(&response_value)
            .map_err(|e| ProviderError::serialization("bedrock", e.to_string()))?;

        self.transform_response(&response_bytes, &request.model, "bedrock-request")
            .await
    }

    async fn chat_completion_stream(
        &self,
        request: ChatRequest,
        context: RequestContext,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatChunk, Self::Error>> + Send>>, Self::Error>
    {
        debug!("Bedrock streaming chat request: model={}", request.model);

        // Check if it's an embedding model
        if self.is_embedding_model(&request.model) {
            return Err(ProviderError::invalid_request(
                "bedrock",
                "Use embeddings endpoint for embedding models".to_string(),
            ));
        }

        // Get model configuration
        let model_config = get_model_config(&request.model)?;

        if !model_config.supports_streaming {
            return Err(ProviderError::not_supported(
                "bedrock",
                format!("Model {} does not support streaming", request.model),
            ));
        }

        // Transform request
        let body = self.transform_request(request.clone(), context).await?;

        // Use streaming endpoint
        let operation = match model_config.api_type {
            super::model_config::BedrockApiType::ConverseStream => "converse-stream",
            super::model_config::BedrockApiType::InvokeStream => "invoke-with-response-stream",
            _ => {
                return Err(ProviderError::not_supported(
                    "bedrock",
                    format!(
                        "Model {} does not support streaming with API type {:?}",
                        request.model, model_config.api_type
                    ),
                ));
            }
        };

        // Send streaming request
        let response = self
            .client
            .send_streaming_request(&request.model, operation, &body)
            .await?;

        // Create BedrockStream
        let stream = super::streaming::BedrockStream::new(
            response.bytes_stream(),
            model_config.family.clone(),
        );

        Ok(Box::pin(stream))
    }

    async fn embeddings(
        &self,
        request: EmbeddingRequest,
        _context: RequestContext,
    ) -> Result<EmbeddingResponse, Self::Error> {
        debug!("Bedrock embedding request: model={}", request.model);

        // Use the embeddings module
        super::embeddings::execute_embedding(&self.client, &request).await
    }

    async fn health_check(&self) -> HealthStatus {
        match self.client.health_check().await {
            Ok(is_healthy) => {
                if is_healthy {
                    HealthStatus::Healthy
                } else {
                    HealthStatus::Unhealthy
                }
            }
            Err(_) => HealthStatus::Unhealthy,
        }
    }

    fn get_error_mapper(&self) -> Self::ErrorMapper {
        BedrockErrorMapper
    }

    async fn calculate_cost(
        &self,
        model: &str,
        input_tokens: u32,
        output_tokens: u32,
    ) -> Result<f64, Self::Error> {
        CostCalculator::calculate_cost(model, input_tokens, output_tokens)
            .ok_or_else(|| ProviderError::model_not_found("bedrock", model.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bedrock_provider_creation() {
        let config = BedrockConfig {
            aws_access_key_id: "AKIATEST123456789012".to_string(),
            aws_secret_access_key: "test_secret".to_string(),
            aws_session_token: None,
            aws_region: "us-east-1".to_string(),
            timeout_seconds: 30,
            max_retries: 3,
        };

        let provider = BedrockProvider::new(config).await;
        assert!(provider.is_ok());

        let provider = provider.unwrap();
        assert_eq!(provider.name(), "bedrock");
        assert!(
            provider
                .capabilities()
                .contains(&ProviderCapability::ChatCompletion)
        );
    }

    #[test]
    fn test_embedding_model_detection() {
        let config = BedrockConfig {
            aws_access_key_id: "test".to_string(),
            aws_secret_access_key: "test".to_string(),
            aws_session_token: None,
            aws_region: "us-east-1".to_string(),
            timeout_seconds: 30,
            max_retries: 3,
        };

        // Create a minimal provider for testing (without async)
        let provider = BedrockProvider {
            client: BedrockClient::new(config).unwrap(),
            models: vec![],
        };

        assert!(provider.is_embedding_model("amazon.titan-embed-text-v1"));
        assert!(!provider.is_embedding_model("anthropic.claude-3-sonnet"));
    }
}
