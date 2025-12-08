//! Routing strategies for provider selection

use crate::core::types::common::RequestContext;
use crate::utils::error::{GatewayError, Result};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use tracing::{debug, info};

/// Routing strategies for provider selection
#[derive(Debug, Clone)]
pub enum RoutingStrategy {
    /// Round-robin selection
    RoundRobin,
    /// Least latency first
    LeastLatency,
    /// Least cost first
    LeastCost,
    /// Random selection
    Random,
    /// Weighted selection based on provider weights
    Weighted,
    /// Priority-based selection
    Priority,
    /// A/B testing with traffic split
    ABTest {
        /// Split ratio for A/B testing (0.0 to 1.0)
        split_ratio: f64,
    },
    /// Custom strategy with user-defined logic
    Custom(String),
}

impl Default for RoutingStrategy {
    fn default() -> Self {
        Self::RoundRobin
    }
}

/// Strategy executor for provider selection
pub struct StrategyExecutor {
    /// Current strategy
    strategy: RoutingStrategy,
    /// Round-robin counter
    round_robin_counter: AtomicUsize,
    /// Consolidated routing data - single lock for all related data
    /// This reduces lock contention when multiple strategies are used
    routing_data: RwLock<RoutingData>,
}

/// Consolidated routing data for all strategies
#[derive(Debug, Default)]
struct RoutingData {
    /// Provider weights for weighted strategy
    weights: HashMap<String, f64>,
    /// Provider latencies for latency-based routing
    latencies: HashMap<String, f64>,
    /// Provider costs for cost-based routing
    costs: HashMap<String, f64>,
    /// Provider priorities
    priorities: HashMap<String, u32>,
}

impl StrategyExecutor {
    /// Create a new strategy executor
    pub async fn new(strategy: RoutingStrategy) -> Result<Self> {
        info!("Creating strategy executor with strategy: {:?}", strategy);

        Ok(Self {
            strategy,
            round_robin_counter: AtomicUsize::new(0),
            routing_data: RwLock::new(RoutingData::default()),
        })
    }

    /// Select a provider based on the current strategy
    pub async fn select_provider(
        &self,
        available_providers: &[String],
        model: &str,
        context: &RequestContext,
    ) -> Result<String> {
        if available_providers.is_empty() {
            return Err(GatewayError::NoProvidersAvailable(
                "No providers available".to_string(),
            ));
        }

        match &self.strategy {
            RoutingStrategy::RoundRobin => self.select_round_robin(available_providers).await,
            RoutingStrategy::LeastLatency => self.select_least_latency(available_providers).await,
            RoutingStrategy::LeastCost => self.select_least_cost(available_providers, model).await,
            RoutingStrategy::Random => self.select_random(available_providers).await,
            RoutingStrategy::Weighted => self.select_weighted(available_providers).await,
            RoutingStrategy::Priority => self.select_priority(available_providers).await,
            RoutingStrategy::ABTest { split_ratio } => {
                self.select_ab_test(available_providers, *split_ratio).await
            }
            RoutingStrategy::Custom(logic) => {
                self.select_custom(available_providers, logic, context)
                    .await
            }
        }
    }

    /// Round-robin provider selection
    async fn select_round_robin(&self, providers: &[String]) -> Result<String> {
        let index = self.round_robin_counter.fetch_add(1, Ordering::Relaxed) % providers.len();
        debug!(
            "Round-robin selected provider at index {}: {}",
            index, providers[index]
        );
        Ok(providers[index].clone())
    }

    /// Select provider with least latency
    async fn select_least_latency(&self, providers: &[String]) -> Result<String> {
        let data = self.routing_data.read();

        let mut best_provider = &providers[0];
        let mut best_latency = f64::MAX;

        for provider in providers {
            let latency = data.latencies.get(provider).copied().unwrap_or(f64::MAX);
            if latency < best_latency {
                best_latency = latency;
                best_provider = provider;
            }
        }

        debug!(
            "Least latency selected provider: {} ({}ms)",
            best_provider, best_latency
        );
        Ok(best_provider.clone())
    }

