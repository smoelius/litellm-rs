//! OpenAI Provider Client Implementation
//!
//! Unified client following the new provider architecture

use async_trait::async_trait;
use futures::Stream;
use serde_json::Value;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;

use crate::core::providers::base::{header, header_owned, GlobalPoolManager, HeaderPair, HttpMethod};
use crate::core::traits::provider::llm_provider::trait_definition::LLMProvider;
use crate::core::types::{
    common::{HealthStatus, ModelInfo, ProviderCapability, RequestContext},
    requests::{ChatRequest, EmbeddingRequest},
    responses::{ChatChunk, ChatResponse, EmbeddingResponse},
};

use super::{
    advanced_chat::{AdvancedChatRequest, AdvancedChatUtils},
    // New functionality modules
    completions::validate_completion_request,
    config::{OpenAIConfig, OpenAIFeature},
    error::OpenAIError,
    fine_tuning::{OpenAIFineTuningRequest, OpenAIFineTuningUtils},
    image_edit::{OpenAIImageEditRequest, OpenAIImageEditUtils},
    image_variations::{OpenAIImageVariationsRequest, OpenAIImageVariationsUtils},
    models::{OpenAIModelRegistry, get_openai_registry},
    realtime::{OpenAIRealtimeUtils, RealtimeSessionConfig},
    vector_stores::{OpenAIVectorStoreRequest, OpenAIVectorStoreUtils},
};

/// OpenAI Provider implementation using unified architecture
#[derive(Debug, Clone)]
pub struct OpenAIProvider {
    /// Connection pool manager
    pool_manager: Arc<GlobalPoolManager>,
    /// Provider configuration
    config: OpenAIConfig,
    /// Model registry
    model_registry: &'static OpenAIModelRegistry,
}

impl OpenAIProvider {
    /// Generate headers for OpenAI API requests
    ///
    /// Uses `HeaderPair` with Cow for static keys to avoid allocations.
    fn get_request_headers(&self) -> Vec<HeaderPair> {
        let mut headers = Vec::with_capacity(4); // Pre-allocate for typical case

        if let Some(api_key) = &self.config.base.api_key {
            headers.push(header("Authorization", format!("Bearer {}", api_key)));
        }

        if let Some(org) = &self.config.organization {
            headers.push(header("OpenAI-Organization", org.clone()));
        }

        if let Some(project) = &self.config.project {
            headers.push(header("OpenAI-Project", project.clone()));
        }

        // Add custom headers (both key and value are dynamic)
        for (key, value) in &self.config.base.headers {
            headers.push(header_owned(key.clone(), value.clone()));
        }

        headers
    }

    /// Create new OpenAI provider
    pub async fn new(config: OpenAIConfig) -> Result<Self, OpenAIError> {
        // Validate configuration
        config.validate().map_err(|e| OpenAIError::Configuration {
            provider: "openai",
            message: e.to_string(),
        })?;

        // Note: Headers are now built per-request in get_request_headers()
        // This avoids redundant HashMap allocation during initialization.

        let pool_manager =
            Arc::new(GlobalPoolManager::new().map_err(|e| OpenAIError::Network {
                provider: "openai",
                message: e.to_string(),
            })?);
        let model_registry = get_openai_registry();

        Ok(Self {
            pool_manager,
            config,
            model_registry,
        })
    }

    /// Create provider with API key only
    pub async fn with_api_key(api_key: impl Into<String>) -> Result<Self, OpenAIError> {
        let mut config = OpenAIConfig::default();
        config.base.api_key = Some(api_key.into());
        Self::new(config).await
    }

    /// Execute chat completion request
    async fn execute_chat_completion(
        &self,
        request: ChatRequest,
    ) -> Result<ChatResponse, OpenAIError> {
        // Transform request to OpenAI format
        let openai_request = self.transform_chat_request(request)?;

        // Execute HTTP request using unified connection pool
        let url = format!("{}/chat/completions", self.config.get_api_base());
        let headers = self.get_request_headers();
        let body = Some(openai_request);

        let response = self
            .pool_manager
            .execute_request(&url, HttpMethod::POST, headers, body)
            .await
            .map_err(|e| OpenAIError::Network {
                provider: "openai",
                message: e.to_string(),
            })?;

        let response_bytes = response.bytes().await.map_err(|e| OpenAIError::Network {
            provider: "openai",
            message: e.to_string(),
        })?;

        let response_json: Value =
            serde_json::from_slice(&response_bytes).map_err(|e| OpenAIError::ResponseParsing {
                provider: "openai",
                message: e.to_string(),
            })?;

        // Transform response back to standard format
        self.transform_chat_response(response_json)
    }

