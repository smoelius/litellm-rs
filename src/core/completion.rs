//! Python LiteLLM compatible completion API
//!
//! This module provides a Python LiteLLM-style API for making completion requests.
//! It serves as the main entry point for the library, providing a unified interface
//! to call 100+ LLM APIs using OpenAI format.

use async_trait::async_trait;
use futures::stream::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::OnceCell;
// Import core types from our unified type system
use crate::core::types::{
    ChatMessage, ChatRequest, ChatResponse, RequestContext, Tool, ToolChoice,
};

// Import provider system
use crate::core::providers::{Provider, ProviderRegistry, ProviderType};
use crate::core::streaming::{ChatCompletionChunk, ChatCompletionChunkChoice, ChatCompletionDelta};
use crate::utils::error::{GatewayError, Result};
use tracing::debug;

/// Core completion function - the main entry point for all LLM calls
/// Mimics Python LiteLLM's completion function signature
pub async fn completion(
    model: &str,
    messages: Vec<Message>,
    options: Option<CompletionOptions>,
) -> Result<CompletionResponse> {
    let router = get_global_router().await;
    router
        .complete(model, messages, options.unwrap_or_default())
        .await
}

/// Async version of completion (though all is async in Rust)
pub async fn acompletion(
    model: &str,
    messages: Vec<Message>,
    options: Option<CompletionOptions>,
) -> Result<CompletionResponse> {
    completion(model, messages, options).await
}

/// Streaming completion function
pub async fn completion_stream(
    model: &str,
    messages: Vec<Message>,
    options: Option<CompletionOptions>,
) -> Result<CompletionStream> {
    let router = get_global_router().await;
    router
        .complete_stream(model, messages, options.unwrap_or_default())
        .await
}

/// Unified message format (OpenAI compatible) - just re-export the core type
pub type Message = ChatMessage;

// Re-export types with proper paths (no duplicate imports)
pub use crate::core::types::{MessageContent, MessageRole};

/// Content part for multimodal messages (re-export from core types)
pub use crate::core::types::ContentPart;

/// Tool call structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub r#type: String,
    pub function: FunctionCall,
}

/// Function call structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

/// Completion options - Python LiteLLM compatible
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompletionOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    #[serde(default)]
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<u32>,

    // Python LiteLLM compatibility fields
    /// Custom API base URL - overrides provider's default base URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_base: Option<String>,

    /// Custom API key - overrides provider's default API key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,

    /// Custom organization ID (for OpenAI)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization: Option<String>,

    /// Custom API version (for Azure)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_version: Option<String>,

    /// Custom headers to add to the request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,

    /// Timeout in seconds for the request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,

    #[serde(flatten)]
    pub extra_params: HashMap<String, serde_json::Value>,
}

/// Completion response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<Choice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
}

/// Choice in completion response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    pub index: u32,
    pub message: Message,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<FinishReason>,
}

/// Usage statistics (re-export from core types)
pub type Usage = crate::core::types::responses::Usage;

/// Finish reason enumeration (re-export from core types)
pub type FinishReason = crate::core::types::responses::FinishReason;

/// Streaming response type
pub type CompletionStream = Box<
    dyn futures::Stream<Item = Result<crate::core::streaming::ChatCompletionChunk>>
        + Send
        + Unpin
        + 'static,
>;

/// LiteLLM Error type
pub type LiteLLMError = GatewayError;

/// Router trait for handling completion requests
#[async_trait]
pub trait Router: Send + Sync {
    async fn complete(
        &self,
        model: &str,
        messages: Vec<Message>,
        options: CompletionOptions,
    ) -> Result<CompletionResponse>;

    async fn complete_stream(
        &self,
        model: &str,
        messages: Vec<Message>,
        options: CompletionOptions,
    ) -> Result<CompletionStream>;
}

/// Default router implementation using the provider registry
pub struct DefaultRouter {
    provider_registry: Arc<ProviderRegistry>,
}

