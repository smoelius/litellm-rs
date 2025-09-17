//! Load balancer for provider selection

use crate::core::providers::Provider;
use crate::core::router::health::HealthChecker;
use crate::core::router::strategy::{RoutingStrategy, StrategyExecutor};
use crate::core::types::common::RequestContext;
use crate::utils::error::{GatewayError, Result};
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};

/// Load balancer for intelligent provider selection
pub struct LoadBalancer {
    /// Available providers
    providers: Arc<DashMap<String, Provider>>,
    /// Strategy executor
    strategy: Arc<StrategyExecutor>,
    /// Health checker
    health_checker: Option<Arc<HealthChecker>>,
    /// Provider model support cache with reference counting for efficiency
    model_support_cache: Arc<DashMap<String, Arc<Vec<String>>>>,
}

impl LoadBalancer {
    /// Create a new load balancer
    pub async fn new(strategy: RoutingStrategy) -> Result<Self> {
        info!("Creating load balancer with strategy: {:?}", strategy);

        let strategy_executor = Arc::new(StrategyExecutor::new(strategy).await?);

        Ok(Self {
            providers: Arc::new(DashMap::new()),
            strategy: strategy_executor,
            health_checker: None,
            model_support_cache: Arc::new(DashMap::new()),
        })
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
        let _selected_name = self
            .strategy
            .select_provider(&healthy_providers, model, context)
            .await?;

        // TODO: Fix provider cloning issue - need to redesign API to avoid cloning
        Err(GatewayError::NotImplemented(
            "Load balancer provider selection not implemented yet".to_string(),
        ))
    }

    /// Get providers that support a specific model
    async fn get_supporting_providers(&self, model: &str) -> Result<Vec<String>> {
        // Check cache first using DashMap
        if let Some(cached_providers) = self.model_support_cache.get(model) {
            debug!(
                "Found cached providers for model {}: {:?}",
                model, cached_providers
            );
            return Ok((**cached_providers).clone());
        }

        // Query providers for model support
        let mut supporting_providers = Vec::new();

        for entry in self.providers.iter() {
            let (name, provider) = entry.pair();
            if provider.supports_model(model) {
                supporting_providers.push(name.clone());
            }
        }

        // Cache the result with Arc for efficient sharing
        let cached_result = Arc::new(supporting_providers.clone());
        self.model_support_cache
            .insert(model.to_string(), cached_result);

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
            let mut updated_providers = (**providers).clone();
            updated_providers.retain(|p| p != name);
            if updated_providers.len() != providers.len() {
                false // Remove this cache entry as it contained the removed provider
            } else {
                true // Keep this cache entry
            }
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
        let mut result = HashMap::new();
        for entry in self.model_support_cache.iter() {
            let (key, value) = entry.pair();
            result.insert(key.clone(), (**value).clone());
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