    /// Execute streaming chat completion
    async fn execute_chat_completion_stream(
        &self,
        request: ChatRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatChunk, OpenAIError>> + Send>>, OpenAIError>
    {
        // Transform request with streaming enabled
        let mut openai_request = self.transform_chat_request(request)?;
        openai_request["stream"] = Value::Bool(true);

        // Get API key
        let api_key =
            self.config
                .base
                .api_key
                .as_ref()
                .ok_or_else(|| OpenAIError::Authentication {
                    provider: "openai",
                    message: "API key is required".to_string(),
                })?;

        // Execute streaming request using reqwest directly for streaming
        let url = format!("{}/chat/completions", self.config.get_api_base());
        let client = reqwest::Client::new();
        let mut req = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&openai_request);

        // Add organization header if present
        if let Some(org) = &self.config.organization {
            req = req.header("OpenAI-Organization", org);
        }

        // Add project header if present
        if let Some(project) = &self.config.project {
            req = req.header("OpenAI-Project", project);
        }

        let response = req.send().await.map_err(|e| OpenAIError::Network {
            provider: "openai",
            message: e.to_string(),
        })?;

        // Create OpenAI-specific stream handler using unified SSE parser
        let stream = response.bytes_stream();
        Ok(Box::pin(super::streaming::create_openai_stream(stream)))
    }

    /// Transform ChatRequest to OpenAI API format
    fn transform_chat_request(&self, request: ChatRequest) -> Result<Value, OpenAIError> {
        let mut openai_request = serde_json::json!({
            "model": self.config.get_model_mapping(&request.model),
            "messages": request.messages
        });

        // Add optional parameters
        if let Some(temp) = request.temperature {
            openai_request["temperature"] =
                Value::Number(serde_json::Number::from_f64(temp as f64).unwrap());
        }

        if let Some(max_tokens) = request.max_tokens {
            openai_request["max_tokens"] = Value::Number(serde_json::Number::from(max_tokens));
        }

        if let Some(max_completion_tokens) = request.max_completion_tokens {
            openai_request["max_completion_tokens"] =
                Value::Number(serde_json::Number::from(max_completion_tokens));
        }

        if let Some(top_p) = request.top_p {
            openai_request["top_p"] =
                Value::Number(serde_json::Number::from_f64(top_p as f64).unwrap());
        }

        if let Some(tools) = request.tools {
            openai_request["tools"] = serde_json::to_value(tools)?;
        }

        if let Some(tool_choice) = request.tool_choice {
            openai_request["tool_choice"] = serde_json::to_value(tool_choice)?;
        }

        if let Some(response_format) = request.response_format {
            openai_request["response_format"] = serde_json::to_value(response_format)?;
        }

        if let Some(stop) = request.stop {
            openai_request["stop"] = serde_json::to_value(stop)?;
        }

        if let Some(user) = request.user {
            openai_request["user"] = Value::String(user);
        }

        if let Some(seed) = request.seed {
            openai_request["seed"] = Value::Number(serde_json::Number::from(seed));
        }

        if let Some(n) = request.n {
            openai_request["n"] = Value::Number(serde_json::Number::from(n));
        }

        // Add extra parameters from config
        // Skip extra_params as BaseConfig doesn't have it

        Ok(openai_request)
    }

    /// Transform OpenAI response to standard format
    fn transform_chat_response(&self, response: Value) -> Result<ChatResponse, OpenAIError> {
        let response: crate::core::providers::openai::models::OpenAIChatResponse =
            serde_json::from_value(response)?;

        // Use existing transformer logic
        use crate::core::providers::openai::transformer::OpenAIResponseTransformer;
        OpenAIResponseTransformer::transform(response).map_err(|e| OpenAIError::Other {
            provider: "openai",
            message: e.to_string(),
        })
    }