impl DefaultRouter {
    /// Helper function to find and select a provider by name with model prefix stripping
    fn select_provider_by_name<'a>(
        providers: &'a [&'a crate::core::providers::Provider],
        provider_name: &str,
        original_model: &str,
        prefix: &str,
        chat_request: &ChatRequest,
    ) -> Option<(&'a crate::core::providers::Provider, ChatRequest)> {
        if !original_model.starts_with(prefix) {
            return None;
        }

        let actual_model = original_model
            .strip_prefix(prefix)
            .unwrap_or(original_model);

        debug!(
            provider = provider_name,
            model = %actual_model,
            "Using static {} provider", provider_name
        );

        for provider in providers.iter() {
            if provider.name() == provider_name {
                let mut updated_request = chat_request.clone();
                updated_request.model = actual_model.to_string();
                return Some((provider, updated_request));
            }
        }

        None
    }

    pub async fn new() -> Result<Self> {
        let mut provider_registry = ProviderRegistry::new();

        // Add OpenAI provider if API key is available
        if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
            use crate::core::providers::base::BaseConfig;
            use crate::core::providers::openai::OpenAIProvider;
            use crate::core::providers::openai::config::OpenAIConfig;

            // Create OpenAI provider config
            let config = OpenAIConfig {
                base: BaseConfig {
                    api_key: Some(api_key),
                    api_base: Some("https://api.openai.com/v1".to_string()),
                    timeout: 60,
                    max_retries: 3,
                    headers: Default::default(),
                    organization: std::env::var("OPENAI_ORGANIZATION").ok(),
                    api_version: None,
                },
                organization: std::env::var("OPENAI_ORGANIZATION").ok(),
                project: None,
                model_mappings: Default::default(),
                features: Default::default(),
            };

            // Create and register OpenAI provider
            if let Ok(openai_provider) = OpenAIProvider::new(config).await {
                provider_registry.register(Provider::OpenAI(openai_provider));
            }
        }

        // Add OpenRouter provider if API key is available
        if let Ok(api_key) = std::env::var("OPENROUTER_API_KEY") {
            use crate::core::providers::openrouter::{OpenRouterConfig, OpenRouterProvider};

            // Clean the API key to remove any whitespace or newlines
            let api_key = api_key.trim().to_string();

            // Create OpenRouter provider config
            let config = OpenRouterConfig {
                api_key,
                base_url: "https://openrouter.ai/api/v1".to_string(),
                site_url: std::env::var("OPENROUTER_HTTP_REFERER").ok(),
                site_name: std::env::var("OPENROUTER_X_TITLE").ok(),
                timeout_seconds: 60,
                max_retries: 3,
                extra_params: Default::default(),
            };

            // Create and register OpenRouter provider
            if let Ok(openrouter_provider) = OpenRouterProvider::new(config) {
                provider_registry.register(Provider::OpenRouter(openrouter_provider));
            }
        }

        // Add Anthropic provider if API key is available
        if let Ok(api_key) = std::env::var("ANTHROPIC_API_KEY") {
            use crate::core::providers::anthropic::{AnthropicConfig, AnthropicProvider};

            let config = AnthropicConfig::new(api_key)
                .with_base_url("https://api.anthropic.com")
                .with_experimental(false);

            let anthropic_provider = AnthropicProvider::new(config)?;
            provider_registry.register(Provider::Anthropic(anthropic_provider));
        }

        // Azure provider registration temporarily disabled - needs migration to new system
        // TODO: Re-enable once Azure provider is fully migrated from BaseLLM to LLMProvider

        // Add VertexAI provider if service account is available
        if std::env::var("GOOGLE_APPLICATION_CREDENTIALS").is_ok() {
            use crate::core::providers::vertex_ai::{
                VertexAIProvider, VertexAIProviderConfig, VertexCredentials,
            };

            let config = VertexAIProviderConfig {
                project_id: std::env::var("GOOGLE_PROJECT_ID")
                    .unwrap_or_else(|_| "default-project".to_string()),
                location: std::env::var("GOOGLE_LOCATION")
                    .unwrap_or_else(|_| "us-central1".to_string()),
                api_version: "v1".to_string(),
                credentials: VertexCredentials::ApplicationDefault,
                api_base: None,
                timeout_seconds: 60,
                max_retries: 3,
                enable_experimental: false,
            };

            if let Ok(vertex_provider) = VertexAIProvider::new(config).await {
                provider_registry.register(Provider::VertexAI(vertex_provider));
            }
        }

        // Add DeepSeek provider if API key is available
        if let Ok(_api_key) = std::env::var("DEEPSEEK_API_KEY") {
            use crate::core::providers::deepseek::{DeepSeekConfig, DeepSeekProvider};

            let config = DeepSeekConfig::from_env();

            if let Ok(deepseek_provider) = DeepSeekProvider::new(config) {
                provider_registry.register(Provider::DeepSeek(deepseek_provider));
            }
        }

        // Add Groq provider if API key is available
        if let Ok(api_key) = std::env::var("GROQ_API_KEY") {
            use crate::core::providers::groq::{GroqConfig, GroqProvider};

            let config = GroqConfig {
                api_key: Some(api_key),
                ..Default::default()
            };

            if let Ok(groq_provider) = GroqProvider::new(config).await {
                provider_registry.register(Provider::Groq(groq_provider));
            }
        }

        Ok(Self {
            provider_registry: Arc::new(provider_registry),
        })
    }

    /// Dynamic provider creation (Python LiteLLM style)
    /// Creates providers on-demand based on model name and provided options
    async fn try_dynamic_provider_creation(
        &self,
        chat_request: &ChatRequest,
        context: RequestContext,
        options: &CompletionOptions,
    ) -> Result<Option<CompletionResponse>> {
        let model = &chat_request.model;

        // Only proceed if user provided an API key
        let api_key = match &options.api_key {
            Some(key) => key.clone(),
            None => return Ok(None), // No dynamic creation without API key
        };

        // Determine provider type from model name
        let (provider_type, actual_model, api_base) = if model.starts_with("openrouter/") {
            let actual_model = model.strip_prefix("openrouter/").unwrap_or(model);
            let api_base = options
                .api_base
                .clone()
                .unwrap_or_else(|| "https://openrouter.ai/api/v1".to_string());
            ("openrouter", actual_model, api_base)
        } else if model.starts_with("anthropic/") {
            let actual_model = model.strip_prefix("anthropic/").unwrap_or(model);
            let api_base = options
                .api_base
                .clone()
                .unwrap_or_else(|| "https://api.anthropic.com".to_string());
            ("anthropic", actual_model, api_base)
        } else if model.starts_with("deepseek/") {
            let actual_model = model.strip_prefix("deepseek/").unwrap_or(model);
            let api_base = options
                .api_base
                .clone()
                .unwrap_or_else(|| "https://api.deepseek.com".to_string());
            ("deepseek", actual_model, api_base)
        } else if model.starts_with("azure_ai/") || model.starts_with("azure-ai/") {
            let actual_model = model
                .strip_prefix("azure_ai/")
                .or_else(|| model.strip_prefix("azure-ai/"))
                .unwrap_or(model);
            let api_base = options
                .api_base
                .clone()
                .or_else(|| std::env::var("AZURE_AI_API_BASE").ok())
                .unwrap_or_else(|| "https://api.azure.com".to_string());
            ("azure_ai", actual_model, api_base)
        } else if model.starts_with("openai/") {
            let actual_model = model.strip_prefix("openai/").unwrap_or(model);
            let api_base = options
                .api_base
                .clone()
                .unwrap_or_else(|| "https://api.openai.com/v1".to_string());
            ("openai", actual_model, api_base)
        } else {
            // For models without provider prefix, try to infer or use custom api_base
            if let Some(api_base) = &options.api_base {
                ("openai-compatible", model.as_str(), api_base.clone())
            } else {
                return Ok(None); // Can't create dynamic provider without api_base
            }
        };

        debug!(
            provider_type = %provider_type,
            model = %actual_model,
            "Creating dynamic provider for model"
        );

        // Create dynamic provider based on type
        let response = match provider_type {
            "openrouter" => {
                self.create_dynamic_openrouter(
                    actual_model,
                    &api_key,
                    &api_base,
                    chat_request,
                    context,
                )
                .await?
            }
            "anthropic" => {
                self.create_dynamic_anthropic(
                    actual_model,
                    &api_key,
                    &api_base,
                    chat_request,
                    context,
                )
                .await?
            }
            "deepseek" => {
                self.create_dynamic_openai_compatible(
                    actual_model,
                    &api_key,
                    &api_base,
                    chat_request,
                    context,
                    "DeepSeek",
                )
                .await?
            }
            "azure_ai" => {
                self.create_dynamic_azure_ai(
                    actual_model,
                    &api_key,
                    &api_base,
                    chat_request,
                    context,
                )
                .await?
            }
            "openai" => {
                self.create_dynamic_openai_compatible(
                    actual_model,
                    &api_key,
                    &api_base,
                    chat_request,
                    context,
                    "OpenAI",
                )
                .await?
            }
            "openai-compatible" => {
                self.create_dynamic_openai_compatible(
                    actual_model,
                    &api_key,
                    &api_base,
                    chat_request,
                    context,
                    "OpenAI-Compatible",
                )
                .await?
            }
            _ => return Ok(None),
        };

        Ok(Some(response))
    }

    /// Create dynamic OpenRouter provider
    async fn create_dynamic_openrouter(
        &self,
        model: &str,
        api_key: &str,
        api_base: &str,
        chat_request: &ChatRequest,
        context: RequestContext,
    ) -> Result<CompletionResponse> {
        use crate::core::providers::openrouter::{OpenRouterConfig, OpenRouterProvider};
        use crate::core::traits::LLMProvider;

        let config = OpenRouterConfig {
            api_key: api_key.to_string(),
            base_url: api_base.to_string(),
            site_url: None, // Could be extracted from options if needed
            site_name: None,
            timeout_seconds: 60,
            max_retries: 3,
            extra_params: Default::default(),
        };

        let provider = OpenRouterProvider::new(config).map_err(|e| {
            GatewayError::internal(format!(
                "Failed to create dynamic OpenRouter provider: {}",
                e
            ))
        })?;

        let mut updated_request = chat_request.clone();
        updated_request.model = model.to_string();

        let response = provider
            .chat_completion(updated_request, context)
            .await
            .map_err(|e| {
                GatewayError::internal(format!("Dynamic OpenRouter provider error: {}", e))
            })?;

        convert_from_chat_completion_response(response)
    }

    /// Create dynamic Anthropic provider
    async fn create_dynamic_anthropic(
        &self,
        model: &str,
        api_key: &str,
        api_base: &str,
        chat_request: &ChatRequest,
        context: RequestContext,
    ) -> Result<CompletionResponse> {
        use crate::core::providers::anthropic::{AnthropicConfig, AnthropicProvider};
        use crate::core::traits::LLMProvider;

        let config = AnthropicConfig::new(api_key)
            .with_base_url(api_base)
            .with_experimental(false);

        let provider = AnthropicProvider::new(config)?;

        let mut updated_request = chat_request.clone();
        updated_request.model = model.to_string();

        let response = LLMProvider::chat_completion(&provider, updated_request, context)
            .await
            .map_err(|e| {
                GatewayError::internal(format!("Dynamic Anthropic provider error: {}", e))
            })?;

        convert_from_chat_completion_response(response)
    }

    /// Create dynamic OpenAI-compatible provider (works for OpenAI, DeepSeek, and other compatible APIs)
    async fn create_dynamic_openai_compatible(
        &self,
        model: &str,
        api_key: &str,
        api_base: &str,
        chat_request: &ChatRequest,
        context: RequestContext,
        provider_name: &str,
    ) -> Result<CompletionResponse> {
        use crate::core::providers::base::BaseConfig;
        use crate::core::providers::openai::OpenAIProvider;
        use crate::core::providers::openai::config::OpenAIConfig;
        use crate::core::traits::LLMProvider;

        let config = OpenAIConfig {
            base: BaseConfig {
                api_key: Some(api_key.to_string()),
                api_base: Some(api_base.to_string()),
                timeout: 60,
                max_retries: 3,
                headers: Default::default(),
                organization: None,
                api_version: None,
            },
            organization: None,
            project: None,
            model_mappings: Default::default(),
            features: Default::default(),
        };

        let provider = OpenAIProvider::new(config).await.map_err(|e| {
            GatewayError::internal(format!(
                "Failed to create dynamic {} provider: {}",
                provider_name, e
            ))
        })?;

        let mut updated_request = chat_request.clone();
        updated_request.model = model.to_string();

        let response = provider
            .chat_completion(updated_request, context)
            .await
            .map_err(|e| {
                GatewayError::internal(format!("Dynamic {} provider error: {}", provider_name, e))
            })?;

        convert_from_chat_completion_response(response)
    }

    /// Create dynamic Azure AI provider
    async fn create_dynamic_azure_ai(
        &self,
        model: &str,
        api_key: &str,
        api_base: &str,
        chat_request: &ChatRequest,
        context: RequestContext,
    ) -> Result<CompletionResponse> {
        use crate::core::providers::azure_ai::{AzureAIConfig, AzureAIProvider};
        use crate::core::traits::LLMProvider;

        let mut config = AzureAIConfig::new("azure_ai");
        config.base.api_key = Some(api_key.to_string());
        config.base.api_base = Some(api_base.to_string());

        // Also check environment variables
        if config.base.api_key.is_none() {
            if let Ok(key) = std::env::var("AZURE_AI_API_KEY") {
                config.base.api_key = Some(key);
            }
        }
        if config.base.api_base.is_none() {
            if let Ok(base) = std::env::var("AZURE_AI_API_BASE") {
                config.base.api_base = Some(base);
            }
        }

        let provider = AzureAIProvider::new(config).map_err(|e| {
            GatewayError::internal(format!("Failed to create dynamic Azure AI provider: {}", e))
        })?;

        let mut updated_request = chat_request.clone();
        updated_request.model = model.to_string();

        let response = provider
            .chat_completion(updated_request, context)
            .await
            .map_err(|e| {
                GatewayError::internal(format!("Dynamic Azure AI provider error: {}", e))
            })?;

        convert_from_chat_completion_response(response)
    }
}

