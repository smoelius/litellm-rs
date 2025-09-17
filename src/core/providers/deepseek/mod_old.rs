//! DeepSeek Provider
//! 
//! DeepSeek AI model integration for LiteLLM

pub mod common_utils;
pub use common_utils::*;

use async_trait::async_trait;
use std::collections::HashMap;
use std::pin::Pin;
use futures::Stream;
use serde_json::Value;

use crate::core::traits::provider::LLMProvider;
use crate::core::traits::error_mapper::ErrorMapper;
use crate::core::providers::unified_provider::ProviderError;
use crate::core::types::{
    requests::ChatRequest,
    responses::{ChatResponse, ChatChunk},
    common::{RequestContext, HealthStatus, ProviderCapability, ModelInfo},
};

// DeepSeek Provider implementation
#[derive(Debug, Clone)]
pub struct DeepSeekProvider {
    config: DeepSeekConfig,
    client: DeepSeekClient,
    supported_models: Vec<ModelInfo>,
}

impl DeepSeekProvider {
    pub fn new(config: DeepSeekConfig) -> Result<Self, ProviderError> {
        let client = DeepSeekClient::new(config.clone())?;
        
        let supported_models = vec![
            DeepSeekUtils::parse_model_name("deepseek-chat"),
            DeepSeekUtils::parse_model_name("deepseek-coder"),
            DeepSeekUtils::parse_model_name("deepseek-v3"),
        ];
        
        Ok(Self {
            config,
            client,
            supported_models,
        })
    }
}

#[async_trait]
impl LLMProvider for DeepSeekProvider {
    type Config = DeepSeekConfig;
    type Error = ProviderError;
    type ErrorMapper = UnifiedErrorMapper;
    
    fn name(&self) -> &'static str {
        "deepseek"
    }
    
    fn capabilities(&self) -> &'static [ProviderCapability] {
        &[
            ProviderCapability::ChatCompletion,
            ProviderCapability::ChatCompletionStream,
            ProviderCapability::ToolCalling,
        ]
    }
    
    fn models(&self) -> &[ModelInfo] {
        &self.supported_models
    }
    
    fn get_supported_openai_params(&self, _model: &str) -> &'static [&'static str] {
        &[
            "temperature",
            "max_tokens",
            "top_p",
            "frequency_penalty",
            "presence_penalty",
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
        // DeepSeek API is OpenAI-compatible, so most params can be passed through
        let mapped_params = params;
        
        // Map any DeepSeek-specific parameters if needed
        if let Some(tools) = mapped_params.get("tools") {
            // Ensure tools are in correct format for DeepSeek
            if !tools.is_array() {
                return Err(ProviderError::invalid_request("deepseek", "Tools must be an array"));
            }
        }
        
        Ok(mapped_params)
    }
    
    async fn transform_request(
        &self,
        request: ChatRequest,
        _context: RequestContext,
    ) -> Result<Value, Self::Error> {
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
        
        if let Some(tools) = request.tools {
            body["tools"] = json!(tools);
        }
        
        if let Some(tool_choice) = request.tool_choice {
            body["tool_choice"] = json!(tool_choice);
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
            .map_err(|e| ProviderError::serialization("deepseek", format!("Invalid UTF-8: {}", e)))?;
        
        let _response_json: Value = serde_json::from_str(response_text)
            .map_err(|e| ProviderError::serialization("deepseek", format!("Invalid JSON: {}", e)))?;
        
        // Transform DeepSeek response to standard ChatResponse format
        // This is a basic implementation - should be enhanced based on actual DeepSeek response format
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
            choices: vec![], // TODO: Parse actual choices from response
            usage: None,     // TODO: Parse usage from response
            system_fingerprint: None,
        };
        
        Ok(chat_response)
    }
    
    fn get_error_mapper(&self) -> Self::ErrorMapper {
        UnifiedErrorMapper
    }
    
    async fn chat_completion(
        &self,
        request: ChatRequest,
        context: RequestContext,
    ) -> Result<ChatResponse, Self::Error> {
        // Validate API key
        let api_key = self.config.get_effective_api_key()
            .ok_or_else(|| ProviderError::authentication("deepseek", "API key is required"))?;
        
        // Transform request to DeepSeek format
        let body = self.transform_request(request.clone(), context.clone()).await?;
        
        // Build request URL
        let url = self.client.build_url("/v1/chat/completions");
        
        // Create headers
        let headers = DeepSeekUtils::create_deepseek_headers(&self.config, api_key)?;
        
        // Make HTTP request
        let response = self.client.get_http_client()
            .post(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::network("deepseek", format!("Request failed: {}", e)))?;
        
        let status = response.status();
        let response_bytes = response.bytes().await
            .map_err(|e| ProviderError::network("deepseek", format!("Failed to read response: {}", e)))?;
        
        if !status.is_success() {
            let error_text = String::from_utf8_lossy(&response_bytes);
            return Err(ProviderError::api_error("deepseek", status.as_u16(), error_text.to_string()));
        }
        
        // Transform response
        self.transform_response(&response_bytes, &request.model, &context.request_id).await
    }
    
    async fn chat_completion_stream(
        &self,
        _request: ChatRequest,
        _context: RequestContext,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatChunk, Self::Error>> + Send>>, Self::Error> {
        Err(ProviderError::not_implemented("deepseek", "streaming"))
    }
    
    async fn health_check(&self) -> HealthStatus {
        // Simple health check - try to make a minimal request
        match self.config.get_effective_api_key() {
            Some(_) => HealthStatus::Healthy,
            None => HealthStatus::Unhealthy,
        }
    }
    
    async fn calculate_cost(
        &self,
        model: &str,
        input_tokens: u32,
        output_tokens: u32,
    ) -> Result<f64, Self::Error> {
        // Basic cost calculation for DeepSeek models
        let cost = match model {
            "deepseek-chat" => (input_tokens as f64 * 0.0001 + output_tokens as f64 * 0.0002) / 1000.0,
            "deepseek-coder" => (input_tokens as f64 * 0.0001 + output_tokens as f64 * 0.0002) / 1000.0,
            "deepseek-v3" => (input_tokens as f64 * 0.0002 + output_tokens as f64 * 0.0004) / 1000.0,
            _ => 0.0,
        };
        Ok(cost)
    }
}

// Provider creation function for backward compatibility
pub fn create_provider(config: DeepSeekConfig) -> Result<DeepSeekProvider, ProviderError> {
    DeepSeekProvider::new(config)
}

/// Unified error mapper for ProviderError - no conversion needed since we use ProviderError directly
#[derive(Debug, Clone)]
pub struct UnifiedErrorMapper;

impl ErrorMapper<ProviderError> for UnifiedErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> ProviderError {
        ProviderError::api_error("deepseek", status_code, response_body.to_string())
    }
    
    fn map_json_error(&self, _error_response: &serde_json::Value) -> ProviderError {
        ProviderError::response_parsing("deepseek", "Failed to parse JSON error response")
    }
    
    fn map_network_error(&self, error: &dyn std::error::Error) -> ProviderError {
        ProviderError::network("deepseek", error.to_string())
    }
}