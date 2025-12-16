//! Health status types and check results
//!
//! This module defines the core types for health monitoring including
//! health status levels and health check results.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Health status levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Service is fully operational
    Healthy,
    /// Service is operational but degraded
    Degraded,
    /// Service is unhealthy but may recover
    Unhealthy,
    /// Service is completely unavailable
    Down,
}

impl HealthStatus {
    /// Check if the status allows requests
    pub fn allows_requests(&self) -> bool {
        matches!(self, HealthStatus::Healthy | HealthStatus::Degraded)
    }

    /// Get numeric score for routing (higher is better)
    pub fn score(&self) -> u32 {
        match self {
            HealthStatus::Healthy => 100,
            HealthStatus::Degraded => 70,
            HealthStatus::Unhealthy => 30,
            HealthStatus::Down => 0,
        }
    }
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    /// Health status
    pub status: HealthStatus,
    /// Response time in milliseconds
    pub response_time_ms: u64,
    /// Timestamp of the check
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Additional details about the health check
    pub details: Option<String>,
    /// Error message if unhealthy
    pub error: Option<String>,
    /// Metrics collected during health check
    pub metrics: HashMap<String, f64>,
}

impl HealthCheckResult {
    /// Create a healthy result
    pub fn healthy(response_time_ms: u64) -> Self {
        Self {
            status: HealthStatus::Healthy,
            response_time_ms,
            timestamp: chrono::Utc::now(),
            details: None,
            error: None,
            metrics: HashMap::new(),
        }
    }

    /// Create an unhealthy result
    pub fn unhealthy(error: String, response_time_ms: u64) -> Self {
        Self {
            status: HealthStatus::Unhealthy,
            response_time_ms,
            timestamp: chrono::Utc::now(),
            details: None,
            error: Some(error),
            metrics: HashMap::new(),
        }
    }

    /// Create a degraded result
    pub fn degraded(reason: String, response_time_ms: u64) -> Self {
        Self {
            status: HealthStatus::Degraded,
            response_time_ms,
            timestamp: chrono::Utc::now(),
            details: Some(reason),
            error: None,
            metrics: HashMap::new(),
        }
    }
}