#[async_trait]
impl Router for DefaultRouter {
    async fn complete(
        &self,
        model: &str,
        messages: Vec<Message>,
        options: CompletionOptions,
    ) -> Result<CompletionResponse> {
        // Convert to internal types
        let chat_messages = convert_messages_to_chat_messages(messages);
        let chat_request =
            convert_to_chat_completion_request(model, chat_messages, options.clone())?;

        // Create request context with override parameters from options
        let mut context = RequestContext::new();

        // Check for dynamic provider configuration overrides
        if let Some(api_base) = &options.api_base {
            context.metadata.insert(
                "api_base_override".to_string(),
                serde_json::Value::String(api_base.clone()),
            );
        }

        if let Some(api_key) = &options.api_key {
            context.metadata.insert(
                "api_key_override".to_string(),
                serde_json::Value::String(api_key.clone()),
            );
        }

        if let Some(organization) = &options.organization {
            context.metadata.insert(
                "organization_override".to_string(),
                serde_json::Value::String(organization.clone()),
            );
        }

        if let Some(api_version) = &options.api_version {
            context.metadata.insert(
                "api_version_override".to_string(),
                serde_json::Value::String(api_version.clone()),
            );
        }

        if let Some(headers) = &options.headers {
            context.metadata.insert(
                "headers_override".to_string(),
                serde_json::to_value(headers).unwrap_or_default(),
            );
        }

        if let Some(timeout) = options.timeout {
            context.metadata.insert(
                "timeout_override".to_string(),
                serde_json::Value::Number(serde_json::Number::from(timeout)),
            );
        }

        // Check if user provided custom api_base (Python LiteLLM compatibility)
        if let Some(api_base) = &options.api_base {
            // When api_base is provided, create a temporary OpenAI-compatible provider
            // This matches Python LiteLLM behavior for custom endpoints
            use crate::core::providers::base::BaseConfig;
            use crate::core::providers::openai::OpenAIProvider;
            use crate::core::providers::openai::config::OpenAIConfig;
            use crate::core::traits::LLMProvider;

            let api_key = options
                .api_key
                .clone()
                .or_else(|| std::env::var("OPENAI_API_KEY").ok())
                .unwrap_or_else(|| "dummy-key-for-local".to_string());

            let config = OpenAIConfig {
                base: BaseConfig {
                    api_key: Some(api_key),
                    api_base: Some(api_base.clone()),
                    timeout: options.timeout.unwrap_or(60),
                    max_retries: 3,
                    headers: options.headers.clone().unwrap_or_default(),
                    organization: options.organization.clone(),
                    api_version: None,
                },
                organization: options.organization.clone(),
                project: None,
                model_mappings: Default::default(),
                features: Default::default(),
            };

            // Create temporary provider with custom base URL
            match OpenAIProvider::new(config).await {
                Ok(temp_provider) => {
                    // Use the temporary provider directly
                    let response = temp_provider
                        .chat_completion(chat_request, context)
                        .await
                        .map_err(|e| GatewayError::internal(format!("Provider error: {}", e)))?;
                    return convert_from_chat_completion_response(response);
                }
                Err(e) => {
                    return Err(GatewayError::internal(format!(
                        "Failed to create provider with custom api_base: {}",
                        e
                    )));
                }
            }
        }

        // Dynamic provider creation (Python LiteLLM style)
        // Try dynamic creation first, fallback to static registry
        if let Some(response) = self
            .try_dynamic_provider_creation(&chat_request, context.clone(), &options)
            .await?
        {
            return Ok(response);
        }

        // Fallback to static provider registry
        let providers = self.provider_registry.all();

        // Check if model explicitly specifies a provider - using helper function
        let mut selected_provider = Self::select_provider_by_name(
            &providers,
            "openrouter",
            model,
            "openrouter/",
            &chat_request,
        )
        .or_else(|| {
            Self::select_provider_by_name(&providers, "deepseek", model, "deepseek/", &chat_request)
        })
        .or_else(|| {
            Self::select_provider_by_name(
                &providers,
                "anthropic",
                model,
                "anthropic/",
                &chat_request,
            )
        })
        .or_else(|| {
            Self::select_provider_by_name(&providers, "azure_ai", model, "azure_ai/", &chat_request)
        })
        .or_else(|| {
            Self::select_provider_by_name(&providers, "groq", model, "groq/", &chat_request)
        });

        // Handle special cases that don't follow the standard pattern
        if selected_provider.is_none() {
            if model.starts_with("openai/") || model.starts_with("azure/") {
                for provider in providers.iter() {
                    if provider.provider_type() == ProviderType::OpenAI
                        && provider.supports_model(model)
                    {
                        selected_provider = Some((provider, chat_request.clone()));
                        break;
                    }
                }
            } else {
                // No explicit provider, try to find one that supports the model
                for provider in providers.iter() {
                    if provider.supports_model(model) {
                        selected_provider = Some((provider, chat_request.clone()));
                        break;
                    }
                }
            }
        }

        // Use static provider if found
        if let Some((provider, request)) = selected_provider {
            let response = provider.chat_completion(request, context).await?;
            return convert_from_chat_completion_response(response);
        }

        Err(GatewayError::internal(
            "No suitable provider found for model",
        ))
    }