    /// Select provider with least cost
    async fn select_least_cost(&self, providers: &[String], model: &str) -> Result<String> {
        let data = self.routing_data.read();

        let mut best_provider = &providers[0];
        let mut best_cost = f64::MAX;

        for provider in providers {
            let cost_key = format!("{}:{}", provider, model);
            let cost = data.costs.get(&cost_key).copied().unwrap_or(f64::MAX);
            if cost < best_cost {
                best_cost = cost;
                best_provider = provider;
            }
        }

        debug!(
            "Least cost selected provider: {} (${:.4})",
            best_provider, best_cost
        );
        Ok(best_provider.clone())
    }

    /// Random provider selection
    async fn select_random(&self, providers: &[String]) -> Result<String> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..providers.len());
        debug!(
            "Random selected provider at index {}: {}",
            index, providers[index]
        );
        Ok(providers[index].clone())
    }

    /// Weighted provider selection
    async fn select_weighted(&self, providers: &[String]) -> Result<String> {
        let data = self.routing_data.read();

        // Calculate total weight
        let total_weight: f64 = providers
            .iter()
            .map(|p| data.weights.get(p).copied().unwrap_or(1.0))
            .sum();

        if total_weight <= 0.0 {
            drop(data); // Release lock before calling another method
            return self.select_round_robin(providers).await;
        }

        // Generate random number
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut random = rng.gen_range(0.0..1.0) * total_weight;

        // Select provider based on weight
        for provider in providers {
            let weight = data.weights.get(provider).copied().unwrap_or(1.0);
            random -= weight;
            if random <= 0.0 {
                debug!(
                    "Weighted selected provider: {} (weight: {})",
                    provider, weight
                );
                return Ok(provider.clone());
            }
        }

        // Fallback to first provider
        Ok(providers[0].clone())
    }

    /// Priority-based provider selection
    async fn select_priority(&self, providers: &[String]) -> Result<String> {
        let data = self.routing_data.read();

        let mut best_provider = &providers[0];
        let mut best_priority = 0u32;

        for provider in providers {
            let priority = data.priorities.get(provider).copied().unwrap_or(0);
            if priority > best_priority {
                best_priority = priority;
                best_provider = provider;
            }
        }

        debug!(
            "Priority selected provider: {} (priority: {})",
            best_provider, best_priority
        );
        Ok(best_provider.clone())
    }

    /// A/B test provider selection
    async fn select_ab_test(&self, providers: &[String], split_ratio: f64) -> Result<String> {
        if providers.len() < 2 {
            return Ok(providers[0].clone());
        }

        use rand::Rng;
        let mut rng = rand::thread_rng();
        let random = rng.gen_range(0.0..1.0);

        let selected = if random < split_ratio {
            &providers[0]
        } else {
            &providers[1]
        };

        debug!(
            "A/B test selected provider: {} (ratio: {}, random: {})",
            selected, split_ratio, random
        );
        Ok(selected.clone())
    }

    /// Custom strategy selection
    async fn select_custom(
        &self,
        providers: &[String],
        _logic: &str,
        _context: &RequestContext,
    ) -> Result<String> {
        // TODO: Implement custom strategy logic
        // For now, fallback to round-robin
        self.select_round_robin(providers).await
    }

    /// Update provider weight
    pub async fn update_weight(&self, provider: &str, weight: f64) -> Result<()> {
        self.routing_data
            .write()
            .weights
            .insert(provider.to_string(), weight);
        debug!("Updated weight for provider {}: {}", provider, weight);
        Ok(())
    }

    /// Update provider latency
    pub async fn update_latency(&self, provider: &str, latency: f64) -> Result<()> {
        self.routing_data
            .write()
            .latencies
            .insert(provider.to_string(), latency);
        debug!("Updated latency for provider {}: {}ms", provider, latency);
        Ok(())
    }

    /// Update provider cost
    pub async fn update_cost(&self, provider: &str, model: &str, cost: f64) -> Result<()> {
        let key = format!("{}:{}", provider, model);
        self.routing_data.write().costs.insert(key, cost);
        debug!(
            "Updated cost for provider {} model {}: ${:.4}",
            provider, model, cost
        );
        Ok(())
    }

    /// Update provider priority
    pub async fn update_priority(&self, provider: &str, priority: u32) -> Result<()> {
        self.routing_data
            .write()
            .priorities
            .insert(provider.to_string(), priority);
        debug!("Updated priority for provider {}: {}", provider, priority);
        Ok(())
    }
}
