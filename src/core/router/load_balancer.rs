//! Load balancer for provider selection

use crate::core::providers::unified_provider::ProviderError;
use crate::core::providers::Provider;
use crate::core::router::health::HealthChecker;
use crate::core::router::strategy::{RoutingStrategy, StrategyExecutor};
use crate::core::types::common::RequestContext;
use crate::utils::error::{GatewayError, Result};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Configuration for error-specific fallbacks
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FallbackConfig {
    /// General fallbacks for any error (model -> fallback models)
    #[serde(default)]
    pub general_fallbacks: HashMap<String, Vec<String>>,
    /// Fallbacks for content policy violations
    #[serde(default)]
    pub content_policy_fallbacks: HashMap<String, Vec<String>>,
    /// Fallbacks for context window exceeded errors
    #[serde(default)]
    pub context_window_fallbacks: HashMap<String, Vec<String>>,
    /// Fallbacks for rate limit errors
    #[serde(default)]
    pub rate_limit_fallbacks: HashMap<String, Vec<String>>,
}

impl FallbackConfig {
    /// Create a new fallback config
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a general fallback
    pub fn add_general_fallback(&mut self, model: &str, fallbacks: Vec<String>) -> &mut Self {
        self.general_fallbacks.insert(model.to_string(), fallbacks);
        self
    }

    /// Add a content policy fallback
    pub fn add_content_policy_fallback(&mut self, model: &str, fallbacks: Vec<String>) -> &mut Self {
        self.content_policy_fallbacks
            .insert(model.to_string(), fallbacks);
        self
    }

    /// Add a context window fallback
    pub fn add_context_window_fallback(&mut self, model: &str, fallbacks: Vec<String>) -> &mut Self {
        self.context_window_fallbacks
            .insert(model.to_string(), fallbacks);
        self
    }

    /// Add a rate limit fallback
    pub fn add_rate_limit_fallback(&mut self, model: &str, fallbacks: Vec<String>) -> &mut Self {
        self.rate_limit_fallbacks
            .insert(model.to_string(), fallbacks);
        self
    }
}

/// Load balancer for intelligent provider selection
pub struct LoadBalancer {
    /// Available providers - DashMap provides interior mutability, no need for Arc wrapper
    providers: DashMap<String, Provider>,
    /// Strategy executor
    strategy: Arc<StrategyExecutor>,
    /// Health checker
    health_checker: Option<Arc<HealthChecker>>,
    /// Provider model support cache - uses Arc to avoid cloning Vec on every lookup
    model_support_cache: DashMap<String, Arc<Vec<String>>>,
    /// Error-specific fallback configuration
    fallback_config: FallbackConfig,
}

impl LoadBalancer {
    /// Create a new load balancer
    pub async fn new(strategy: RoutingStrategy) -> Result<Self> {
        info!("Creating load balancer with strategy: {:?}", strategy);

        let strategy_executor = Arc::new(StrategyExecutor::new(strategy).await?);

        Ok(Self {
            providers: DashMap::new(),
            strategy: strategy_executor,
            health_checker: None,
            model_support_cache: DashMap::new(),
            fallback_config: FallbackConfig::default(),
        })
    }

    /// Create a new load balancer with fallback configuration
    pub async fn with_fallbacks(
        strategy: RoutingStrategy,
        fallback_config: FallbackConfig,
    ) -> Result<Self> {
        info!(
            "Creating load balancer with strategy: {:?} and fallback config",
            strategy
        );

        let strategy_executor = Arc::new(StrategyExecutor::new(strategy).await?);

        Ok(Self {
            providers: DashMap::new(),
            strategy: strategy_executor,
            health_checker: None,
            model_support_cache: DashMap::new(),
            fallback_config,
        })
    }

    /// Set fallback configuration
    pub fn set_fallback_config(&mut self, config: FallbackConfig) {
        self.fallback_config = config;
        info!("Updated fallback configuration");
    }

    /// Get fallback configuration
    pub fn fallback_config(&self) -> &FallbackConfig {
        &self.fallback_config
    }

    /// Set health checker
    pub async fn set_health_checker(&mut self, health_checker: Arc<HealthChecker>) {
        self.health_checker = Some(health_checker);
        info!("Health checker attached to load balancer");
    }