    async fn complete_stream(
        &self,
        model: &str,
        messages: Vec<Message>,
        options: CompletionOptions,
    ) -> Result<CompletionStream> {
        // Convert to internal types
        let chat_messages = convert_messages_to_chat_messages(messages);
        let mut chat_request =
            convert_to_chat_completion_request(model, chat_messages, options.clone())?;
        chat_request.stream = true;

        // Create request context
        let context = RequestContext::new();

        // Find provider (reuse the same logic as complete)
        let providers = self.provider_registry.all();

        // Check if model explicitly specifies a provider
        let selected_provider = Self::select_provider_by_name(
            &providers,
            "openrouter",
            model,
            "openrouter/",
            &chat_request,
        )
        .or_else(|| {
            Self::select_provider_by_name(&providers, "deepseek", model, "deepseek/", &chat_request)
        })
        .or_else(|| {
            Self::select_provider_by_name(
                &providers,
                "anthropic",
                model,
                "anthropic/",
                &chat_request,
            )
        })
        .or_else(|| {
            Self::select_provider_by_name(&providers, "azure_ai", model, "azure_ai/", &chat_request)
        })
        .or_else(|| {
            Self::select_provider_by_name(&providers, "groq", model, "groq/", &chat_request)
        });

        // Get the provider and execute streaming
        if let Some((provider, request)) = selected_provider {
            let stream = provider
                .chat_completion_stream(request, context)
                .await
                .map_err(|e| GatewayError::internal(format!("Streaming error: {}", e)))?;

            // Convert ChatChunk stream to ChatCompletionChunk stream
            let converted_stream = stream.map(|result| {
                result
                    .map(convert_chat_chunk_to_completion_chunk)
                    .map_err(|e| GatewayError::internal(format!("Stream chunk error: {}", e)))
            });

            return Ok(Box::new(Box::pin(converted_stream)));
        }

        Err(GatewayError::internal(
            "No suitable provider found for streaming",
        ))
    }
}

