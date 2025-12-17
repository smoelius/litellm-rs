//! Health checking types and data structures

use std::collections::HashMap;
use std::time::Duration;

/// Overall system health status
#[derive(Debug, Clone, serde::Serialize)]
pub struct HealthStatus {
    /// Whether the system is overall healthy
    pub overall_healthy: bool,
    /// Timestamp of last health check
    pub last_check: chrono::DateTime<chrono::Utc>,
    /// Individual component health
    pub components: HashMap<String, ComponentHealth>,
    /// System uptime
    pub uptime_seconds: u64,
    /// Health check summary
    pub summary: HealthSummary,
}

/// Individual component health
#[derive(Debug, Clone, serde::Serialize)]
pub struct ComponentHealth {
    /// Component name
    pub name: String,
    /// Whether the component is healthy
    pub healthy: bool,
    /// Health status message
    pub status: String,
    /// Last check timestamp
    pub last_check: chrono::DateTime<chrono::Utc>,
    /// Response time for health check
    pub response_time_ms: u64,
    /// Error message (if unhealthy)
    pub error: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Health check summary
#[derive(Debug, Clone, serde::Serialize)]
pub struct HealthSummary {
    /// Total number of components
    pub total_components: usize,
    /// Number of healthy components
    pub healthy_components: usize,
    /// Number of unhealthy components
    pub unhealthy_components: usize,
    /// Health percentage
    pub health_percentage: f64,
}

/// Health check configuration for a component
#[derive(Debug, Clone)]
pub struct HealthCheckConfig {
    /// Component name
    pub name: String,
    /// Check interval
    pub interval: Duration,
    /// Timeout for health check
    pub timeout: Duration,
    /// Number of retries
    pub retries: u32,
    /// Whether this component is critical
    pub critical: bool,
}

/// Consolidated health data - single lock for all health-related state
#[derive(Debug)]
pub(super) struct HealthData {
    /// Component health status
    pub components: HashMap<String, ComponentHealth>,
    /// Overall health status
    pub overall: HealthStatus,
}