    /// Select a provider for the given model and context
    pub async fn select_provider(&self, model: &str, context: &RequestContext) -> Result<Provider> {
        // Get providers that support the model
        let supporting_providers = self.get_supporting_providers(model).await?;

        if supporting_providers.is_empty() {
            return Err(GatewayError::NoProvidersForModel(model.to_string()));
        }

        // Filter by health status if health checker is available
        let healthy_providers = if let Some(health_checker) = &self.health_checker {
            let healthy_list = health_checker.get_healthy_providers().await?;
            supporting_providers
                .into_iter()
                .filter(|p| healthy_list.contains(p))
                .collect()
        } else {
            supporting_providers
        };

        if healthy_providers.is_empty() {
            return Err(GatewayError::NoHealthyProviders(
                "No healthy providers available".to_string(),
            ));
        }

        // Use strategy to select provider
        let selected_name = self
            .strategy
            .select_provider(&healthy_providers, model, context)
            .await?;

        // Get the selected provider and clone it
        if let Some(provider_ref) = self.providers.get(&selected_name) {
            Ok(provider_ref.value().clone())
        } else {
            Err(GatewayError::ProviderNotFound(format!(
                "Provider {} not found in load balancer",
                selected_name
            )))
        }
    }

    /// Get providers that support a specific model
    async fn get_supporting_providers(&self, model: &str) -> Result<Vec<String>> {
        // Check cache first using DashMap
        if let Some(cached_providers) = self.model_support_cache.get(model) {
            debug!(
                "Found cached providers for model {}: {:?}",
                model,
                cached_providers.value()
            );
            // Arc::as_ref() gives &Vec, then clone the Vec (unavoidable due to return type)
            return Ok(cached_providers.value().as_ref().clone());
        }

        // Query providers for model support - pre-allocate with capacity hint
        let mut supporting_providers = Vec::with_capacity(self.providers.len());

        for entry in self.providers.iter() {
            let (name, provider) = entry.pair();
            if provider.supports_model(model) {
                supporting_providers.push(name.clone());
            }
        }

        // Cache the result wrapped in Arc for potential concurrent access
        self.model_support_cache
            .insert(model.to_string(), Arc::new(supporting_providers.clone()));

        debug!(
            "Providers supporting model {}: {:?}",
            model, supporting_providers
        );
        Ok(supporting_providers)
    }

    /// Add a provider to the load balancer
    pub async fn add_provider(&self, name: &str, provider: Provider) -> Result<()> {
        // Add provider to the map
        self.providers.insert(name.to_string(), provider);

        // Clear model support cache since we have a new provider
        self.model_support_cache.clear();

        info!("Added provider {} to load balancer", name);
        Ok(())
    }

    /// Remove a provider from the load balancer
    pub async fn remove_provider(&self, name: &str) -> Result<()> {
        // Remove provider from the map
        self.providers.remove(name);

        // Selectively invalidate cache entries that might include this provider
        self.model_support_cache.retain(|_, providers| {
            !providers.contains(&name.to_string())
        });

        info!("Removed provider {} from load balancer", name);
        Ok(())
    }

    /// Update provider weight for weighted routing
    pub async fn update_provider_weight(&self, provider: &str, weight: f64) -> Result<()> {
        self.strategy.update_weight(provider, weight).await
    }

    /// Update provider latency for latency-based routing
    pub async fn update_provider_latency(&self, provider: &str, latency: f64) -> Result<()> {
        self.strategy.update_latency(provider, latency).await
    }

    /// Update provider cost for cost-based routing
    pub async fn update_provider_cost(&self, provider: &str, model: &str, cost: f64) -> Result<()> {
        self.strategy.update_cost(provider, model, cost).await
    }

    /// Update provider priority for priority-based routing
    pub async fn update_provider_priority(&self, provider: &str, priority: u32) -> Result<()> {
        self.strategy.update_priority(provider, priority).await
    }

    /// Get load balancer statistics
    pub async fn get_stats(&self) -> Result<LoadBalancerStats> {
        let provider_count = self.providers.len();

        let healthy_count = if let Some(health_checker) = &self.health_checker {
            health_checker.get_healthy_providers().await?.len()
        } else {
            provider_count
        };

        let cached_models = self.model_support_cache.len();

        Ok(LoadBalancerStats {
            total_providers: provider_count,
            healthy_providers: healthy_count,
            cached_models,
        })
    }

    /// Clear model support cache
    pub async fn clear_cache(&self) -> Result<()> {
        self.model_support_cache.clear();
        info!("Cleared model support cache");
        Ok(())
    }

    /// Get cached model support information
    pub async fn get_model_cache(&self) -> Result<HashMap<String, Vec<String>>> {
        let mut result = HashMap::with_capacity(self.model_support_cache.len());
        for entry in self.model_support_cache.iter() {
            let (key, value) = entry.pair();
            result.insert(key.clone(), value.as_ref().clone());
        }
        Ok(result)
    }

