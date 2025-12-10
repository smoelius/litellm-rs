//! Routing strategies for provider selection

use crate::core::types::common::RequestContext;
use crate::utils::error::{GatewayError, Result};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use tracing::{debug, info};

/// Routing strategies for provider selection
#[derive(Debug, Clone, Default)]
pub enum RoutingStrategy {
    /// Round-robin selection
    #[default]
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
    /// Route to provider with lowest TPM/RPM usage
    UsageBased,
    /// Route to provider with fewest active concurrent requests
    LeastBusy,
    /// Custom strategy with user-defined logic
    Custom(String),
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

/// Usage metrics for a provider
#[derive(Debug, Clone, Default)]
pub struct ProviderUsage {
    /// Tokens per minute (TPM) usage
    pub tpm: u64,
    /// Requests per minute (RPM) usage
    pub rpm: u64,
    /// Active concurrent requests
    pub active_requests: usize,
    /// TPM limit (if known)
    pub tpm_limit: Option<u64>,
    /// RPM limit (if known)
    pub rpm_limit: Option<u64>,
}

impl ProviderUsage {
    /// Calculate usage percentage (0.0 to 1.0) based on limits
    pub fn usage_percentage(&self) -> f64 {
        let tpm_pct = self
            .tpm_limit
            .map(|limit| self.tpm as f64 / limit as f64)
            .unwrap_or(0.0);
        let rpm_pct = self
            .rpm_limit
            .map(|limit| self.rpm as f64 / limit as f64)
            .unwrap_or(0.0);

        // Return the higher of the two percentages
        tpm_pct.max(rpm_pct)
    }
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
    /// Provider usage metrics for usage-based routing
    usage: HashMap<String, ProviderUsage>,
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
            RoutingStrategy::UsageBased => self.select_usage_based(available_providers).await,
            RoutingStrategy::LeastBusy => self.select_least_busy(available_providers).await,
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

