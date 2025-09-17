//! Unified pricing service using LiteLLM pricing data format
//!
//! This service loads pricing data from LiteLLM's JSON format and provides
//! unified cost calculation for all AI providers

use crate::utils::error::{GatewayError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};
use tokio::sync::broadcast;
use tracing::{debug, info, warn};

/// LiteLLM compatible model pricing data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Maximum total tokens
    pub max_tokens: Option<u32>,
    /// Maximum input tokens
    pub max_input_tokens: Option<u32>,
    /// Maximum output tokens
    pub max_output_tokens: Option<u32>,
    /// Input cost per token
    pub input_cost_per_token: Option<f64>,
    /// Output cost per token  
    pub output_cost_per_token: Option<f64>,
    /// Input cost per character (for some providers)
    pub input_cost_per_character: Option<f64>,
    /// Output cost per character (for some providers)
    pub output_cost_per_character: Option<f64>,
    /// Cost per second (for time-based providers)
    pub cost_per_second: Option<f64>,
    /// LiteLLM provider name
    pub litellm_provider: String,
    /// Model mode (chat, completion, embedding, etc.)
    pub mode: String,
    /// Supports function calling
    pub supports_function_calling: Option<bool>,
    /// Supports vision
    pub supports_vision: Option<bool>,
    /// Supports streaming
    pub supports_streaming: Option<bool>,
    /// Supports parallel function calling
    pub supports_parallel_function_calling: Option<bool>,
    /// Supports system message
    pub supports_system_message: Option<bool>,
    /// Additional metadata
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Pricing service using LiteLLM data format
#[derive(Debug, Clone)]
pub struct PricingService {
    /// Model pricing data (model_name -> ModelInfo)
    model_data: Arc<RwLock<HashMap<String, ModelInfo>>>,
    /// Last update time
    last_updated: Arc<RwLock<SystemTime>>,
    /// HTTP client for fetching updates
    http_client: reqwest::Client,
    /// Pricing data source URL
    pricing_url: String,
    /// Cache TTL
    cache_ttl: Duration,
    /// Event broadcaster for updates
    event_sender: broadcast::Sender<PricingUpdateEvent>,
}

/// Pricing update event
/// Event for pricing updates
#[derive(Debug, Clone)]
pub struct PricingUpdateEvent {
    /// Type of pricing event that occurred
    pub event_type: PricingEventType,
    /// Model name that was affected
    pub model: String,
    /// Provider name that was affected
    pub provider: String,
    /// When the event occurred
    pub timestamp: SystemTime,
}

/// Types of pricing events that can occur
#[derive(Debug, Clone)]
pub enum PricingEventType {
    /// A new model was added to the pricing data
    ModelAdded,
    /// An existing model's pricing was updated
    ModelUpdated,
    /// A model was removed from the pricing data
    ModelRemoved,
    /// The entire pricing dataset was refreshed
    DataRefreshed,
}

/// Cost calculation result
#[derive(Debug, Clone, Serialize)]
pub struct CostResult {
    /// Cost for input tokens/characters
    pub input_cost: f64,
    /// Cost for output tokens/characters
    pub output_cost: f64,
    /// Total cost (input + output)
    pub total_cost: f64,
    /// Number of input tokens used
    pub input_tokens: u32,
    /// Number of output tokens used
    pub output_tokens: u32,
    /// The model name used for pricing calculation
    pub model: String,
    /// The provider name (e.g., "openai", "anthropic")
    pub provider: String,
    /// The type of cost calculation used
    pub cost_type: CostType,
}

/// Type of cost calculation method
#[derive(Debug, Clone, Serialize)]
pub enum CostType {
    /// Cost calculated based on token count
    TokenBased,
    /// Cost calculated based on character count
    CharacterBased,
    /// Cost calculated based on time duration
    TimeBased,
    /// Custom cost calculation method
    Custom,
}

impl PricingService {
    /// Create a new pricing service
    pub fn new(pricing_url: Option<String>) -> Self {
        let (event_sender, _) = broadcast::channel(1000);

        let service = Self {
            model_data: Arc::new(RwLock::new(HashMap::new())),
            last_updated: Arc::new(RwLock::new(SystemTime::UNIX_EPOCH)),
            http_client: reqwest::Client::new(),
            pricing_url: pricing_url.unwrap_or_else(|| {
                "https://raw.githubusercontent.com/BerriAI/litellm/main/model_prices_and_context_window.json".to_string()
            }),
            cache_ttl: Duration::from_secs(3600), // 1 hour
            event_sender,
        };

        info!("Pricing service initialized with LiteLLM data source");
        service
    }