    /// Get model information with validation
    pub fn get_model_info(
        &self,
        model_id: &str,
    ) -> Result<crate::core::types::common::ModelInfo, OpenAIError> {
        // Return a default ModelInfo for any model
        // Like Python LiteLLM, we don't validate models locally
        use crate::core::types::common::ModelInfo;
        Ok(ModelInfo {
            id: model_id.to_string(),
            name: model_id.to_string(),
            provider: "openai".to_string(),
            max_context_length: 128000, // Default context
            max_output_length: Some(4096),
            supports_streaming: true,
            supports_tools: true,
            supports_multimodal: false,
            capabilities: vec![], // Empty capabilities, API will handle validation
            input_cost_per_1k_tokens: None,
            output_cost_per_1k_tokens: None,
            currency: "USD".to_string(),
            created_at: None,
            updated_at: None,
            metadata: std::collections::HashMap::new(),
        })
    }

    /// Check if model supports a specific capability
    pub fn model_supports_capability(
        &self,
        model_id: &str,
        capability: &ProviderCapability,
    ) -> bool {
        if let Some(model_spec) = self.model_registry.get_model_spec(model_id) {
            model_spec.model_info.capabilities.contains(capability)
        } else {
            false
        }
    }

    /// Get model configuration
    pub fn get_model_config(&self, model_id: &str) -> Option<&super::models::OpenAIModelConfig> {
        self.model_registry
            .get_model_spec(model_id)
            .map(|spec| &spec.config)
    }
}

#[async_trait]
impl LLMProvider for OpenAIProvider {
    type Config = OpenAIConfig;
    type Error = OpenAIError;
    type ErrorMapper = crate::core::traits::error_mapper::implementations::OpenAIErrorMapper;

