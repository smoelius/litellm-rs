//! Provider selection methods for different routing strategies

use super::types::RoutingData;
use crate::core::types::common::RequestContext;
use crate::utils::error::Result;
use parking_lot::RwLock;
use std::sync::atomic::{AtomicUsize, Ordering};
use tracing::debug;

/// Selection methods for strategy executor
pub(super) struct SelectionMethods;

impl SelectionMethods {
    /// Round-robin provider selection
    pub fn select_round_robin(
        providers: &[String],
        counter: &AtomicUsize,
    ) -> Result<String> {
        let index = counter.fetch_add(1, Ordering::Relaxed) % providers.len();
        debug!(
            "Round-robin selected provider at index {}: {}",
            index, providers[index]
        );
        Ok(providers[index].clone())
    }

    /// Select provider with least latency
    pub fn select_least_latency(
        providers: &[String],
        routing_data: &RwLock<RoutingData>,
    ) -> Result<String> {
        let data = routing_data.read();

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
    pub fn select_least_cost(
        providers: &[String],
        model: &str,
        routing_data: &RwLock<RoutingData>,
    ) -> Result<String> {
        let data = routing_data.read();

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
    pub fn select_random(providers: &[String]) -> Result<String> {
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
    pub fn select_weighted(
        providers: &[String],
        routing_data: &RwLock<RoutingData>,
        counter: &AtomicUsize,
    ) -> Result<String> {
        // Collect weights and calculate total within lock scope
        let (total_weight, weights): (f64, Vec<(String, f64)>) = {
            let data = routing_data.read();
            let weights: Vec<(String, f64)> = providers
                .iter()
                .map(|p| (p.clone(), data.weights.get(p).copied().unwrap_or(1.0)))
                .collect();
            let total: f64 = weights.iter().map(|(_, w)| w).sum();
            (total, weights)
        }; // Lock released here

        if total_weight <= 0.0 {
            return Self::select_round_robin(providers, counter);
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
    pub fn select_priority(
        providers: &[String],
        routing_data: &RwLock<RoutingData>,
    ) -> Result<String> {
        let data = routing_data.read();

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
    pub fn select_ab_test(providers: &[String], split_ratio: f64) -> Result<String> {
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
    pub fn select_custom(
        providers: &[String],
        _logic: &str,
        _context: &RequestContext,
        counter: &AtomicUsize,
    ) -> Result<String> {
        // TODO: Implement custom strategy logic
        // For now, fallback to round-robin
        Self::select_round_robin(providers, counter)
    }

    /// Usage-based provider selection (lowest TPM/RPM usage)
    pub fn select_usage_based(
        providers: &[String],
        routing_data: &RwLock<RoutingData>,
    ) -> Result<String> {
        let data = routing_data.read();

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
    pub fn select_least_busy(
        providers: &[String],
        routing_data: &RwLock<RoutingData>,
    ) -> Result<String> {
        let data = routing_data.read();

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
}
