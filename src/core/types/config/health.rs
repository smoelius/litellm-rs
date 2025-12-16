//! Health check configuration types

use super::defaults::*;
use serde::{Deserialize, Serialize};

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Check interval (seconds)
    #[serde(default = "default_health_check_interval")]
    pub interval_seconds: u64,
    /// Timeout duration (seconds)
    #[serde(default = "default_health_check_timeout")]
    pub timeout_seconds: u64,
    /// Healthy threshold (consecutive successes)
    #[serde(default = "default_health_threshold")]
    pub healthy_threshold: u32,
    /// Unhealthy threshold (consecutive failures)
    #[serde(default = "default_unhealthy_threshold")]
    pub unhealthy_threshold: u32,
    /// Health check endpoint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    /// Enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            interval_seconds: default_health_check_interval(),
            timeout_seconds: default_health_check_timeout(),
            healthy_threshold: default_health_threshold(),
            unhealthy_threshold: default_unhealthy_threshold(),
            endpoint: None,
            enabled: true,
        }
    }
}