    fn name(&self) -> &'static str {
        "openai"
    }

    fn capabilities(&self) -> &'static [ProviderCapability] {
        static CAPABILITIES: &[ProviderCapability] = &[
            ProviderCapability::ChatCompletion,
            ProviderCapability::ChatCompletionStream,
            ProviderCapability::Embeddings,
            ProviderCapability::ImageGeneration,
            ProviderCapability::AudioTranscription,
            ProviderCapability::ToolCalling,
            ProviderCapability::FunctionCalling,
            // New capabilities
            ProviderCapability::FineTuning,
            ProviderCapability::ImageEdit,
            ProviderCapability::ImageVariation,
            ProviderCapability::RealtimeApi,
        ];
        CAPABILITIES
    }

    fn models(&self) -> &[ModelInfo] {
        static MODELS: std::sync::LazyLock<Vec<ModelInfo>> =
            std::sync::LazyLock::new(|| get_openai_registry().get_all_models());
        &MODELS
    }

    async fn chat_completion(
        &self,
        request: ChatRequest,
        _context: RequestContext,
    ) -> Result<ChatResponse, Self::Error> {
        // Like Python LiteLLM, we don't validate models locally
        // OpenAI API will handle invalid models

        // Execute request
        self.execute_chat_completion(request).await
    }

    async fn chat_completion_stream(
        &self,
        request: ChatRequest,
        _context: RequestContext,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatChunk, Self::Error>> + Send>>, Self::Error>
    {
        // Like Python LiteLLM, we don't validate models locally
        // OpenAI API will handle invalid models

        self.execute_chat_completion_stream(request).await
    }

    async fn health_check(&self) -> HealthStatus {
        let url = format!("{}/models?limit=1", self.config.get_api_base());
        let client = reqwest::Client::new();
        let mut req = client.get(&url);

        if let Some(api_key) = &self.config.base.api_key {
            req = req.header("Authorization", format!("Bearer {}", api_key));
        }

        match req.send().await {
            Ok(response) if response.status().is_success() => HealthStatus::Healthy,
            _ => HealthStatus::Unhealthy,
        }
    }

    async fn calculate_cost(
        &self,
        model: &str,
        input_tokens: u32,
        output_tokens: u32,
    ) -> Result<f64, Self::Error> {
        let model_info = self.get_model_info(model)?;

        let input_cost = model_info
            .input_cost_per_1k_tokens
            .map(|cost| (input_tokens as f64 / 1000.0) * cost)
            .unwrap_or(0.0);

        let output_cost = model_info
            .output_cost_per_1k_tokens
            .map(|cost| (output_tokens as f64 / 1000.0) * cost)
            .unwrap_or(0.0);

        Ok(input_cost + output_cost)
    }

    // ==================== Python LiteLLM Compatible Interface ====================

    fn get_supported_openai_params(&self, model: &str) -> &'static [&'static str] {
        // Return parameters based on model capabilities
        if let Some(model_spec) = self.model_registry.get_model_spec(model) {
            match model_spec.family {
                super::models::OpenAIModelFamily::GPT4
                | super::models::OpenAIModelFamily::GPT4Turbo
                | super::models::OpenAIModelFamily::GPT4O => &[
                    "messages",
                    "model",
                    "temperature",
                    "max_tokens",
                    "max_completion_tokens",
                    "top_p",
                    "frequency_penalty",
                    "presence_penalty",
                    "stop",
                    "stream",
                    "tools",
                    "tool_choice",
                    "parallel_tool_calls",
                    "response_format",
                    "user",
                    "seed",
                    "n",
                    "logit_bias",
                    "logprobs",
                    "top_logprobs",
                ],
                super::models::OpenAIModelFamily::GPT35 => &[
                    "messages",
                    "model",
                    "temperature",
                    "max_tokens",
                    "top_p",
                    "frequency_penalty",
                    "presence_penalty",
                    "stop",
                    "stream",
                    "tools",
                    "tool_choice",
                    "response_format",
                    "user",
                    "n",
                    "logit_bias",
                ],
                super::models::OpenAIModelFamily::O1 => &[
                    "messages",
                    "model",
                    "max_completion_tokens",
                    "stream",
                    "user",
                ],
                _ => &[
                    "messages",
                    "model",
                    "temperature",
                    "max_tokens",
                    "top_p",
                    "stream",
                    "user",
                ],
            }
        } else {
            &[
                "messages",
                "model",
                "temperature",
                "max_tokens",
                "top_p",
                "stream",
                "user",
            ]
        }
    }

    async fn map_openai_params(
        &self,
        params: HashMap<String, Value>,
        _model: &str,
    ) -> Result<HashMap<String, Value>, Self::Error> {
        // OpenAI provider uses standard OpenAI parameters, no mapping needed
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
        _model: &str,
        _request_id: &str,
    ) -> Result<ChatResponse, Self::Error> {
        let response_value: Value = serde_json::from_slice(raw_response)?;
        self.transform_chat_response(response_value)
    }

    fn get_error_mapper(&self) -> Self::ErrorMapper {
        crate::core::traits::error_mapper::implementations::OpenAIErrorMapper
    }
}

// Additional OpenAI-specific methods
impl OpenAIProvider {
    /// Generate embeddings
    pub async fn embeddings(
        &self,
        request: EmbeddingRequest,
    ) -> Result<EmbeddingResponse, OpenAIError> {
        // Like Python LiteLLM, we don't validate models locally
        // OpenAI API will handle invalid models

        // Transform to OpenAI format
        let openai_request = serde_json::json!({
            "input": request.input,
            "model": request.model,
            "encoding_format": request.encoding_format,
            "dimensions": request.dimensions,
            "user": request.user
        });

        // Execute request using high-performance connection pool
        let url = format!("{}/embeddings", self.config.get_api_base());

        let headers = self.get_request_headers();
        let body = Some(openai_request);

        let response = self
            .pool_manager
            .execute_request(&url, HttpMethod::POST, headers, body)
            .await
            .map_err(|e| OpenAIError::Network {
                provider: "openai",
                message: e.to_string(),
            })?;

        let response_bytes = response.bytes().await.map_err(|e| OpenAIError::Network {
            provider: "openai",
            message: e.to_string(),
        })?;

        let response_json: Value =
            serde_json::from_slice(&response_bytes).map_err(|e| OpenAIError::ResponseParsing {
                provider: "openai",
                message: e.to_string(),
            })?;

        // Transform response
        serde_json::from_value(response_json).map_err(|e| OpenAIError::ResponseParsing {
            provider: "openai",
            message: e.to_string(),
        })
    }