/// Convert ChatChunk (from provider) to ChatCompletionChunk (for API)
fn convert_chat_chunk_to_completion_chunk(
    chunk: crate::core::types::ChatChunk,
) -> ChatCompletionChunk {
    ChatCompletionChunk {
        id: chunk.id,
        object: chunk.object,
        created: chunk.created as u64,
        model: chunk.model,
        system_fingerprint: chunk.system_fingerprint,
        choices: chunk
            .choices
            .into_iter()
            .map(|c| ChatCompletionChunkChoice {
                index: c.index,
                delta: ChatCompletionDelta {
                    role: c.delta.role,
                    content: c.delta.content,
                    tool_calls: None, // Simplified for now
                },
                finish_reason: c.finish_reason.map(|fr| format!("{:?}", fr).to_lowercase()),
                logprobs: None,
            })
            .collect(),
        usage: chunk.usage.map(|u| crate::core::models::openai::Usage {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
            prompt_tokens_details: None,
            completion_tokens_details: None,
        }),
    }
}

/// Fallback router for when initialization fails
pub struct ErrorRouter {
    error: String,
}

#[async_trait]
impl Router for ErrorRouter {
    async fn complete(
        &self,
        _model: &str,
        _messages: Vec<Message>,
        _options: CompletionOptions,
    ) -> Result<CompletionResponse> {
        Err(GatewayError::internal(format!(
            "Router initialization failed: {}",
            self.error
        )))
    }

