//! Load balancer for provider selection

use crate::core::providers::Provider;
use crate::core::providers::unified_provider::ProviderError;
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
    pub fn add_content_policy_fallback(
        &mut self,
        model: &str,
        fallbacks: Vec<String>,
    ) -> &mut Self {
        self.content_policy_fallbacks
            .insert(model.to_string(), fallbacks);
        self
    }

    /// Add a context window fallback
    pub fn add_context_window_fallback(
        &mut self,
        model: &str,
        fallbacks: Vec<String>,
    ) -> &mut Self {
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

/// Deployment information for tag/group-based routing
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeploymentInfo {
    /// Tags for this deployment (e.g., ["fast", "high-quality", "cost-effective"])
    #[serde(default)]
    pub tags: Vec<String>,
    /// Model group this deployment belongs to (e.g., "gpt-4-group")
    #[serde(default)]
    pub model_group: Option<String>,
    /// Priority within the group (lower = higher priority)
    #[serde(default)]
    pub priority: u32,
    /// Custom metadata for this deployment
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl DeploymentInfo {
    /// Create new deployment info
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a tag
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Add multiple tags
    pub fn with_tags(mut self, tags: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.tags.extend(tags.into_iter().map(|t| t.into()));
        self
    }

    /// Set model group
    pub fn with_group(mut self, group: impl Into<String>) -> Self {
        self.model_group = Some(group.into());
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// Check if deployment has all specified tags
    pub fn has_all_tags(&self, required_tags: &[String]) -> bool {
        required_tags.iter().all(|tag| self.tags.contains(tag))
    }

    /// Check if deployment has any of the specified tags
    pub fn has_any_tag(&self, tags: &[String]) -> bool {
        tags.iter().any(|tag| self.tags.contains(tag))
    }
}

/// Load balancer for intelligent provider selection
pub struct LoadBalancer {
    /// Available providers - DashMap provides interior mutability, no need for Arc wrapper
    providers: DashMap<String, Provider>,
    /// Deployment information for each provider (tags, groups, etc.)
    deployments: DashMap<String, DeploymentInfo>,
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
            deployments: DashMap::new(),
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
            deployments: DashMap::new(),
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

        // Create default deployment info if not already present
        self.deployments.entry(name.to_string()).or_default();

        // Clear model support cache since we have a new provider
        self.model_support_cache.clear();

        info!("Added provider {} to load balancer", name);
        Ok(())
    }

    /// Add a provider with deployment info (tags, groups, etc.)
    pub async fn add_provider_with_deployment(
        &self,
        name: &str,
        provider: Provider,
        deployment_info: DeploymentInfo,
    ) -> Result<()> {
        // Add provider to the map
        self.providers.insert(name.to_string(), provider);
        self.deployments
            .insert(name.to_string(), deployment_info.clone());

        // Clear model support cache since we have a new provider
        self.model_support_cache.clear();

        info!(
            "Added provider {} with tags {:?}, group {:?}",
            name, deployment_info.tags, deployment_info.model_group
        );
        Ok(())
    }

    /// Update deployment info for an existing provider
    pub fn update_deployment_info(&self, name: &str, deployment_info: DeploymentInfo) {
        self.deployments.insert(name.to_string(), deployment_info);
        debug!("Updated deployment info for provider {}", name);
    }

    /// Get deployment info for a provider
    pub fn get_deployment_info(&self, name: &str) -> Option<DeploymentInfo> {
        self.deployments
            .get(name)
            .map(|entry| entry.value().clone())
    }

    /// Remove a provider from the load balancer
    pub async fn remove_provider(&self, name: &str) -> Result<()> {
        // Remove provider and deployment info from the maps
        self.providers.remove(name);
        self.deployments.remove(name);

        // Selectively invalidate cache entries that might include this provider
        self.model_support_cache
            .retain(|_, providers| !providers.contains(&name.to_string()));

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

    /// Select a provider with tag filtering
    ///
    /// Filters providers by tags before applying the routing strategy.
    /// If `require_all_tags` is true, providers must have ALL specified tags.
    /// If false, providers with ANY of the tags will be included.
    pub async fn select_provider_with_tags(
        &self,
        model: &str,
        tags: &[String],
        require_all_tags: bool,
        context: &RequestContext,
    ) -> Result<Provider> {
        // Get providers that support the model
        let supporting_providers = self.get_supporting_providers(model).await?;

        if supporting_providers.is_empty() {
            return Err(GatewayError::NoProvidersForModel(model.to_string()));
        }

        // Filter by tags
        let tagged_providers: Vec<String> = supporting_providers
            .into_iter()
            .filter(|name| {
                self.deployments
                    .get(name)
                    .map(|info| {
                        if require_all_tags {
                            info.has_all_tags(tags)
                        } else {
                            info.has_any_tag(tags)
                        }
                    })
                    .unwrap_or(false)
            })
            .collect();

        if tagged_providers.is_empty() {
            return Err(GatewayError::NoProvidersForModel(format!(
                "{} with tags {:?}",
                model, tags
            )));
        }

        // Filter by health status if health checker is available
        let healthy_providers = if let Some(health_checker) = &self.health_checker {
            let healthy_list = health_checker.get_healthy_providers().await?;
            tagged_providers
                .into_iter()
                .filter(|p| healthy_list.contains(p))
                .collect()
        } else {
            tagged_providers
        };

        if healthy_providers.is_empty() {
            return Err(GatewayError::NoHealthyProviders(
                "No healthy providers with matching tags available".to_string(),
            ));
        }

        // Use strategy to select provider
        let selected_name = self
            .strategy
            .select_provider(&healthy_providers, model, context)
            .await?;

        // Get the selected provider and clone it
        if let Some(provider_ref) = self.providers.get(&selected_name) {
            debug!(
                "Selected provider {} for model {} with tags {:?}",
                selected_name, model, tags
            );
            Ok(provider_ref.value().clone())
        } else {
            Err(GatewayError::ProviderNotFound(format!(
                "Provider {} not found in load balancer",
                selected_name
            )))
        }
    }

    /// Select a provider by model group
    ///
    /// Filters providers by model group before applying the routing strategy.
    pub async fn select_provider_by_group(
        &self,
        model: &str,
        group: &str,
        context: &RequestContext,
    ) -> Result<Provider> {
        // Get providers that support the model
        let supporting_providers = self.get_supporting_providers(model).await?;

        if supporting_providers.is_empty() {
            return Err(GatewayError::NoProvidersForModel(model.to_string()));
        }

        // Filter by model group and collect with priority for sorting
        let mut grouped_providers: Vec<(String, u32)> = supporting_providers
            .into_iter()
            .filter_map(|name| {
                self.deployments.get(&name).and_then(|info| {
                    if info.model_group.as_deref() == Some(group) {
                        Some((name, info.priority))
                    } else {
                        None
                    }
                })
            })
            .collect();

        if grouped_providers.is_empty() {
            return Err(GatewayError::NoProvidersForModel(format!(
                "{} in group {}",
                model, group
            )));
        }

        // Sort by priority (lower priority number = higher priority)
        grouped_providers.sort_by_key(|(_, priority)| *priority);

        // Extract just the names for strategy selection
        let provider_names: Vec<String> = grouped_providers
            .into_iter()
            .map(|(name, _)| name)
            .collect();

        // Filter by health status if health checker is available
        let healthy_providers = if let Some(health_checker) = &self.health_checker {
            let healthy_list = health_checker.get_healthy_providers().await?;
            provider_names
                .into_iter()
                .filter(|p| healthy_list.contains(p))
                .collect()
        } else {
            provider_names
        };

        if healthy_providers.is_empty() {
            return Err(GatewayError::NoHealthyProviders(
                "No healthy providers in group available".to_string(),
            ));
        }

        // Use strategy to select provider
        let selected_name = self
            .strategy
            .select_provider(&healthy_providers, model, context)
            .await?;

        // Get the selected provider and clone it
        if let Some(provider_ref) = self.providers.get(&selected_name) {
            debug!(
                "Selected provider {} for model {} in group {}",
                selected_name, model, group
            );
            Ok(provider_ref.value().clone())
        } else {
            Err(GatewayError::ProviderNotFound(format!(
                "Provider {} not found in load balancer",
                selected_name
            )))
        }
    }

    /// Get all providers with a specific tag
    pub fn get_providers_by_tag(&self, tag: &str) -> Vec<String> {
        self.deployments
            .iter()
            .filter_map(|entry| {
                if entry.value().tags.contains(&tag.to_string()) {
                    Some(entry.key().clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get all providers in a specific model group
    pub fn get_providers_by_group(&self, group: &str) -> Vec<String> {
        self.deployments
            .iter()
            .filter_map(|entry| {
                if entry.value().model_group.as_deref() == Some(group) {
                    Some(entry.key().clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get all unique tags across all deployments
    pub fn get_all_tags(&self) -> Vec<String> {
        let mut tags: Vec<String> = self
            .deployments
            .iter()
            .flat_map(|entry| entry.value().tags.clone())
            .collect();
        tags.sort();
        tags.dedup();
        tags
    }

    /// Get all unique model groups
    pub fn get_all_groups(&self) -> Vec<String> {
        let mut groups: Vec<String> = self
            .deployments
            .iter()
            .filter_map(|entry| entry.value().model_group.clone())
            .collect();
        groups.sort();
        groups.dedup();
        groups
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
                    warn!("Fallback model {} not available: {}", fallback_model, e);
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
            lb.fallback_config().general_fallbacks.get("gpt-4"),
            Some(&vec!["gpt-3.5-turbo".to_string()])
        );
    }

    #[tokio::test]
    async fn test_select_fallback_models_context_length() {
        let mut config = FallbackConfig::new();
        config
            .add_context_window_fallback(
                "gpt-4",
                vec!["gpt-4-32k".to_string(), "gpt-4-turbo".to_string()],
            )
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
        let lb = LoadBalancer::new(RoutingStrategy::RoundRobin)
            .await
            .unwrap();

        let error = ProviderError::timeout("openai", "Request timeout");
        let fallbacks = lb.select_fallback_models(&error, "gpt-4");
        assert_eq!(fallbacks, None);
    }

    // Tag/Group routing tests

    #[test]
    fn test_deployment_info_builder() {
        let info = DeploymentInfo::new()
            .with_tag("fast")
            .with_tag("high-quality")
            .with_group("gpt-4-group")
            .with_priority(1);

        assert_eq!(info.tags, vec!["fast", "high-quality"]);
        assert_eq!(info.model_group, Some("gpt-4-group".to_string()));
        assert_eq!(info.priority, 1);
    }

    #[test]
    fn test_deployment_info_with_tags() {
        let info = DeploymentInfo::new().with_tags(["fast", "cheap", "reliable"]);

        assert_eq!(info.tags.len(), 3);
        assert!(info.tags.contains(&"fast".to_string()));
        assert!(info.tags.contains(&"cheap".to_string()));
        assert!(info.tags.contains(&"reliable".to_string()));
    }

    #[test]
    fn test_deployment_info_has_all_tags() {
        let info = DeploymentInfo::new().with_tags(["fast", "cheap", "reliable"]);

        // Should match when all tags present
        assert!(info.has_all_tags(&["fast".to_string(), "cheap".to_string()]));

        // Should not match when a tag is missing
        assert!(!info.has_all_tags(&["fast".to_string(), "expensive".to_string()]));

        // Empty tags should always match
        assert!(info.has_all_tags(&[]));
    }

    #[test]
    fn test_deployment_info_has_any_tag() {
        let info = DeploymentInfo::new().with_tags(["fast", "cheap"]);

        // Should match when any tag present
        assert!(info.has_any_tag(&["fast".to_string(), "expensive".to_string()]));

        // Should not match when no tags present
        assert!(!info.has_any_tag(&["expensive".to_string(), "slow".to_string()]));

        // Empty tags should not match
        assert!(!info.has_any_tag(&[]));
    }

    #[tokio::test]
    async fn test_load_balancer_with_deployment_info() {
        let lb = LoadBalancer::new(RoutingStrategy::RoundRobin)
            .await
            .unwrap();

        let deployment = DeploymentInfo::new()
            .with_tag("fast")
            .with_group("test-group");

        // Use update_deployment_info since we don't have a real Provider
        lb.deployments
            .insert("test_provider".to_string(), deployment);

        let info = lb.get_deployment_info("test_provider");
        assert!(info.is_some());

        let info = info.unwrap();
        assert!(info.tags.contains(&"fast".to_string()));
        assert_eq!(info.model_group, Some("test-group".to_string()));
    }

    #[tokio::test]
    async fn test_get_providers_by_tag() {
        let lb = LoadBalancer::new(RoutingStrategy::RoundRobin)
            .await
            .unwrap();

        // Add deployment info directly
        lb.deployments.insert(
            "provider_a".to_string(),
            DeploymentInfo::new().with_tags(["fast", "cheap"]),
        );
        lb.deployments.insert(
            "provider_b".to_string(),
            DeploymentInfo::new().with_tags(["fast", "quality"]),
        );
        lb.deployments.insert(
            "provider_c".to_string(),
            DeploymentInfo::new().with_tag("quality"),
        );

        let fast_providers = lb.get_providers_by_tag("fast");
        assert_eq!(fast_providers.len(), 2);
        assert!(fast_providers.contains(&"provider_a".to_string()));
        assert!(fast_providers.contains(&"provider_b".to_string()));

        let quality_providers = lb.get_providers_by_tag("quality");
        assert_eq!(quality_providers.len(), 2);
        assert!(quality_providers.contains(&"provider_b".to_string()));
        assert!(quality_providers.contains(&"provider_c".to_string()));

        let cheap_providers = lb.get_providers_by_tag("cheap");
        assert_eq!(cheap_providers.len(), 1);
        assert!(cheap_providers.contains(&"provider_a".to_string()));
    }

    #[tokio::test]
    async fn test_get_providers_by_group() {
        let lb = LoadBalancer::new(RoutingStrategy::RoundRobin)
            .await
            .unwrap();

        lb.deployments.insert(
            "provider_a".to_string(),
            DeploymentInfo::new().with_group("gpt-4-group"),
        );
        lb.deployments.insert(
            "provider_b".to_string(),
            DeploymentInfo::new().with_group("gpt-4-group"),
        );
        lb.deployments.insert(
            "provider_c".to_string(),
            DeploymentInfo::new().with_group("claude-group"),
        );

        let gpt4_providers = lb.get_providers_by_group("gpt-4-group");
        assert_eq!(gpt4_providers.len(), 2);
        assert!(gpt4_providers.contains(&"provider_a".to_string()));
        assert!(gpt4_providers.contains(&"provider_b".to_string()));

        let claude_providers = lb.get_providers_by_group("claude-group");
        assert_eq!(claude_providers.len(), 1);
        assert!(claude_providers.contains(&"provider_c".to_string()));
    }

    #[tokio::test]
    async fn test_get_all_tags() {
        let lb = LoadBalancer::new(RoutingStrategy::RoundRobin)
            .await
            .unwrap();

        lb.deployments.insert(
            "provider_a".to_string(),
            DeploymentInfo::new().with_tags(["fast", "cheap"]),
        );
        lb.deployments.insert(
            "provider_b".to_string(),
            DeploymentInfo::new().with_tags(["fast", "quality"]),
        );

        let all_tags = lb.get_all_tags();
        assert_eq!(all_tags.len(), 3); // cheap, fast, quality (sorted, deduplicated)
        assert_eq!(all_tags, vec!["cheap", "fast", "quality"]);
    }

    #[tokio::test]
    async fn test_get_all_groups() {
        let lb = LoadBalancer::new(RoutingStrategy::RoundRobin)
            .await
            .unwrap();

        lb.deployments.insert(
            "provider_a".to_string(),
            DeploymentInfo::new().with_group("gpt-4-group"),
        );
        lb.deployments.insert(
            "provider_b".to_string(),
            DeploymentInfo::new().with_group("gpt-4-group"),
        );
        lb.deployments.insert(
            "provider_c".to_string(),
            DeploymentInfo::new().with_group("claude-group"),
        );
        lb.deployments.insert(
            "provider_d".to_string(),
            DeploymentInfo::new(), // No group
        );

        let all_groups = lb.get_all_groups();
        assert_eq!(all_groups.len(), 2); // claude-group, gpt-4-group (sorted, deduplicated)
        assert_eq!(all_groups, vec!["claude-group", "gpt-4-group"]);
    }
}
