//! Router configuration

use super::*;
use serde::{Deserialize, Serialize};

/// Router configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RouterConfig {
    /// Routing strategy
    #[serde(default)]
    pub strategy: RoutingStrategyConfig,
    /// Circuit breaker configuration
    #[serde(default)]
    pub circuit_breaker: CircuitBreakerConfig,
    /// Load balancer configuration
    #[serde(default)]
    pub load_balancer: LoadBalancerConfig,
}

#[allow(dead_code)]
impl RouterConfig {
    /// Merge router configurations
    pub fn merge(mut self, other: Self) -> Self {
        self.strategy = other.strategy;
        self.circuit_breaker = self.circuit_breaker.merge(other.circuit_breaker);
        self.load_balancer = self.load_balancer.merge(other.load_balancer);
        self
    }
}

/// Routing strategy configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RoutingStrategyConfig {
    /// Round-robin routing
    #[default]
    RoundRobin,
    /// Least latency routing
    LeastLatency,
    /// Least cost routing
    LeastCost,
    /// Random routing
    Random,
    /// Weighted routing
    Weighted {
        /// Provider weights
        weights: std::collections::HashMap<String, f64>,
    },
    /// Priority-based routing
    Priority {
        /// Provider priorities
        priorities: std::collections::HashMap<String, u32>,
    },
    /// A/B testing
    ABTest {
        /// Traffic split ratio
        split_ratio: f64,
    },
    /// Custom strategy
    Custom {
        /// Custom logic identifier
        logic: String,
    },
}

/// Circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Failure threshold
    #[serde(default = "default_failure_threshold")]
    pub failure_threshold: u32,
    /// Recovery timeout in seconds
    #[serde(default = "default_recovery_timeout")]
    pub recovery_timeout: u64,
    /// Minimum requests before circuit breaker activates
    #[serde(default = "default_min_requests")]
    pub min_requests: u32,
    /// Success threshold for half-open state
    #[serde(default = "default_success_threshold")]
    pub success_threshold: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: default_failure_threshold(),
            recovery_timeout: default_recovery_timeout(),
            min_requests: default_min_requests(),
            success_threshold: 3,
        }
    }
}

#[allow(dead_code)]
impl CircuitBreakerConfig {
    /// Merge circuit breaker configurations
    pub fn merge(mut self, other: Self) -> Self {
        if other.failure_threshold != default_failure_threshold() {
            self.failure_threshold = other.failure_threshold;
        }
        if other.recovery_timeout != default_recovery_timeout() {
            self.recovery_timeout = other.recovery_timeout;
        }
        if other.min_requests != default_min_requests() {
            self.min_requests = other.min_requests;
        }
        if other.success_threshold != 3 {
            self.success_threshold = other.success_threshold;
        }
        self
    }
}

/// Load balancer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancerConfig {
    /// Health check enabled
    #[serde(default = "default_true")]
    pub health_check_enabled: bool,
    /// Sticky sessions enabled
    #[serde(default)]
    pub sticky_sessions: bool,
    /// Session timeout in seconds
    #[serde(default = "default_session_timeout")]
    pub session_timeout: u64,
}

impl Default for LoadBalancerConfig {
    fn default() -> Self {
        Self {
            health_check_enabled: true,
            sticky_sessions: false,
            session_timeout: 3600,
        }
    }
}

#[allow(dead_code)]
impl LoadBalancerConfig {
    /// Merge load balancer configurations
    pub fn merge(mut self, other: Self) -> Self {
        if !other.health_check_enabled {
            self.health_check_enabled = other.health_check_enabled;
        }
        if other.sticky_sessions {
            self.sticky_sessions = other.sticky_sessions;
        }
        if other.session_timeout != 3600 {
            self.session_timeout = other.session_timeout;
        }
        self
    }
}

fn default_success_threshold() -> u32 {
    3
}

fn default_session_timeout() -> u64 {
    3600
}

fn default_true() -> bool {
    true
}