        // Pre-allocate buffer for cost key to avoid repeated allocations in loop
        let mut cost_key = String::with_capacity(64);
        for provider in providers {
            cost_key.clear();
            cost_key.push_str(provider);
            cost_key.push(':');
            cost_key.push_str(model);

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
        // Collect weights and calculate total within lock scope
        let (total_weight, weights): (f64, Vec<(String, f64)>) = {
            let data = self.routing_data.read();
            let weights: Vec<(String, f64)> = providers
                .iter()
                .map(|p| (p.clone(), data.weights.get(p).copied().unwrap_or(1.0)))
                .collect();
            let total: f64 = weights.iter().map(|(_, w)| w).sum();
            (total, weights)
        }; // Lock released here

        if total_weight <= 0.0 {
            return self.select_round_robin(providers).await;
        }

        // Generate random number
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut random = rng.gen_range(0.0..1.0) * total_weight;

        // Select provider based on weight
        for (provider, weight) in &weights {
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

    /// Usage-based provider selection (lowest TPM/RPM usage)
    async fn select_usage_based(&self, providers: &[String]) -> Result<String> {
        let data = self.routing_data.read();

        let mut best_provider = &providers[0];
        let mut best_usage_pct = f64::MAX;

        for provider in providers {
            let usage_pct = data
                .usage
                .get(provider)
                .map(|u| u.usage_percentage())
                .unwrap_or(0.0); // No usage data = 0% usage

            if usage_pct < best_usage_pct {
                best_usage_pct = usage_pct;
                best_provider = provider;
            }
        }

        debug!(
            "Usage-based selected provider: {} (usage: {:.1}%)",
            best_provider,
            best_usage_pct * 100.0
        );
        Ok(best_provider.clone())
    }

    /// Least-busy provider selection (fewest active requests)
    async fn select_least_busy(&self, providers: &[String]) -> Result<String> {
        let data = self.routing_data.read();

        let mut best_provider = &providers[0];
        let mut least_active = usize::MAX;

        for provider in providers {
            let active = data
                .usage
                .get(provider)
                .map(|u| u.active_requests)
                .unwrap_or(0); // No usage data = 0 active requests

            if active < least_active {
                least_active = active;
                best_provider = provider;
            }
        }

        debug!(
            "Least-busy selected provider: {} (active requests: {})",
            best_provider, least_active
        );
        Ok(best_provider.clone())
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

    /// Update provider usage metrics
    pub async fn update_usage(&self, provider: &str, usage: ProviderUsage) -> Result<()> {
        let (tpm, rpm, active) = (usage.tpm, usage.rpm, usage.active_requests);
        self.routing_data
            .write()
            .usage
            .insert(provider.to_string(), usage);
        debug!(
            "Updated usage for provider {}: TPM={}, RPM={}, active={}",
            provider, tpm, rpm, active
        );
        Ok(())
    }

    /// Increment active request count for a provider
    pub async fn increment_active_requests(&self, provider: &str) -> Result<()> {
        let mut data = self.routing_data.write();
        let usage = data.usage.entry(provider.to_string()).or_default();
        usage.active_requests += 1;
        debug!(
            "Incremented active requests for {}: now {}",
            provider, usage.active_requests
        );
        Ok(())
    }

    /// Decrement active request count for a provider
    pub async fn decrement_active_requests(&self, provider: &str) -> Result<()> {
        let mut data = self.routing_data.write();
        if let Some(usage) = data.usage.get_mut(provider) {
            usage.active_requests = usage.active_requests.saturating_sub(1);
            debug!(
                "Decremented active requests for {}: now {}",
                provider, usage.active_requests
            );
        }
        Ok(())
    }

    /// Record token usage for a provider (updates TPM tracking)
    pub async fn record_token_usage(&self, provider: &str, tokens: u64) -> Result<()> {
        let mut data = self.routing_data.write();
        let usage = data.usage.entry(provider.to_string()).or_default();
        usage.tpm += tokens;
        usage.rpm += 1;
        debug!(
            "Recorded token usage for {}: +{} tokens (TPM: {}, RPM: {})",
            provider, tokens, usage.tpm, usage.rpm
        );
        Ok(())
    }

    /// Set rate limits for a provider
    pub async fn set_rate_limits(
        &self,
        provider: &str,
        tpm_limit: Option<u64>,
        rpm_limit: Option<u64>,
    ) -> Result<()> {
        let mut data = self.routing_data.write();
        let usage = data.usage.entry(provider.to_string()).or_default();
        usage.tpm_limit = tpm_limit;
        usage.rpm_limit = rpm_limit;
        debug!(
            "Set rate limits for {}: TPM={:?}, RPM={:?}",
            provider, tpm_limit, rpm_limit
        );
        Ok(())
    }

    /// Reset usage counters (typically called at the start of each minute)
    pub async fn reset_usage_counters(&self) -> Result<()> {
        let mut data = self.routing_data.write();
        for usage in data.usage.values_mut() {
            usage.tpm = 0;
            usage.rpm = 0;
        }
        debug!("Reset usage counters for all providers");
        Ok(())
    }

    /// Get current usage for a provider
    pub async fn get_usage(&self, provider: &str) -> Option<ProviderUsage> {
        self.routing_data.read().usage.get(provider).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_usage_percentage() {
        let mut usage = ProviderUsage::default();

        // No limits set = 0%
        assert_eq!(usage.usage_percentage(), 0.0);

        // Set limits and usage
        usage.tpm = 5000;
        usage.tpm_limit = Some(10000);
        usage.rpm = 50;
        usage.rpm_limit = Some(100);

        // Should be 50% (both at 50%)
        assert!((usage.usage_percentage() - 0.5).abs() < 0.001);

        // TPM at 80%, RPM at 50% -> should return 80%
        usage.tpm = 8000;
        assert!((usage.usage_percentage() - 0.8).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_usage_based_routing() {
        let executor = StrategyExecutor::new(RoutingStrategy::UsageBased)
            .await
            .unwrap();

        // Set up usage data
        executor
            .update_usage(
                "provider_a",
                ProviderUsage {
                    tpm: 8000,
                    rpm: 80,
                    active_requests: 5,
                    tpm_limit: Some(10000),
                    rpm_limit: Some(100),
                },
            )
            .await
            .unwrap();

        executor
            .update_usage(
                "provider_b",
                ProviderUsage {
                    tpm: 2000,
                    rpm: 20,
                    active_requests: 2,
                    tpm_limit: Some(10000),
                    rpm_limit: Some(100),
                },
            )
            .await
            .unwrap();

        let providers = vec!["provider_a".to_string(), "provider_b".to_string()];
        let context = RequestContext::default();

        // Should select provider_b (20% usage vs 80% usage)
        let selected = executor
            .select_provider(&providers, "gpt-4", &context)
            .await
            .unwrap();
        assert_eq!(selected, "provider_b");
    }

    #[tokio::test]
    async fn test_least_busy_routing() {
        let executor = StrategyExecutor::new(RoutingStrategy::LeastBusy)
            .await
            .unwrap();

        // Set up usage data
        executor
            .update_usage(
                "provider_a",
                ProviderUsage {
                    active_requests: 10,
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        executor
            .update_usage(
                "provider_b",
                ProviderUsage {
                    active_requests: 3,
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        let providers = vec!["provider_a".to_string(), "provider_b".to_string()];
        let context = RequestContext::default();

        // Should select provider_b (3 active vs 10 active)
        let selected = executor
            .select_provider(&providers, "gpt-4", &context)
            .await
            .unwrap();
        assert_eq!(selected, "provider_b");
    }

    #[tokio::test]
    async fn test_active_request_tracking() {
        let executor = StrategyExecutor::new(RoutingStrategy::LeastBusy)
            .await
            .unwrap();

        executor
            .increment_active_requests("provider_a")
            .await
            .unwrap();
        executor
            .increment_active_requests("provider_a")
            .await
            .unwrap();
        executor
            .increment_active_requests("provider_a")
            .await
            .unwrap();

        let usage = executor.get_usage("provider_a").await.unwrap();
        assert_eq!(usage.active_requests, 3);

        executor
            .decrement_active_requests("provider_a")
            .await
            .unwrap();
        let usage = executor.get_usage("provider_a").await.unwrap();
        assert_eq!(usage.active_requests, 2);
    }

    #[tokio::test]
    async fn test_token_usage_recording() {
        let executor = StrategyExecutor::new(RoutingStrategy::UsageBased)
            .await
            .unwrap();

        executor
            .record_token_usage("provider_a", 1000)
            .await
            .unwrap();
        executor
            .record_token_usage("provider_a", 500)
            .await
            .unwrap();

        let usage = executor.get_usage("provider_a").await.unwrap();
        assert_eq!(usage.tpm, 1500);
        assert_eq!(usage.rpm, 2);
    }

    #[tokio::test]
    async fn test_usage_counter_reset() {
        let executor = StrategyExecutor::new(RoutingStrategy::UsageBased)
            .await
            .unwrap();

        executor
            .record_token_usage("provider_a", 1000)
            .await
            .unwrap();
        executor
            .increment_active_requests("provider_a")
            .await
            .unwrap();

        executor.reset_usage_counters().await.unwrap();

        let usage = executor.get_usage("provider_a").await.unwrap();
        assert_eq!(usage.tpm, 0);
        assert_eq!(usage.rpm, 0);
        // Active requests should NOT be reset
        assert_eq!(usage.active_requests, 1);
    }
}