    /// Start automatic pricing data refresh task
    pub fn start_auto_refresh_task(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        let service = Arc::clone(&self);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(service.cache_ttl);

            loop {
                interval.tick().await;

                if let Err(e) = service.refresh_pricing_data().await {
                    warn!("Auto-refresh pricing data failed: {}", e);
                } else {
                    debug!("Auto-refresh pricing data completed successfully");
                }
            }
        })
    }

    /// Force refresh pricing data immediately
    pub async fn force_refresh(&self) -> Result<()> {
        info!("Force refreshing pricing data");
        self.refresh_pricing_data().await
    }

    /// Initialize pricing data (load from URL or local file)
    pub async fn initialize(&self) -> Result<()> {
        self.refresh_pricing_data().await
    }

    /// Refresh pricing data from source
    pub async fn refresh_pricing_data(&self) -> Result<()> {
        info!("Refreshing pricing data from: {}", self.pricing_url);

        let data = if self.pricing_url.starts_with("http") {
            // Load from URL
            self.load_from_url().await?
        } else {
            // Load from local file
            self.load_from_file().await?
        };

        // Update in-memory data
        {
            let mut model_data = self.model_data.write().unwrap();
            model_data.clear();
            model_data.extend(data);
        }

        // Update timestamp
        {
            let mut last_updated = self.last_updated.write().unwrap();
            *last_updated = SystemTime::now();
        }

        // Send update event
        let _ = self.event_sender.send(PricingUpdateEvent {
            event_type: PricingEventType::DataRefreshed,
            model: "*".to_string(),
            provider: "*".to_string(),
            timestamp: SystemTime::now(),
        });

        info!("Pricing data refreshed successfully");
        Ok(())
    }

    /// Load pricing data from URL
    async fn load_from_url(&self) -> Result<HashMap<String, ModelInfo>> {
        let response = self
            .http_client
            .get(&self.pricing_url)
            .timeout(Duration::from_secs(30))
            .send()
            .await
            .map_err(|e| GatewayError::network(format!("Failed to fetch pricing data: {}", e)))?;

        if !response.status().is_success() {
            return Err(GatewayError::network(format!(
                "HTTP {}: Failed to fetch pricing data",
                response.status()
            )));
        }

        let text = response
            .text()
            .await
            .map_err(|e| GatewayError::network(format!("Failed to read response: {}", e)))?;

        let data: HashMap<String, ModelInfo> = serde_json::from_str(&text)
            .map_err(|e| GatewayError::parsing(format!("Failed to parse pricing JSON: {}", e)))?;

        debug!("Loaded {} models from URL", data.len());
        Ok(data)
    }

    /// Load pricing data from local file
    async fn load_from_file(&self) -> Result<HashMap<String, ModelInfo>> {
        let content = tokio::fs::read_to_string(&self.pricing_url)
            .await
            .map_err(GatewayError::Io)?;

        let data: HashMap<String, ModelInfo> = serde_json::from_str(&content)
            .map_err(|e| GatewayError::parsing(format!("Failed to parse pricing JSON: {}", e)))?;

        debug!("Loaded {} models from file", data.len());
        Ok(data)
    }

    /// Get model information
    pub fn get_model_info(&self, model: &str) -> Option<ModelInfo> {
        let model_data = self.model_data.read().unwrap();
        model_data.get(model).cloned()
    }

    /// Check if pricing data needs refresh
    pub fn needs_refresh(&self) -> bool {
        let last_updated = self.last_updated.read().unwrap();
        SystemTime::now()
            .duration_since(*last_updated)
            .map(|duration| duration > self.cache_ttl)
            .unwrap_or(true)
    }

    /// Calculate completion cost
    pub async fn calculate_completion_cost(
        &self,
        model: &str,
        input_tokens: u32,
        output_tokens: u32,
        prompt: Option<&str>,
        completion: Option<&str>,
        total_time_seconds: Option<f64>,
    ) -> Result<CostResult> {
        // Auto-refresh if needed
        if self.needs_refresh() {
            if let Err(e) = self.refresh_pricing_data().await {
                warn!("Failed to refresh pricing data: {}", e);
            }
        }

        let model_info = self
            .get_model_info(model)
            .ok_or_else(|| GatewayError::not_found(format!("Model not found: {}", model)))?;

        let provider = model_info.litellm_provider.clone();

        // Provider-specific cost calculation
        match provider.as_str() {
            "openai" | "azure" => {
                self.calculate_token_based_cost(model, &model_info, input_tokens, output_tokens)
            }
            "anthropic" => {
                self.calculate_token_based_cost(model, &model_info, input_tokens, output_tokens)
            }
            "google" | "vertex_ai" => self.calculate_google_cost(
                model,
                &model_info,
                input_tokens,
                output_tokens,
                prompt,
                completion,
            ),
            "replicate" | "together_ai" | "baseten" => self.calculate_time_based_cost(
                model,
                &model_info,
                total_time_seconds.unwrap_or(0.0),
            ),
            "zhipuai" | "glm" => {
                self.calculate_token_based_cost(model, &model_info, input_tokens, output_tokens)
            }
            _ => {
                // Default to token-based calculation
                self.calculate_token_based_cost(model, &model_info, input_tokens, output_tokens)
            }
        }
    }

    /// Calculate token-based cost
    fn calculate_token_based_cost(
        &self,
        model: &str,
        model_info: &ModelInfo,
        input_tokens: u32,
        output_tokens: u32,
    ) -> Result<CostResult> {
        let input_cost_per_token = model_info.input_cost_per_token.unwrap_or(0.0);
        let output_cost_per_token = model_info.output_cost_per_token.unwrap_or(0.0);

        let input_cost = (input_tokens as f64) * input_cost_per_token;
        let output_cost = (output_tokens as f64) * output_cost_per_token;
        let total_cost = input_cost + output_cost;

        Ok(CostResult {
            input_cost,
            output_cost,
            total_cost,
            input_tokens,
            output_tokens,
            model: model.to_string(),
            provider: model_info.litellm_provider.clone(),
            cost_type: CostType::TokenBased,
        })
    }

    /// Calculate Google/Vertex AI cost (character or token based)
    fn calculate_google_cost(
        &self,
        model: &str,
        model_info: &ModelInfo,
        input_tokens: u32,
        output_tokens: u32,
        prompt: Option<&str>,
        completion: Option<&str>,
    ) -> Result<CostResult> {
        // Check if character-based pricing is available
        if model_info.input_cost_per_character.is_some()
            || model_info.output_cost_per_character.is_some()
        {
            let input_cost_per_char = model_info.input_cost_per_character.unwrap_or(0.0);
            let output_cost_per_char = model_info.output_cost_per_character.unwrap_or(0.0);

            let input_chars = prompt.map(|p| p.len()).unwrap_or(0) as f64;
            let output_chars = completion.map(|c| c.len()).unwrap_or(0) as f64;

            let input_cost = input_chars * input_cost_per_char;
            let output_cost = output_chars * output_cost_per_char;

            Ok(CostResult {
                input_cost,
                output_cost,
                total_cost: input_cost + output_cost,
                input_tokens,
                output_tokens,
                model: model.to_string(),
                provider: model_info.litellm_provider.clone(),
                cost_type: CostType::CharacterBased,
            })
        } else {
            // Fall back to token-based
            self.calculate_token_based_cost(model, model_info, input_tokens, output_tokens)
        }
    }

    /// Calculate time-based cost (for deployment providers)
    fn calculate_time_based_cost(
        &self,
        model: &str,
        model_info: &ModelInfo,
        total_time_seconds: f64,
    ) -> Result<CostResult> {
        let cost_per_second = model_info.cost_per_second.unwrap_or(0.0);
        let total_cost = total_time_seconds * cost_per_second;

        Ok(CostResult {
            input_cost: 0.0,
            output_cost: 0.0,
            total_cost,
            input_tokens: 0,
            output_tokens: 0,
            model: model.to_string(),
            provider: model_info.litellm_provider.clone(),
            cost_type: CostType::TimeBased,
        })
    }

    /// Get cost per token for a model
    pub fn get_cost_per_token(&self, model: &str) -> Option<(f64, f64)> {
        let model_info = self.get_model_info(model)?;
        Some((
            model_info.input_cost_per_token.unwrap_or(0.0),
            model_info.output_cost_per_token.unwrap_or(0.0),
        ))
    }

    /// Check if model supports a feature
    pub fn supports_feature(&self, model: &str, feature: &str) -> bool {
        let model_info = match self.get_model_info(model) {
            Some(info) => info,
            None => return false,
        };

        match feature {
            "function_calling" => model_info.supports_function_calling.unwrap_or(false),
            "vision" => model_info.supports_vision.unwrap_or(false),
            "streaming" => model_info.supports_streaming.unwrap_or(true), // Default to true
            "parallel_function_calling" => model_info
                .supports_parallel_function_calling
                .unwrap_or(false),
            "system_message" => model_info.supports_system_message.unwrap_or(true),
            _ => false,
        }
    }

    /// Get all available models for a provider
    pub fn get_models_by_provider(&self, provider: &str) -> Vec<String> {
        let model_data = self.model_data.read().unwrap();
        model_data
            .iter()
            .filter(|(_, info)| info.litellm_provider == provider)
            .map(|(model, _)| model.clone())
            .collect()
    }

    /// Get all available providers
    pub fn get_providers(&self) -> Vec<String> {
        let model_data = self.model_data.read().unwrap();
        let mut providers: Vec<String> = model_data
            .values()
            .map(|info| info.litellm_provider.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        providers.sort();
        providers
    }

    /// Add custom model pricing
    pub fn add_custom_model(&self, model: String, model_info: ModelInfo) {
        let mut model_data = self.model_data.write().unwrap();
        model_data.insert(model.clone(), model_info.clone());

        // Send update event
        let _ = self.event_sender.send(PricingUpdateEvent {
            event_type: PricingEventType::ModelAdded,
            model,
            provider: model_info.litellm_provider,
            timestamp: SystemTime::now(),
        });
    }

    /// Subscribe to pricing update events
    pub fn subscribe_to_updates(&self) -> broadcast::Receiver<PricingUpdateEvent> {
        self.event_sender.subscribe()
    }

    /// Get pricing statistics
    pub fn get_statistics(&self) -> PricingStatistics {
        let model_data = self.model_data.read().unwrap();
        let total_models = model_data.len();

        let mut provider_stats = HashMap::new();
        let mut cost_ranges = HashMap::new();

        for (_, model_info) in model_data.iter() {
            let provider = &model_info.litellm_provider;
            *provider_stats.entry(provider.clone()).or_insert(0) += 1;

            // Track cost ranges
            if let (Some(input_cost), Some(output_cost)) = (
                model_info.input_cost_per_token,
                model_info.output_cost_per_token,
            ) {
                let range = cost_ranges.entry(provider.clone()).or_insert(CostRange {
                    input_min: f64::MAX,
                    input_max: f64::MIN,
                    output_min: f64::MAX,
                    output_max: f64::MIN,
                });

                range.input_min = range.input_min.min(input_cost);
                range.input_max = range.input_max.max(input_cost);
                range.output_min = range.output_min.min(output_cost);
                range.output_max = range.output_max.max(output_cost);
            }
        }

        PricingStatistics {
            total_models,
            provider_stats,
            cost_ranges,
            last_updated: *self.last_updated.read().unwrap(),
        }
    }
}

