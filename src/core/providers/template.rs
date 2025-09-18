//! Provider Migration Template
//!
//! Copy this file and replace PROVIDER_NAME with the actual provider name

use async_trait::async_trait;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use futures::Stream;
use serde_json::{json, Value};

use crate::core::providers::base::{BaseConfig, get_http_client, get_pricing_db};
use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::{LLMProvider, ErrorMapper, ProviderConfig};
use crate::core::types::{
    requests::ChatRequest,
    responses::{ChatResponse, ChatChunk},
    common::{RequestContext, HealthStatus, ProviderCapability, ModelInfo},
};
use crate::define_provider_config;

// Configuration
// Configuration
// Example: model_preference: String = "default".to_string()
define_provider_config!(PROVIDER_NAMEConfig {});

// ========== Providerimplementation ==========
#[derive(Debug)]
pub struct PROVIDER_NAMEProvider {
    config: PROVIDER_NAMEConfig,
    client: Arc<crate::core::providers::base::UnifiedHttpClient>,
    supported_models: Vec<ModelInfo>,
}

impl PROVIDER_NAMEProvider {
    pub fn new(config: PROVIDER_NAMEConfig) -> Result<Self, ProviderError> {
        // Configuration
        config.base.validate("PROVIDER_NAME_LOWER")
            .map_err(|e| ProviderError::configuration("PROVIDER_NAME_LOWER", e))?;
        
        // Get
        let client = get_http_client();
        
        // Settings
        client.set_rate_limit("PROVIDER_NAME_LOWER", 100.0);
        
        // Define supported models
        let supported_models = vec![
            ModelInfo {
                id: "MODEL_ID".to_string(),
                name: "MODEL_NAME".to_string(),
                provider: "PROVIDER_NAME_LOWER".to_string(),
                max_context_length: 4096,
                max_output_length: Some(2048),
                supports_streaming: true,
                supports_tools: false,
                supports_multimodal: false,
                input_cost_per_1k_tokens: Some(0.001),
                output_cost_per_1k_tokens: Some(0.002),
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
            supported_models,
        })
    }
    
    pub fn from_env() -> Result<Self, ProviderError> {
        let config = PROVIDER_NAMEConfig::new("PROVIDER_NAME_LOWER");
        Self::new(config)
    }
    
    fn transform_chat_request(&self, request: ChatRequest) -> Value {
        // Adjust according to provider's API format
        json!({
            "model": request.model,
            "messages": request.messages,
            "temperature": request.temperature,
            "max_tokens": request.max_tokens,
            // Add provider-specific fields
        })
    }
    
    fn transform_chat_response(&self, response: Value) -> Result<ChatResponse, ProviderError> {
        // Response
        serde_json::from_value(response)
            .map_err(|e| ProviderError::response_parsing("PROVIDER_NAME_LOWER", e.to_string()))
    }
}

// Error
#[derive(Debug)]
pub struct PROVIDER_NAMEErrorMapper;

impl ErrorMapper<ProviderError> for PROVIDER_NAMEErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> ProviderError {
        match status_code {
            401 => ProviderError::authentication("PROVIDER_NAME_LOWER", response_body),
            403 => ProviderError::authentication("PROVIDER_NAME_LOWER", "Permission denied"),
            404 => ProviderError::model_not_found("PROVIDER_NAME_LOWER", "Model not found"),
            429 => ProviderError::rate_limit("PROVIDER_NAME_LOWER", None),
            500..=599 => ProviderError::api_error("PROVIDER_NAME_LOWER", status_code, response_body),
            _ => ProviderError::api_error("PROVIDER_NAME_LOWER", status_code, response_body),
        }
    }
}

// ========== LLMProviderimplementation ==========
#[async_trait]
impl LLMProvider for PROVIDER_NAMEProvider {
    type Config = PROVIDER_NAMEConfig;
    type Error = ProviderError;
    type ErrorMapper = PROVIDER_NAMEErrorMapper;
    