    /// Generate images (DALL-E)
    pub async fn generate_images(
        &self,
        prompt: String,
        model: Option<String>,
        n: Option<u32>,
        size: Option<String>,
        quality: Option<String>,
        style: Option<String>,
    ) -> Result<Value, OpenAIError> {
        let model = model.unwrap_or_else(|| "dall-e-3".to_string());

        // Validate image generation capability
        if !self
            .config
            .is_feature_enabled(OpenAIFeature::ImageGeneration)
        {
            return Err(OpenAIError::NotSupported {
                provider: "openai",
                feature: "Image generation is disabled in configuration".to_string(),
            });
        }

        let request = serde_json::json!({
            "prompt": prompt,
            "model": model,
            "n": n,
            "size": size,
            "quality": quality,
            "style": style
        });

        let url = format!("{}/images/generations", self.config.get_api_base());

        let headers = self.get_request_headers();
        let body = Some(request);

        let response = self
            .pool_manager
            .execute_request(&url, HttpMethod::POST, headers, body)
            .await
            .map_err(|e| OpenAIError::Network {
                provider: "openai",
                message: e.to_string(),
            })?;

        let response_bytes = response.bytes().await.map_err(|e| OpenAIError::Network {
            provider: "openai",
            message: e.to_string(),
        })?;

        serde_json::from_slice(&response_bytes).map_err(|e| OpenAIError::ResponseParsing {
            provider: "openai",
            message: e.to_string(),
        })
    }

    /// Audio transcription (Whisper)
    pub async fn transcribe_audio(
        &self,
        _file: Vec<u8>,
        model: Option<String>,
        language: Option<String>,
        response_format: Option<String>,
    ) -> Result<Value, OpenAIError> {
        if !self
            .config
            .is_feature_enabled(OpenAIFeature::AudioTranscription)
        {
            return Err(OpenAIError::NotSupported {
                provider: "openai",
                feature: "Audio transcription is disabled in configuration".to_string(),
            });
        }

        // This would need multipart form handling - simplified for now
        let request = serde_json::json!({
            "model": model.unwrap_or_else(|| "whisper-1".to_string()),
            "language": language,
            "response_format": response_format
        });

        // In a real implementation, this would handle file upload
        let url = format!("{}/audio/transcriptions", self.config.get_api_base());

        let headers = self.get_request_headers();
        let body = Some(request);

        let response = self
            .pool_manager
            .execute_request(&url, HttpMethod::POST, headers, body)
            .await
            .map_err(|e| OpenAIError::Network {
                provider: "openai",
                message: e.to_string(),
            })?;

        let response_bytes = response.bytes().await.map_err(|e| OpenAIError::Network {
            provider: "openai",
            message: e.to_string(),
        })?;

        serde_json::from_slice(&response_bytes).map_err(|e| OpenAIError::ResponseParsing {
            provider: "openai",
            message: e.to_string(),
        })
    }

    // ==================== NEW FUNCTIONALITY METHODS ====================

    /// Text completion (legacy)
    pub async fn text_completion(
        &self,
        request: super::completions::OpenAICompletionRequest,
    ) -> Result<super::completions::OpenAICompletionResponse, OpenAIError> {
        // Validate request
        validate_completion_request(&request).map_err(|e| OpenAIError::InvalidRequest {
            provider: "openai",
            message: e.to_string(),
        })?;

        // Execute request
        let url = format!("{}/completions", self.config.get_api_base());
        let request_value =
            serde_json::to_value(request).map_err(|e| OpenAIError::InvalidRequest {
                provider: "openai",
                message: e.to_string(),
            })?;

        let headers = self.get_request_headers();
        let body = Some(request_value);

        let response = self
            .pool_manager
            .execute_request(&url, HttpMethod::POST, headers, body)
            .await
            .map_err(|e| OpenAIError::Network {
                provider: "openai",
                message: e.to_string(),
            })?;

        let response_bytes = response.bytes().await.map_err(|e| OpenAIError::Network {
            provider: "openai",
            message: e.to_string(),
        })?;

        serde_json::from_slice(&response_bytes).map_err(|e| OpenAIError::ResponseParsing {
            provider: "openai",
            message: e.to_string(),
        })
    }