/// Pricing statistics
#[derive(Debug, Clone)]
pub struct PricingStatistics {
    /// Total number of models in the pricing database
    pub total_models: usize,
    /// Number of models per provider
    pub provider_stats: HashMap<String, usize>,
    /// Cost ranges for each provider
    pub cost_ranges: HashMap<String, CostRange>,
    /// When the pricing data was last updated
    pub last_updated: SystemTime,
}

/// Cost range statistics for a provider
#[derive(Debug, Clone)]
pub struct CostRange {
    /// Minimum input cost per token
    pub input_min: f64,
    /// Maximum input cost per token
    pub input_max: f64,
    /// Minimum output cost per token
    pub output_min: f64,
    /// Maximum output cost per token
    pub output_max: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_info_deserialization() {
        let json = r#"{
            "max_tokens": 4096,
            "input_cost_per_token": 0.00001,
            "output_cost_per_token": 0.00003,
            "litellm_provider": "openai",
            "mode": "chat",
            "supports_function_calling": true
        }"#;

        let model_info: ModelInfo = serde_json::from_str(json).unwrap();
        assert_eq!(model_info.max_tokens, Some(4096));
        assert_eq!(model_info.input_cost_per_token, Some(0.00001));
        assert_eq!(model_info.litellm_provider, "openai");
    }

    #[tokio::test]
    async fn test_token_based_cost_calculation() {
        let service = PricingService::new(None);

        let model_info = ModelInfo {
            max_tokens: Some(4096),
            max_input_tokens: None,
            max_output_tokens: None,
            input_cost_per_token: Some(0.001),
            output_cost_per_token: Some(0.002),
            input_cost_per_character: None,
            output_cost_per_character: None,
            cost_per_second: None,
            litellm_provider: "openai".to_string(),
            mode: "chat".to_string(),
            supports_function_calling: Some(true),
            supports_vision: None,
            supports_streaming: None,
            supports_parallel_function_calling: None,
            supports_system_message: None,
            extra: HashMap::new(),
        };

        let result = service
            .calculate_token_based_cost("gpt-4", &model_info, 1000, 500)
            .unwrap();

        // 1000 * 0.001 + 500 * 0.002 = 1 + 1 = 2
        assert!((result.total_cost - 2.0).abs() < f64::EPSILON);
        assert_eq!(result.input_tokens, 1000);
        assert_eq!(result.output_tokens, 500);
    }
}
