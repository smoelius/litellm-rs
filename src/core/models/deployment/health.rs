//! Deployment health monitoring
//!
//! This module defines health status tracking and circuit breaker functionality.

use crate::core::models::HealthStatus;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, AtomicU64};

/// Deployment health information
#[derive(Debug)]
pub struct DeploymentHealth {
    /// Current health status
    pub status: parking_lot::RwLock<HealthStatus>,
    /// Last health check timestamp
    pub last_check: AtomicU64,
    /// Consecutive failure count
    pub failure_count: AtomicU32,
    /// Last failure timestamp
    pub last_failure: AtomicU64,
    /// Average response time in milliseconds
    pub avg_response_time: AtomicU64,
    /// Success rate (0-10000 for 0.00% to 100.00%)
    pub success_rate: AtomicU32,
    /// Circuit breaker state
    pub circuit_breaker: parking_lot::RwLock<CircuitBreakerState>,
}

/// Circuit breaker state
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CircuitBreakerState {
    /// Circuit is closed (normal operation)
    Closed,
    /// Circuit is open (failing fast)
    Open,
    /// Circuit is half-open (testing recovery)
    HalfOpen,
}

impl Default for DeploymentHealth {
    fn default() -> Self {
        Self::new()
    }
}

impl DeploymentHealth {
    /// Create new deployment health
    pub fn new() -> Self {
        Self {
            status: parking_lot::RwLock::new(HealthStatus::Unknown),
            last_check: AtomicU64::new(0),
            failure_count: AtomicU32::new(0),
            last_failure: AtomicU64::new(0),
            avg_response_time: AtomicU64::new(0),
            success_rate: AtomicU32::new(10000), // 100.00%
            circuit_breaker: parking_lot::RwLock::new(CircuitBreakerState::Closed),
        }
    }
}