    /// Create fine-tuning job
    pub async fn create_fine_tuning_job(
        &self,
        request: OpenAIFineTuningRequest,
    ) -> Result<super::fine_tuning::OpenAIFineTuningJob, OpenAIError> {
        // Validate request
        OpenAIFineTuningUtils::validate_request(&request).map_err(|e| {
            OpenAIError::InvalidRequest {
                provider: "openai",
                message: e.to_string(),
            }
        })?;

        // Execute request
        let url = format!("{}/fine_tuning/jobs", self.config.get_api_base());
        let request_value =
            serde_json::to_value(request).map_err(|e| OpenAIError::InvalidRequest {
                provider: "openai",
                message: e.to_string(),
            })?;

        let headers = self.get_request_headers();
        let body = Some(request_value);

        let response = self
            .pool_manager
            .execute_request(&url, HttpMethod::POST, headers, body)
            .await
            .map_err(|e| OpenAIError::Network {
                provider: "openai",
                message: e.to_string(),
            })?;

        let response_bytes = response.bytes().await.map_err(|e| OpenAIError::Network {
            provider: "openai",
            message: e.to_string(),
        })?;

        serde_json::from_slice(&response_bytes).map_err(|e| OpenAIError::ResponseParsing {
            provider: "openai",
            message: e.to_string(),
        })
    }

