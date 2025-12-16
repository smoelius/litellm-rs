//! Router configuration types
//!
//! This module defines configuration types for the router including
//! routing strategies and router settings.

/// Routing strategy enumeration
///
/// Defines how the router selects which deployment to use when multiple deployments
/// are available for the same model.
///
/// ## Strategies
///
/// - **SimpleShuffle**: Weighted random selection (default, good for even distribution)
/// - **LeastBusy**: Select deployment with fewest active requests (good for balanced load)
/// - **UsageBased**: Select deployment with lowest TPM usage rate (good for rate limit optimization)
/// - **LatencyBased**: Select deployment with lowest average latency (good for performance)
/// - **CostBased**: Select deployment with lowest cost (good for cost optimization)
/// - **RateLimitAware**: Avoid deployments near rate limits (good for avoiding 429s)
/// - **RoundRobin**: Simple round-robin selection (good for predictable distribution)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RoutingStrategy {
    /// Weighted random selection (considers deployment weights)
    #[default]
    SimpleShuffle,
    /// Select deployment with fewest active requests
    LeastBusy,
    /// Select deployment with lowest TPM usage rate
    UsageBased,
    /// Select deployment with lowest average latency
    LatencyBased,
    /// Select deployment with lowest cost
    CostBased,
    /// Avoid deployments near rate limits
    RateLimitAware,
    /// Simple round-robin selection
    RoundRobin,
}

/// Router configuration
///
/// Contains global settings for router behavior including retry policies,
/// cooldown parameters, and feature flags.
///
/// ## Defaults
///
/// - `routing_strategy`: SimpleShuffle
/// - `num_retries`: 3
/// - `retry_after_secs`: 0 (no delay between retries)
/// - `allowed_fails`: 3 (failures before cooldown)
/// - `cooldown_time_secs`: 5
/// - `timeout_secs`: 60
/// - `max_fallbacks`: 5
/// - `enable_pre_call_checks`: true
#[derive(Debug, Clone)]
pub struct RouterConfig {
    /// Routing strategy to use for deployment selection
    pub routing_strategy: RoutingStrategy,

    /// Number of retry attempts on failure (default: 3)
    pub num_retries: u32,

    /// Minimum seconds to wait between retries (default: 0)
    pub retry_after_secs: u64,

    /// Number of failures allowed before entering cooldown (default: 3)
    pub allowed_fails: u32,

    /// Cooldown duration in seconds (default: 5)
    pub cooldown_time_secs: u64,

    /// Default timeout for requests in seconds (default: 60)
    pub timeout_secs: u64,

    /// Maximum number of fallback attempts (default: 5)
    pub max_fallbacks: u32,

    /// Enable pre-call validation checks (default: true)
    pub enable_pre_call_checks: bool,
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            routing_strategy: RoutingStrategy::SimpleShuffle,
            num_retries: 3,
            retry_after_secs: 0,
            allowed_fails: 3,
            cooldown_time_secs: 5,
            timeout_secs: 60,
            max_fallbacks: 5,
            enable_pre_call_checks: true,
        }
    }
}