    /// Preload model support cache for common models
    pub async fn preload_cache(&self, models: &[String]) -> Result<()> {
        info!("Preloading model support cache for {} models", models.len());

        for model in models {
            self.get_supporting_providers(model).await?;
        }

        info!("Model support cache preloaded successfully");
        Ok(())
    }

    /// Select fallback model based on error type
    ///
    /// Returns an ordered list of fallback models to try based on the error type.
    /// The fallback selection priority is:
    /// 1. Error-specific fallbacks (context_window, content_policy, rate_limit)
    /// 2. General fallbacks
    /// 3. None if no fallbacks configured
    pub fn select_fallback_models(
        &self,
        error: &ProviderError,
        original_model: &str,
    ) -> Option<Vec<String>> {
        // First, try error-specific fallbacks
        let specific_fallbacks = match error {
            ProviderError::ContextLengthExceeded { .. } => {
                debug!(
                    "Looking for context window fallbacks for model: {}",
                    original_model
                );
                self.fallback_config
                    .context_window_fallbacks
                    .get(original_model)
            }
            ProviderError::ContentFiltered { .. } => {
                debug!(
                    "Looking for content policy fallbacks for model: {}",
                    original_model
                );
                self.fallback_config
                    .content_policy_fallbacks
                    .get(original_model)
            }
            ProviderError::RateLimit { .. } => {
                debug!(
                    "Looking for rate limit fallbacks for model: {}",
                    original_model
                );
                self.fallback_config
                    .rate_limit_fallbacks
                    .get(original_model)
            }
            _ => None,
        };

        if let Some(fallbacks) = specific_fallbacks {
            if !fallbacks.is_empty() {
                info!(
                    "Found error-specific fallbacks for {}: {:?}",
                    original_model, fallbacks
                );
                return Some(fallbacks.clone());
            }
        }

        // Fall back to general fallbacks
        if let Some(general) = self.fallback_config.general_fallbacks.get(original_model) {
            if !general.is_empty() {
                info!(
                    "Using general fallbacks for {}: {:?}",
                    original_model, general
                );
                return Some(general.clone());
            }
        }

        debug!("No fallbacks configured for model: {}", original_model);
        None
    }

    /// Select fallback provider for error with context
    ///
    /// Attempts to find a healthy provider that supports one of the fallback models.
    /// Returns the first available (model, provider) pair.
    pub async fn select_fallback_provider(
        &self,
        error: &ProviderError,
        original_model: &str,
        context: &RequestContext,
    ) -> Option<(String, Provider)> {
        let fallback_models = self.select_fallback_models(error, original_model)?;

        for fallback_model in fallback_models {
            match self.select_provider(&fallback_model, context).await {
                Ok(provider) => {
                    info!(
                        "Selected fallback: model={}, provider for original={}",
                        fallback_model, original_model
                    );
                    return Some((fallback_model, provider));
                }
                Err(e) => {
                    warn!(
                        "Fallback model {} not available: {}",
                        fallback_model, e
                    );
                    continue;
                }
            }
        }

        warn!(
            "No fallback providers available for model: {}",
            original_model
        );
        None
    }
}

/// Load balancer statistics
#[derive(Debug, Clone)]
pub struct LoadBalancerStats {
    /// Total number of providers
    pub total_providers: usize,
    /// Number of healthy providers
    pub healthy_providers: usize,
    /// Number of cached model mappings
    pub cached_models: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fallback_config_builder() {
        let mut config = FallbackConfig::new();
        config
            .add_general_fallback("gpt-4", vec!["gpt-3.5-turbo".to_string()])
            .add_context_window_fallback("gpt-4", vec!["gpt-4-32k".to_string()])
            .add_content_policy_fallback("gpt-4", vec!["claude-3-opus".to_string()])
            .add_rate_limit_fallback("gpt-4", vec!["gpt-4-turbo".to_string()]);

        assert_eq!(
            config.general_fallbacks.get("gpt-4"),
            Some(&vec!["gpt-3.5-turbo".to_string()])
        );
        assert_eq!(
            config.context_window_fallbacks.get("gpt-4"),
            Some(&vec!["gpt-4-32k".to_string()])
        );
        assert_eq!(
            config.content_policy_fallbacks.get("gpt-4"),
            Some(&vec!["claude-3-opus".to_string()])
        );
        assert_eq!(
            config.rate_limit_fallbacks.get("gpt-4"),
            Some(&vec!["gpt-4-turbo".to_string()])
        );
    }