    async fn complete_stream(
        &self,
        _model: &str,
        _messages: Vec<Message>,
        _options: CompletionOptions,
    ) -> Result<CompletionStream> {
        Err(GatewayError::internal(format!(
            "Router initialization failed: {}",
            self.error
        )))
    }
}

/// Global router instance
static GLOBAL_ROUTER: OnceCell<Box<dyn Router>> = OnceCell::const_new();

/// Get or initialize the global router
async fn get_global_router() -> &'static Box<dyn Router> {
    GLOBAL_ROUTER
        .get_or_init(|| async {
            match DefaultRouter::new().await {
                Ok(router) => Box::new(router) as Box<dyn Router>,
                Err(e) => Box::new(ErrorRouter {
                    error: e.to_string(),
                }) as Box<dyn Router>,
            }
        })
        .await
}

/// Helper function to create user message
pub fn user_message(content: impl Into<String>) -> Message {
    use crate::core::types::{MessageContent, MessageRole};
    ChatMessage {
        role: MessageRole::User,
        content: Some(MessageContent::Text(content.into())),
        name: None,
        tool_calls: None,
        tool_call_id: None,
        function_call: None,
    }
}

/// Helper function to create system message
pub fn system_message(content: impl Into<String>) -> Message {
    use crate::core::types::{MessageContent, MessageRole};
    ChatMessage {
        role: MessageRole::System,
        content: Some(MessageContent::Text(content.into())),
        name: None,
        tool_calls: None,
        tool_call_id: None,
        function_call: None,
    }
}