    fn name(&self) -> &'static str {
        "PROVIDER_NAME_LOWER"
    }
    
    fn capabilities(&self) -> &'static [ProviderCapability] {
        &[
            ProviderCapability::ChatCompletion,
            // Add according to provider capabilities:
            // ProviderCapability::ChatCompletionStream,
            // ProviderCapability::ToolCalling,
            // ProviderCapability::VisionModel,
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
            // Add provider-supported parameters
        ]
    }
    
    async fn map_openai_params(
        &self,
        params: HashMap<String, Value>,
        _model: &str,
    ) -> Result<HashMap<String, Value>, Self::Error> {
        // If API is OpenAI compatible, return directly
        Ok(params)
        
        // Otherwise perform parameter mapping
        // let mut mapped = HashMap::new();
        // if let Some(temp) = params.get("temperature") {
        //     mapped.insert("PROVIDER_TEMP_PARAM".to_string(), temp.clone());
        // }
        // Ok(mapped)
    }
    
    async fn transform_request(
        &self,
        request: ChatRequest,
        _context: RequestContext,
    ) -> Result<Value, Self::Error> {
        Ok(self.transform_chat_request(request))
    }
    
    async fn transform_response(
        &self,
        raw_response: &[u8],
        _model: &str,
        _request_id: &str,
    ) -> Result<ChatResponse, Self::Error> {
        let response: Value = serde_json::from_slice(raw_response)
            .map_err(|e| ProviderError::response_parsing("PROVIDER_NAME_LOWER", e.to_string()))?;
        self.transform_chat_response(response)
    }
    
    fn get_error_mapper(&self) -> Self::ErrorMapper {
        PROVIDER_NAMEErrorMapper
    }
    
    async fn chat_completion(
        &self,
        request: ChatRequest,
        context: RequestContext,
    ) -> Result<ChatResponse, Self::Error> {
        let url = format!("{}/ENDPOINT", self.config.base.get_effective_api_base("PROVIDER_NAME_LOWER"));
        let body = self.transform_chat_request(request.clone());
        
        let response = self.client
            .post(&url, body, &self.config.base, "PROVIDER_NAME_LOWER")
            .await?;
        
        let response_bytes = response.bytes().await
            .map_err(|e| ProviderError::network("PROVIDER_NAME_LOWER", e.to_string()))?;
        
        self.transform_response(&response_bytes, &request.model, &context.request_id).await
    }
    
    async fn chat_completion_stream(
        &self,
        _request: ChatRequest,
        _context: RequestContext,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatChunk, Self::Error>> + Send>>, Self::Error> {
        // Response
        Err(ProviderError::not_implemented("PROVIDER_NAME_LOWER", "Streaming not yet implemented"))
    }
    
    async fn health_check(&self) -> HealthStatus {
        if self.config.base.get_effective_api_key("PROVIDER_NAME_LOWER").is_some() {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unhealthy
        }
    }
    
    async fn calculate_cost(
        &self,
        model: &str,
        input_tokens: u32,
        output_tokens: u32,
    ) -> Result<f64, Self::Error> {
        // Use global pricing database
        let usage = crate::core::providers::base::pricing::Usage {
            prompt_tokens: input_tokens,
            completion_tokens: output_tokens,
            total_tokens: input_tokens + output_tokens,
            reasoning_tokens: None,
        };
        
        Ok(get_pricing_db().calculate(model, &usage))
    }
}

// ========== ProviderConfigimplementation ==========
impl ProviderConfig for PROVIDER_NAMEConfig {
    fn validate(&self) -> Result<(), String> {
        self.base.validate("PROVIDER_NAME_LOWER")
    }
    
    fn api_key(&self) -> Option<&str> {
        self.base.api_key.as_deref()
    }
    
    fn api_base(&self) -> Option<&str> {
        self.base.api_base.as_deref()
    }
    
    fn timeout(&self) -> std::time::Duration {
        self.base.timeout_duration()
    }
    
    fn max_retries(&self) -> u32 {
        self.base.max_retries
    }
}

// ========== Tests ==========
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config() {
        let config = PROVIDER_NAMEConfig::new("PROVIDER_NAME_LOWER");
        assert_eq!(config.base.timeout, 60);
    }
}

// ========== Migration Steps ==========
// 1. Copy this file to src/core/providers/PROVIDER_NAME/mod.rs
// 2. Globally replace PROVIDER_NAME with actual provider name (PascalCase, e.g. OpenAI)
// 3. Globally replace PROVIDER_NAME_LOWER with lowercase name (e.g. openai)
// 4. Replace MODEL_ID, MODEL_NAME, ENDPOINT etc placeholders
// 5. Adjust according to provider API:
//    - Request transformation
//    - Response transformation
//    - Supported parameter list
//    - Error handling
// 6. Add module import in src/core/providers/mod.rs
// 7. Add new variant in Provider enum
// 8. Update registry