    /// List fine-tuning jobs
    pub async fn list_fine_tuning_jobs(
        &self,
        after: Option<String>,
        limit: Option<u32>,
    ) -> Result<Value, OpenAIError> {
        let mut query_params = Vec::new();
        if let Some(after) = after {
            query_params.push(format!("after={}", after));
        }
        if let Some(limit) = limit {
            query_params.push(format!("limit={}", limit));
        }

        let endpoint = if query_params.is_empty() {
            "fine_tuning/jobs".to_string()
        } else {
            format!("fine_tuning/jobs?{}", query_params.join("&"))
        };

        let url = format!("{}/{}", self.config.get_api_base(), endpoint);
        let client = reqwest::Client::new();
        let mut req = client.get(&url);

        if let Some(api_key) = &self.config.base.api_key {
            req = req.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = req.send().await.map_err(|e| OpenAIError::Network {
            provider: "openai",
            message: e.to_string(),
        })?;

        let response_bytes = response.bytes().await.map_err(|e| OpenAIError::Network {
            provider: "openai",
            message: e.to_string(),
        })?;

        serde_json::from_slice(&response_bytes).map_err(|e| OpenAIError::ResponseParsing {
            provider: "openai",
            message: e.to_string(),
        })
    }

    /// Edit image
    pub async fn edit_image(
        &self,
        request: OpenAIImageEditRequest,
    ) -> Result<super::image_edit::OpenAIImageEditResponse, OpenAIError> {
        // Validate request
        OpenAIImageEditUtils::validate_request(&request).map_err(|e| {
            OpenAIError::InvalidRequest {
                provider: "openai",
                message: e.to_string(),
            }
        })?;

        // Execute request (would need multipart form data in real implementation)
        let url = format!("{}/images/edits", self.config.get_api_base());
        let request_value =
            serde_json::to_value(request).map_err(|e| OpenAIError::InvalidRequest {
                provider: "openai",
                message: e.to_string(),
            })?;

        let headers = self.get_request_headers();
        let body = Some(request_value);

        let response = self
            .pool_manager
            .execute_request(&url, HttpMethod::POST, headers, body)
            .await
            .map_err(|e| OpenAIError::Network {
                provider: "openai",
                message: e.to_string(),
            })?;

        let response_bytes = response.bytes().await.map_err(|e| OpenAIError::Network {
            provider: "openai",
            message: e.to_string(),
        })?;

        serde_json::from_slice(&response_bytes).map_err(|e| OpenAIError::ResponseParsing {
            provider: "openai",
            message: e.to_string(),
        })
    }

    /// Create image variations
    pub async fn create_image_variations(
        &self,
        request: OpenAIImageVariationsRequest,
    ) -> Result<super::image_variations::OpenAIImageVariationsResponse, OpenAIError> {
        // Validate request
        OpenAIImageVariationsUtils::validate_request(&request).map_err(|e| {
            OpenAIError::InvalidRequest {
                provider: "openai",
                message: e.to_string(),
            }
        })?;

        // Execute request (would need multipart form data in real implementation)
        let url = format!("{}/images/variations", self.config.get_api_base());
        let request_value =
            serde_json::to_value(request).map_err(|e| OpenAIError::InvalidRequest {
                provider: "openai",
                message: e.to_string(),
            })?;

        let headers = self.get_request_headers();
        let body = Some(request_value);

        let response = self
            .pool_manager
            .execute_request(&url, HttpMethod::POST, headers, body)
            .await
            .map_err(|e| OpenAIError::Network {
                provider: "openai",
                message: e.to_string(),
            })?;

        let response_bytes = response.bytes().await.map_err(|e| OpenAIError::Network {
            provider: "openai",
            message: e.to_string(),
        })?;

        serde_json::from_slice(&response_bytes).map_err(|e| OpenAIError::ResponseParsing {
            provider: "openai",
            message: e.to_string(),
        })
    }

    /// Create vector store
    pub async fn create_vector_store(
        &self,
        request: OpenAIVectorStoreRequest,
    ) -> Result<super::vector_stores::OpenAIVectorStore, OpenAIError> {
        // Validate request
        OpenAIVectorStoreUtils::validate_request(&request).map_err(|e| {
            OpenAIError::InvalidRequest {
                provider: "openai",
                message: e.to_string(),
            }
        })?;

        // Execute request
        let url = format!("{}/vector_stores", self.config.get_api_base());
        let request_value =
            serde_json::to_value(request).map_err(|e| OpenAIError::InvalidRequest {
                provider: "openai",
                message: e.to_string(),
            })?;

        let headers = self.get_request_headers();
        let body = Some(request_value);

        let response = self
            .pool_manager
            .execute_request(&url, HttpMethod::POST, headers, body)
            .await
            .map_err(|e| OpenAIError::Network {
                provider: "openai",
                message: e.to_string(),
            })?;

        let response_bytes = response.bytes().await.map_err(|e| OpenAIError::Network {
            provider: "openai",
            message: e.to_string(),
        })?;

        serde_json::from_slice(&response_bytes).map_err(|e| OpenAIError::ResponseParsing {
            provider: "openai",
            message: e.to_string(),
        })
    }

    /// List vector stores
    pub async fn list_vector_stores(
        &self,
        limit: Option<u32>,
        order: Option<String>,
        after: Option<String>,
        before: Option<String>,
    ) -> Result<Value, OpenAIError> {
        let mut query_params = Vec::new();
        if let Some(limit) = limit {
            query_params.push(format!("limit={}", limit));
        }
        if let Some(order) = order {
            query_params.push(format!("order={}", order));
        }
        if let Some(after) = after {
            query_params.push(format!("after={}", after));
        }
        if let Some(before) = before {
            query_params.push(format!("before={}", before));
        }

        let endpoint = if query_params.is_empty() {
            "vector_stores".to_string()
        } else {
            format!("vector_stores?{}", query_params.join("&"))
        };

        let url = format!("{}/{}", self.config.get_api_base(), endpoint);
        let client = reqwest::Client::new();
        let mut req = client.get(&url);

        if let Some(api_key) = &self.config.base.api_key {
            req = req.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = req.send().await.map_err(|e| OpenAIError::Network {
            provider: "openai",
            message: e.to_string(),
        })?;

        let response_bytes = response.bytes().await.map_err(|e| OpenAIError::Network {
            provider: "openai",
            message: e.to_string(),
        })?;

        serde_json::from_slice(&response_bytes).map_err(|e| OpenAIError::ResponseParsing {
            provider: "openai",
            message: e.to_string(),
        })
    }

    /// Create real-time session
    pub async fn create_realtime_session(
        &self,
        config: RealtimeSessionConfig,
    ) -> Result<Value, OpenAIError> {
        // Validate configuration
        OpenAIRealtimeUtils::validate_session_config(&config).map_err(|e| {
            OpenAIError::InvalidRequest {
                provider: "openai",
                message: e.to_string(),
            }
        })?;

        // Real-time API uses WebSocket, this is a simplified version
        // In practice, this would establish a WebSocket connection
        Ok(serde_json::json!({
            "session_id": "session_123",
            "status": "connected",
            "config": config
        }))
    }

    /// Advanced chat completion with structured outputs and reasoning
    pub async fn advanced_chat_completion(
        &self,
        request: AdvancedChatRequest,
    ) -> Result<super::advanced_chat::AdvancedChatResponse, OpenAIError> {
        // Validate advanced request
        AdvancedChatUtils::validate_request(&request).map_err(|e| OpenAIError::InvalidRequest {
            provider: "openai",
            message: e.to_string(),
        })?;

        // Execute request
        let url = format!("{}/chat/completions", self.config.get_api_base());
        let request_value =
            serde_json::to_value(request).map_err(|e| OpenAIError::InvalidRequest {
                provider: "openai",
                message: e.to_string(),
            })?;

        let headers = self.get_request_headers();
        let body = Some(request_value);

        let response = self
            .pool_manager
            .execute_request(&url, HttpMethod::POST, headers, body)
            .await
            .map_err(|e| OpenAIError::Network {
                provider: "openai",
                message: e.to_string(),
            })?;

        let response_bytes = response.bytes().await.map_err(|e| OpenAIError::Network {
            provider: "openai",
            message: e.to_string(),
        })?;

        serde_json::from_slice(&response_bytes).map_err(|e| OpenAIError::ResponseParsing {
            provider: "openai",
            message: e.to_string(),
        })
    }

    /// Get model capabilities for advanced features
    pub fn get_advanced_model_capabilities(
        &self,
        model: &str,
    ) -> super::advanced_chat::ModelCapabilities {
        AdvancedChatUtils::get_model_capabilities(model)
    }

    /// Estimate cost for advanced features
    pub fn estimate_advanced_cost(
        &self,
        model: &str,
        input_tokens: u32,
        output_tokens: u32,
        reasoning_tokens: Option<u32>,
    ) -> Result<f64, OpenAIError> {
        AdvancedChatUtils::estimate_advanced_cost(
            model,
            input_tokens,
            output_tokens,
            reasoning_tokens,
        )
        .map_err(|e| OpenAIError::InvalidRequest {
            provider: "openai",
            message: e.to_string(),
        })
    }
}

/// Error mapper for OpenAI provider
pub struct OpenAIErrorMapper;

impl crate::core::traits::error_mapper::trait_def::ErrorMapper<OpenAIError> for OpenAIErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> OpenAIError {
        // Simple error mapping - in real implementation would parse OpenAI error format
        match status_code {
            401 => OpenAIError::Authentication {
                provider: "openai",
                message: "Invalid API key".to_string(),
            },
            429 => OpenAIError::rate_limit_simple("openai", "Rate limit exceeded"),
            400 => OpenAIError::InvalidRequest {
                provider: "openai",
                message: response_body.to_string(),
            },
            _ => OpenAIError::Other {
                provider: "openai",
                message: format!("HTTP {}: {}", status_code, response_body),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_provider_creation() {
        let mut config = OpenAIConfig::default();
        config.base.api_key = Some("sk-test123".to_string());

        let provider = OpenAIProvider::new(config).await;
        assert!(provider.is_ok());
    }

    #[test]
    fn test_model_support_detection() {
        let provider = OpenAIProvider {
            pool_manager: Arc::new(GlobalPoolManager::default()),
            config: OpenAIConfig::default(),
            model_registry: get_openai_registry(),
        };

        // Test GPT-4 capabilities
        assert!(provider.model_supports_capability("gpt-4", &ProviderCapability::ChatCompletion));
        assert!(provider.model_supports_capability("gpt-4", &ProviderCapability::ToolCalling));

        // Test embedding model
        assert!(!provider.model_supports_capability(
            "text-embedding-ada-002",
            &ProviderCapability::ChatCompletion
        ));
    }
}
