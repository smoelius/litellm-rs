//! Types and configurations for error recovery patterns

use std::time::Duration;

/// Circuit breaker state
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum CircuitState {
    /// Circuit is closed, requests flow normally
    Closed,
    /// Circuit is open, requests are rejected
    Open,
    /// Circuit is half-open, allowing test requests
    HalfOpen,
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CircuitBreakerConfig {
    /// Failure threshold to open circuit
    pub failure_threshold: u32,
    /// Success threshold to close circuit from half-open
    pub success_threshold: u32,
    /// Minimum requests before considering failure rate
    pub min_requests: u32,
    /// Timeout before transitioning from open to half-open
    pub timeout: Duration,
    /// Window size for failure rate calculation
    pub window_size: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            min_requests: 10,
            timeout: Duration::from_secs(60),
            window_size: Duration::from_secs(60),
        }
    }
}

/// Circuit breaker metrics
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CircuitBreakerMetrics {
    /// Current circuit breaker state
    pub state: CircuitState,
    /// Number of consecutive failures
    pub failure_count: u32,
    /// Number of consecutive successes
    pub success_count: u32,
    /// Total number of requests processed
    pub request_count: u32,
}

/// Retry configuration
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Base delay between retries
    pub base_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Backoff multiplier
    pub backoff_multiplier: f64,
    /// Whether to add jitter to delays
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }
}
