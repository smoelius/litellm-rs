//! Deployment runtime metrics
//!
//! This module defines metrics tracking for deployment monitoring and observability.

use std::sync::atomic::{AtomicU32, AtomicU64};

/// Deployment runtime metrics
#[derive(Debug)]
pub struct DeploymentMetrics {
    /// Total requests processed
    pub total_requests: AtomicU64,
    /// Successful requests
    pub successful_requests: AtomicU64,
    /// Failed requests
    pub failed_requests: AtomicU64,
    /// Total tokens processed
    pub total_tokens: AtomicU64,
    /// Total cost incurred
    pub total_cost: parking_lot::RwLock<f64>,
    /// Active connections
    pub active_connections: AtomicU32,
    /// Queue size
    pub queue_size: AtomicU32,
    /// Last request timestamp
    pub last_request: AtomicU64,
    /// Request rate (requests per minute)
    pub request_rate: AtomicU32,
    /// Token rate (tokens per minute)
    pub token_rate: AtomicU32,
    /// Average response time
    pub avg_response_time: AtomicU64,
    /// P95 response time
    pub p95_response_time: AtomicU64,
    /// P99 response time
    pub p99_response_time: AtomicU64,
}

impl Default for DeploymentMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl DeploymentMetrics {
    /// Create new deployment metrics
    pub fn new() -> Self {
        Self {
            total_requests: AtomicU64::new(0),
            successful_requests: AtomicU64::new(0),
            failed_requests: AtomicU64::new(0),
            total_tokens: AtomicU64::new(0),
            total_cost: parking_lot::RwLock::new(0.0),
            active_connections: AtomicU32::new(0),
            queue_size: AtomicU32::new(0),
            last_request: AtomicU64::new(0),
            request_rate: AtomicU32::new(0),
            token_rate: AtomicU32::new(0),
            avg_response_time: AtomicU64::new(0),
            p95_response_time: AtomicU64::new(0),
            p99_response_time: AtomicU64::new(0),
        }
    }
}
