//! Service status types

use super::health::HealthStatus;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// API version
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApiVersion {
    /// v1 version
    #[default]
    V1,
    /// v2 version (future extension)
    V2,
}

impl std::fmt::Display for ApiVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::V1 => write!(f, "v1"),
            Self::V2 => write!(f, "v2"),
        }
    }
}

/// Service status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStatus {
    /// Service name
    pub name: String,
    /// Version
    pub version: String,
    /// Status
    pub status: HealthStatus,
    /// Start time
    pub uptime: SystemTime,
    /// Active connections
    pub active_connections: u32,
    /// Requests processed
    pub requests_processed: u64,
    /// Errors count
    pub errors: u64,
    /// Average response time (ms)
    pub avg_response_time_ms: f64,
    /// Memory usage (bytes)
    pub memory_usage_bytes: u64,
    /// CPU usage rate (percentage)
    pub cpu_usage_percent: f64,
}
