//! New Router implementation using Provider enum
//! 
//! This is the new router that uses the Rust-idiomatic enum-based Provider system

use crate::config::ProviderConfig;
use crate::core::types::{
    requests::{ChatRequest, CompletionRequest, EmbeddingRequest, ImageGenerationRequest},
    responses::{ChatResponse, CompletionResponse, EmbeddingResponse, ImageGenerationResponse, ChatChunk},
    common::{RequestContext, ModelInfo, ProviderCapability},
};
use crate::core::providers::{Provider, ProviderType, ProviderError, UnifiedProviderError, ProviderRegistry};
use crate::storage::StorageLayer;
use crate::utils::error::{Result, GatewayError};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, debug};

/// Modern router using enum-based providers
pub struct RouterV2 {
    /// Storage layer
    storage: Arc<StorageLayer>,
    /// Consolidated router data - single lock for registry and model cache
    router_data: Arc<RwLock<RouterData>>,
}

/// Consolidated router data - single lock for provider registry and model cache
struct RouterData {
    /// Provider registry
    registry: ProviderRegistry,
    /// Model to provider mapping cache
    model_cache: HashMap<String, Vec<String>>,
}

impl RouterV2 {
    /// Create new router
    pub async fn new(storage: Arc<StorageLayer>) -> Result<Self> {
        Ok(Self {
            storage,
            router_data: Arc::new(RwLock::new(RouterData {
                registry: ProviderRegistry::new(),
                model_cache: HashMap::new(),
            })),
        })
    }

    /// Register a provider
    pub async fn register_provider(&self, provider: Provider) -> Result<()> {
        let name = provider.name().to_string();
        info!("Registering provider: {}", name);

        let mut data = self.router_data.write().await;
        data.registry.register(provider);
        // Clear model cache when providers change
        data.model_cache.clear();

        Ok(())
    }
    
    /// Initialize providers from config
    pub async fn init_providers(&self, configs: Vec<ProviderConfig>) -> Result<()> {
        for config in configs {
            match self.create_provider_from_config(config).await {
                Ok(provider) => {
                    self.register_provider(provider).await?;
                }
                Err(e) => {
                    warn!("Failed to create provider: {}", e);
                }
            }
        }
        
        Ok(())
    }
    
    /// Create provider from config
    async fn create_provider_from_config(&self, config: ProviderConfig) -> Result<Provider> {
        use crate::core::providers::*;
        
        // Extract provider type from config
        let provider_type = match config.provider_type.as_str() {
            "openai" => ProviderType::OpenAI,
            "anthropic" => ProviderType::Anthropic,
            "azure" => ProviderType::Azure,
            "mistral" => ProviderType::Mistral,
            "deepseek" => ProviderType::DeepSeek,
            "moonshot" => ProviderType::Moonshot,
            _ => return Err(GatewayError::Configuration(
                format!("Unknown provider type: {}", config.provider_type)
            )),
        };
        
        // Create provider based on type
        // Note: In real implementation, you'd parse config and create appropriate provider
        match provider_type {
            ProviderType::OpenAI => {
                // let openai = openai::OpenAIProvider::from_config(config)?;
                // Ok(Provider::OpenAI(openai))
                Err(GatewayError::Configuration("OpenAI provider creation not implemented".into()))
            }
            ProviderType::Anthropic => {
                // let anthropic = anthropic::AnthropicProvider::from_config(config)?;
                // Ok(Provider::Anthropic(anthropic))
                Err(GatewayError::Configuration("Anthropic provider creation not implemented".into()))
            }
            _ => Err(GatewayError::Configuration(
                format!("Provider type {:?} not yet implemented", provider_type)
            )),
        }
    }
    
    /// Route chat request to best provider
    pub async fn route_chat(
        &self,
        request: ChatRequest,
        context: RequestContext,
    ) -> Result<ChatResponse> {
        let model = request.model.clone();
        let provider = self.select_provider_for_model(&model).await?;
        
        debug!("Routing chat request for model {} to provider {}", model, provider.name());
        
        provider.chat_completion(request, context)
            .await
            .map_err(|e| GatewayError::Provider(format!("{}", e)))
    }
    
    /// Route streaming chat request
    pub async fn route_chat_stream(
        &self,
        request: ChatRequest,
        context: RequestContext,
    ) -> Result<impl futures::Stream<Item = Result<ChatChunk>>> {
        let model = request.model.clone();
        let provider = self.select_provider_for_model(&model).await?;
        
        debug!("Routing streaming chat for model {} to provider {}", model, provider.name());
        
        let stream = provider.chat_completion_stream(request, context)
            .await
            .map_err(|e| GatewayError::Provider(format!("{}", e)))?;
        
        // Wrap stream to convert errors
        let wrapped_stream = futures::stream::StreamExt::map(stream, |result| {
            result.map_err(|e| GatewayError::Provider(format!("{}", e)))
        });
        
        Ok(wrapped_stream)
    }
    