/// Helper function to create assistant message
pub fn assistant_message(content: impl Into<String>) -> Message {
    use crate::core::types::{MessageContent, MessageRole};
    ChatMessage {
        role: MessageRole::Assistant,
        content: Some(MessageContent::Text(content.into())),
        name: None,
        tool_calls: None,
        tool_call_id: None,
        function_call: None,
    }
}

// Internal conversion functions

fn convert_messages_to_chat_messages(messages: Vec<Message>) -> Vec<ChatMessage> {
    // Since Message is now an alias for ChatMessage, this is just a no-op
    messages
}

fn convert_to_chat_completion_request(
    model: &str,
    messages: Vec<ChatMessage>,
    options: CompletionOptions,
) -> Result<ChatRequest> {
    Ok(ChatRequest {
        model: model.to_string(),
        messages,
        temperature: options.temperature,
        max_tokens: options.max_tokens,
        max_completion_tokens: None,
        top_p: options.top_p,
        frequency_penalty: options.frequency_penalty,
        presence_penalty: options.presence_penalty,
        stop: options.stop,
        stream: options.stream,
        tools: None,       // Will implement tool conversion later
        tool_choice: None, // Will implement tool choice conversion later
        parallel_tool_calls: None,
        response_format: None,
        user: options.user,
        seed: options.seed,
        n: options.n,
        logit_bias: None,
        functions: None,
        function_call: None,
        logprobs: options.logprobs,
        top_logprobs: options.top_logprobs,
        extra_params: options.extra_params,
    })
}

fn convert_from_chat_completion_response(response: ChatResponse) -> Result<CompletionResponse> {
    let choices = response
        .choices
        .into_iter()
        .map(|choice| Choice {
            index: choice.index,
            message: choice.message,             // Same type already
            finish_reason: choice.finish_reason, // Same type already
        })
        .collect();

    Ok(CompletionResponse {
        id: response.id,
        object: response.object,
        created: response.created,
        model: response.model,
        choices,
        usage: response.usage, // Same type already
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let msg = user_message("Hello, world!");
        assert_eq!(msg.role, MessageRole::User);
        if let Some(MessageContent::Text(content)) = msg.content {
            assert_eq!(content, "Hello, world!");
        } else {
            panic!("Expected text content");
        }
    }

    #[test]
    fn test_completion_options_default() {
        let options = CompletionOptions::default();
        assert!(!options.stream);
        assert_eq!(options.extra_params.len(), 0);
    }
}