    #[tokio::test]
    async fn test_load_balancer_creation() {
        let lb = LoadBalancer::new(RoutingStrategy::RoundRobin).await;
        assert!(lb.is_ok());
    }

    #[tokio::test]
    async fn test_load_balancer_with_fallbacks() {
        let mut config = FallbackConfig::new();
        config.add_general_fallback("gpt-4", vec!["gpt-3.5-turbo".to_string()]);

        let lb = LoadBalancer::with_fallbacks(RoutingStrategy::RoundRobin, config).await;
        assert!(lb.is_ok());

        let lb = lb.unwrap();
        assert_eq!(
            lb.fallback_config()
                .general_fallbacks
                .get("gpt-4"),
            Some(&vec!["gpt-3.5-turbo".to_string()])
        );
    }

    #[tokio::test]
    async fn test_select_fallback_models_context_length() {
        let mut config = FallbackConfig::new();
        config
            .add_context_window_fallback("gpt-4", vec!["gpt-4-32k".to_string(), "gpt-4-turbo".to_string()])
            .add_general_fallback("gpt-4", vec!["gpt-3.5-turbo".to_string()]);

        let lb = LoadBalancer::with_fallbacks(RoutingStrategy::RoundRobin, config)
            .await
            .unwrap();

        // Context length error should use context_window_fallbacks
        let error = ProviderError::ContextLengthExceeded {
            provider: "openai",
            max: 8192,
            actual: 10000,
        };
        let fallbacks = lb.select_fallback_models(&error, "gpt-4");
        assert_eq!(
            fallbacks,
            Some(vec!["gpt-4-32k".to_string(), "gpt-4-turbo".to_string()])
        );
    }

    #[tokio::test]
    async fn test_select_fallback_models_content_filtered() {
        let mut config = FallbackConfig::new();
        config
            .add_content_policy_fallback("gpt-4", vec!["claude-3-opus".to_string()])
            .add_general_fallback("gpt-4", vec!["gpt-3.5-turbo".to_string()]);

        let lb = LoadBalancer::with_fallbacks(RoutingStrategy::RoundRobin, config)
            .await
            .unwrap();

        // Content filter error should use content_policy_fallbacks
        let error = ProviderError::ContentFiltered {
            provider: "openai",
            reason: "Violated content policy".to_string(),
            policy_violations: None,
            potentially_retryable: Some(false),
        };
        let fallbacks = lb.select_fallback_models(&error, "gpt-4");
        assert_eq!(fallbacks, Some(vec!["claude-3-opus".to_string()]));
    }

    #[tokio::test]
    async fn test_select_fallback_models_rate_limit() {
        let mut config = FallbackConfig::new();
        config
            .add_rate_limit_fallback("gpt-4", vec!["gpt-4-turbo".to_string()])
            .add_general_fallback("gpt-4", vec!["gpt-3.5-turbo".to_string()]);

        let lb = LoadBalancer::with_fallbacks(RoutingStrategy::RoundRobin, config)
            .await
            .unwrap();

        // Rate limit error should use rate_limit_fallbacks
        let error = ProviderError::RateLimit {
            provider: "openai",
            message: "Rate limit exceeded".to_string(),
            retry_after: Some(60),
            rpm_limit: Some(100),
            tpm_limit: Some(10000),
            current_usage: None,
        };
        let fallbacks = lb.select_fallback_models(&error, "gpt-4");
        assert_eq!(fallbacks, Some(vec!["gpt-4-turbo".to_string()]));
    }

    #[tokio::test]
    async fn test_select_fallback_models_falls_back_to_general() {
        let mut config = FallbackConfig::new();
        // Only general fallback configured
        config.add_general_fallback("gpt-4", vec!["gpt-3.5-turbo".to_string()]);

        let lb = LoadBalancer::with_fallbacks(RoutingStrategy::RoundRobin, config)
            .await
            .unwrap();

        // Any error should fall back to general
        let error = ProviderError::timeout("openai", "Request timeout");
        let fallbacks = lb.select_fallback_models(&error, "gpt-4");
        assert_eq!(fallbacks, Some(vec!["gpt-3.5-turbo".to_string()]));
    }

    #[tokio::test]
    async fn test_select_fallback_models_no_config() {
        let lb = LoadBalancer::new(RoutingStrategy::RoundRobin).await.unwrap();

        let error = ProviderError::timeout("openai", "Request timeout");
        let fallbacks = lb.select_fallback_models(&error, "gpt-4");
        assert_eq!(fallbacks, None);
    }
}