    /// Route embeddings request
    pub async fn route_embeddings(
        &self,
        request: EmbeddingRequest,
        context: RequestContext,
    ) -> Result<EmbeddingResponse> {
        let model = request.model.clone();
        let provider = self.select_provider_for_embeddings(&model).await?;
        
        debug!("Routing embeddings for model {} to provider {}", model, provider.name());
        
        provider.create_embeddings(request, context)
            .await
            .map_err(|e| GatewayError::Provider(format!("{}", e)))
    }
    
    /// Route image generation request
    pub async fn route_images(
        &self,
        request: ImageGenerationRequest,
        context: RequestContext,
    ) -> Result<ImageGenerationResponse> {
        let model = request.model.as_ref().unwrap_or(&"dall-e-3".to_string()).clone();
        let provider = self.select_provider_for_images(&model).await?;
        
        debug!("Routing image generation to provider {}", provider.name());
        
        provider.create_images(request, context)
            .await
            .map_err(|e| GatewayError::Provider(format!("{}", e)))
    }
    
    /// Select best provider for model
    async fn select_provider_for_model(&self, model: &str) -> Result<Provider> {
        let data = self.router_data.read().await;

        // Check cache first
        if let Some(provider_names) = data.model_cache.get(model) {
            if let Some(name) = provider_names.first() {
                if let Some(provider) = data.registry.get(name) {
                    return Ok(provider.clone());
                }
            }
        }

        // Find providers that support this model
        let mut supporting_providers = Vec::new();
        for provider_name in data.registry.list() {
            if let Some(provider) = data.registry.get(&provider_name) {
                if provider.supports_model(model) {
                    supporting_providers.push(provider.clone());
                }
            }
        }

        if supporting_providers.is_empty() {
            return Err(GatewayError::ModelNotFound(format!(
                "No provider supports model: {}", model
            )));
        }

        // For now, just return the first one
        // TODO: Implement smart selection based on cost, latency, etc.
        Ok(supporting_providers[0].clone())
    }
    
    /// Select provider for embeddings
    async fn select_provider_for_embeddings(&self, model: &str) -> Result<Provider> {
        let data = self.router_data.read().await;

        for provider_name in data.registry.list() {
            if let Some(provider) = data.registry.get(&provider_name) {
                let capabilities = provider.capabilities();
                if capabilities.contains(&ProviderCapability::Embeddings) {
                    if provider.supports_model(model) {
                        return Ok(provider.clone());
                    }
                }
            }
        }

        Err(GatewayError::ProviderNotFound(
            "No provider supports embeddings".into()
        ))
    }

    /// Select provider for image generation
    async fn select_provider_for_images(&self, _model: &str) -> Result<Provider> {
        let data = self.router_data.read().await;

        for provider_name in data.registry.list() {
            if let Some(provider) = data.registry.get(&provider_name) {
                let capabilities = provider.capabilities();
                if capabilities.contains(&ProviderCapability::ImageGeneration) {
                    return Ok(provider.clone());
                }
            }
        }

        Err(GatewayError::ProviderNotFound(
            "No provider supports image generation".into()
        ))
    }

    /// Get provider info
    pub async fn get_provider_info(&self, name: &str) -> Result<ProviderInfo> {
        let data = self.router_data.read().await;

        if let Some(provider) = data.registry.get(name) {
            Ok(ProviderInfo {
                name: provider.name().to_string(),
                provider_type: provider.provider_type(),
                capabilities: provider.capabilities().to_vec(),
            })
        } else {
            Err(GatewayError::ProviderNotFound(format!(
                "Provider not found: {}", name
            )))
        }
    }

    /// List all providers
    pub async fn list_providers(&self) -> Vec<ProviderInfo> {
        let data = self.router_data.read().await;
        let mut providers = Vec::new();

        for name in data.registry.list() {
            if let Some(provider) = data.registry.get(&name) {
                providers.push(ProviderInfo {
                    name: provider.name().to_string(),
                    provider_type: provider.provider_type(),
                    capabilities: provider.capabilities().to_vec(),
                });
            }
        }

        providers
    }
}

/// Provider information
#[derive(Debug, Clone)]
pub struct ProviderInfo {
    pub name: String,
    pub provider_type: ProviderType,
    pub capabilities: Vec<ProviderCapability>,
}