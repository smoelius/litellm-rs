//! Routing strategy types and definitions

use std::collections::HashMap;

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
pub(super) struct RoutingData {
    /// Provider weights for weighted strategy
    pub weights: HashMap<String, f64>,
    /// Provider latencies for latency-based routing
    pub latencies: HashMap<String, f64>,
    /// Provider costs for cost-based routing
    pub costs: HashMap<String, f64>,
    /// Provider priorities
    pub priorities: HashMap<String, u32>,
    /// Provider usage metrics for usage-based routing
    pub usage: HashMap<String, ProviderUsage>,
}
