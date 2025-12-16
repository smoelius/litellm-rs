//! Health check types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

/// Health status
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    /// Healthy
    Healthy,
    /// Unhealthy
    Unhealthy,
    /// Unknown status
    #[default]
    Unknown,
    /// Degraded service
    Degraded,
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    /// Status
    pub status: HealthStatus,
    /// Check time
    pub checked_at: SystemTime,
    /// Latency (milliseconds)
    pub latency_ms: Option<u64>,
    /// Error message
    pub error: Option<String>,
    /// Extra details
    pub details: HashMap<String, serde_json::Value>,
}

impl Default for HealthCheckResult {
    fn default() -> Self {
        Self {
            status: HealthStatus::Unknown,
            checked_at: SystemTime::now(),
            latency_ms: None,
            error: None,
            details: HashMap::new(),
        }
    }
}
