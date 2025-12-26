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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;

    // ==================== DeploymentHealth Tests ====================

    #[test]
    fn test_deployment_health_new() {
        let health = DeploymentHealth::new();

        assert_eq!(health.last_check.load(Ordering::Relaxed), 0);
        assert_eq!(health.failure_count.load(Ordering::Relaxed), 0);
        assert_eq!(health.last_failure.load(Ordering::Relaxed), 0);
        assert_eq!(health.avg_response_time.load(Ordering::Relaxed), 0);
        assert_eq!(health.success_rate.load(Ordering::Relaxed), 10000);
    }

    #[test]
    fn test_deployment_health_default() {
        let health = DeploymentHealth::default();

        assert_eq!(health.success_rate.load(Ordering::Relaxed), 10000);
        assert_eq!(health.failure_count.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_deployment_health_initial_status() {
        let health = DeploymentHealth::new();
        let status = health.status.read();

        match *status {
            HealthStatus::Unknown => (),
            _ => panic!("Expected Unknown status"),
        }
    }

    #[test]
    fn test_deployment_health_initial_circuit_breaker() {
        let health = DeploymentHealth::new();
        let cb_state = health.circuit_breaker.read();

        match *cb_state {
            CircuitBreakerState::Closed => (),
            _ => panic!("Expected Closed circuit breaker"),
        }
    }

    #[test]
    fn test_deployment_health_atomic_operations() {
        let health = DeploymentHealth::new();

        // Test failure count increment
        health.failure_count.fetch_add(1, Ordering::Relaxed);
        assert_eq!(health.failure_count.load(Ordering::Relaxed), 1);

        health.failure_count.fetch_add(5, Ordering::Relaxed);
        assert_eq!(health.failure_count.load(Ordering::Relaxed), 6);

        // Test failure count reset
        health.failure_count.store(0, Ordering::Relaxed);
        assert_eq!(health.failure_count.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_deployment_health_response_time() {
        let health = DeploymentHealth::new();

        health.avg_response_time.store(150, Ordering::Relaxed);
        assert_eq!(health.avg_response_time.load(Ordering::Relaxed), 150);

        // Simulate average calculation
        let current = health.avg_response_time.load(Ordering::Relaxed);
        let new_time = 200;
        let avg = (current + new_time) / 2;
        health.avg_response_time.store(avg, Ordering::Relaxed);
        assert_eq!(health.avg_response_time.load(Ordering::Relaxed), 175);
    }

    #[test]
    fn test_deployment_health_success_rate() {
        let health = DeploymentHealth::new();

        // Initial 100%
        assert_eq!(health.success_rate.load(Ordering::Relaxed), 10000);

        // Decrease to 95%
        health.success_rate.store(9500, Ordering::Relaxed);
        assert_eq!(health.success_rate.load(Ordering::Relaxed), 9500);

        // Decrease to 50%
        health.success_rate.store(5000, Ordering::Relaxed);
        assert_eq!(health.success_rate.load(Ordering::Relaxed), 5000);
    }

    #[test]
    fn test_deployment_health_status_update() {
        let health = DeploymentHealth::new();

        // Update to healthy
        {
            let mut status = health.status.write();
            *status = HealthStatus::Healthy;
        }

        {
            let status = health.status.read();
            match *status {
                HealthStatus::Healthy => (),
                _ => panic!("Expected Healthy status"),
            }
        }

        // Update to unhealthy
        {
            let mut status = health.status.write();
            *status = HealthStatus::Unhealthy;
        }

        {
            let status = health.status.read();
            match *status {
                HealthStatus::Unhealthy => (),
                _ => panic!("Expected Unhealthy status"),
            }
        }
    }

    #[test]
    fn test_deployment_health_circuit_breaker_transitions() {
        let health = DeploymentHealth::new();

        // Start closed
        {
            let cb = health.circuit_breaker.read();
            assert!(matches!(*cb, CircuitBreakerState::Closed));
        }

        // Transition to open
        {
            let mut cb = health.circuit_breaker.write();
            *cb = CircuitBreakerState::Open;
        }
        {
            let cb = health.circuit_breaker.read();
            assert!(matches!(*cb, CircuitBreakerState::Open));
        }

        // Transition to half-open
        {
            let mut cb = health.circuit_breaker.write();
            *cb = CircuitBreakerState::HalfOpen;
        }
        {
            let cb = health.circuit_breaker.read();
            assert!(matches!(*cb, CircuitBreakerState::HalfOpen));
        }

        // Transition back to closed
        {
            let mut cb = health.circuit_breaker.write();
            *cb = CircuitBreakerState::Closed;
        }
        {
            let cb = health.circuit_breaker.read();
            assert!(matches!(*cb, CircuitBreakerState::Closed));
        }
    }

    #[test]
    fn test_deployment_health_timestamp_updates() {
        let health = DeploymentHealth::new();

        // Simulate timestamp updates
        let timestamp = 1700000000u64;

        health.last_check.store(timestamp, Ordering::Relaxed);
        assert_eq!(health.last_check.load(Ordering::Relaxed), timestamp);

        health.last_failure.store(timestamp + 100, Ordering::Relaxed);
        assert_eq!(health.last_failure.load(Ordering::Relaxed), timestamp + 100);
    }

    // ==================== CircuitBreakerState Tests ====================

    #[test]
    fn test_circuit_breaker_state_closed() {
        let state = CircuitBreakerState::Closed;
        let json = serde_json::to_string(&state).unwrap();
        assert!(json.contains("closed"));
    }

    #[test]
    fn test_circuit_breaker_state_open() {
        let state = CircuitBreakerState::Open;
        let json = serde_json::to_string(&state).unwrap();
        assert!(json.contains("open"));
    }

    #[test]
    fn test_circuit_breaker_state_half_open() {
        let state = CircuitBreakerState::HalfOpen;
        let json = serde_json::to_string(&state).unwrap();
        assert!(json.contains("half_open"));
    }

    #[test]
    fn test_circuit_breaker_state_clone() {
        let state = CircuitBreakerState::Open;
        let cloned = state.clone();

        let json1 = serde_json::to_string(&state).unwrap();
        let json2 = serde_json::to_string(&cloned).unwrap();
        assert_eq!(json1, json2);
    }

    #[test]
    fn test_circuit_breaker_state_serialization_roundtrip() {
        let states = vec![
            CircuitBreakerState::Closed,
            CircuitBreakerState::Open,
            CircuitBreakerState::HalfOpen,
        ];

        for state in states {
            let json = serde_json::to_string(&state).unwrap();
            let deserialized: CircuitBreakerState = serde_json::from_str(&json).unwrap();

            let json1 = serde_json::to_string(&state).unwrap();
            let json2 = serde_json::to_string(&deserialized).unwrap();
            assert_eq!(json1, json2);
        }
    }

    // ==================== Concurrent Access Tests ====================

    #[test]
    fn test_deployment_health_concurrent_atomic_updates() {
        use std::sync::Arc;
        use std::thread;

        let health = Arc::new(DeploymentHealth::new());
        let mut handles = vec![];

        for _ in 0..10 {
            let health = Arc::clone(&health);
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    health.failure_count.fetch_add(1, Ordering::Relaxed);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // Should have 10 threads * 100 increments = 1000
        assert_eq!(health.failure_count.load(Ordering::Relaxed), 1000);
    }

    #[test]
    fn test_deployment_health_concurrent_status_access() {
        use std::sync::Arc;
        use std::thread;

        let health = Arc::new(DeploymentHealth::new());
        let mut handles = vec![];

        // Writers
        for _ in 0..5 {
            let health = Arc::clone(&health);
            let handle = thread::spawn(move || {
                for _ in 0..10 {
                    let mut status = health.status.write();
                    *status = HealthStatus::Healthy;
                }
            });
            handles.push(handle);
        }

        // Readers
        for _ in 0..5 {
            let health = Arc::clone(&health);
            let handle = thread::spawn(move || {
                for _ in 0..10 {
                    let _status = health.status.read();
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // Should not panic or deadlock
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_deployment_health_max_values() {
        let health = DeploymentHealth::new();

        health.failure_count.store(u32::MAX, Ordering::Relaxed);
        assert_eq!(health.failure_count.load(Ordering::Relaxed), u32::MAX);

        health.avg_response_time.store(u64::MAX, Ordering::Relaxed);
        assert_eq!(health.avg_response_time.load(Ordering::Relaxed), u64::MAX);
    }

    #[test]
    fn test_deployment_health_success_rate_boundaries() {
        let health = DeploymentHealth::new();

        // 0%
        health.success_rate.store(0, Ordering::Relaxed);
        assert_eq!(health.success_rate.load(Ordering::Relaxed), 0);

        // 100%
        health.success_rate.store(10000, Ordering::Relaxed);
        assert_eq!(health.success_rate.load(Ordering::Relaxed), 10000);
    }
}
