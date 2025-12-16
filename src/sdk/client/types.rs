//! Type definitions for the LLM client

use std::time::SystemTime;

/// Provider statistics
#[derive(Debug, Clone, Default)]
pub struct ProviderStats {
    pub requests: u64,
    pub errors: u64,
    pub total_tokens: u64,
    pub total_cost: f64,
    pub avg_latency_ms: f64,
    pub last_used: Option<SystemTime>,
    pub health_score: f64,
}

/// Load balancer
#[derive(Debug)]
pub struct LoadBalancer {
    pub(crate) strategy: LoadBalancingStrategy,
}

/// Load balancing strategy
#[derive(Debug, Clone)]
pub enum LoadBalancingStrategy {
    RoundRobin,
    LeastLatency,
    WeightedRandom,
    HealthBased,
}

impl LoadBalancer {
    pub(crate) fn new(strategy: LoadBalancingStrategy) -> Self {
        Self { strategy }
    }
}
