//! Error recovery and resilience utilities
//!
//! This module provides utilities for error recovery, circuit breakers, and resilience patterns.

use crate::utils::error::{GatewayError, Result};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{debug, error, warn};

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

/// Circuit breaker implementation
#[allow(dead_code)]
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: Arc<Mutex<CircuitState>>,
    failure_count: AtomicU32,
    success_count: AtomicU32,
    last_failure_time: Arc<Mutex<Option<Instant>>>,
    request_count: AtomicU32,
    window_start: Arc<Mutex<Instant>>,
}

#[allow(dead_code)]
impl CircuitBreaker {
    /// Create a new circuit breaker
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: Arc::new(Mutex::new(CircuitState::Closed)),
            failure_count: AtomicU32::new(0),
            success_count: AtomicU32::new(0),
            last_failure_time: Arc::new(Mutex::new(None)),
            request_count: AtomicU32::new(0),
            window_start: Arc::new(Mutex::new(Instant::now())),
        }
    }

    /// Execute a function with circuit breaker protection
    pub async fn call<F, R, E>(&self, f: F) -> Result<R>
    where
        F: std::future::Future<Output = std::result::Result<R, E>>,
        E: std::fmt::Display + std::fmt::Debug,
    {
        // Check if circuit should allow the request
        if !self.can_execute().await {
            return Err(GatewayError::ProviderUnavailable(
                "Circuit breaker is open".to_string(),
            ));
        }

        self.request_count.fetch_add(1, Ordering::Relaxed);

        match f.await {
            Ok(result) => {
                self.on_success().await;
                Ok(result)
            }
            Err(error) => {
                self.on_failure().await;
                Err(GatewayError::External(format!(
                    "Circuit breaker protected call failed: {}",
                    error
                )))
            }
        }
    }

    /// Check if the circuit breaker allows execution
    async fn can_execute(&self) -> bool {
        let mut state = self.state.lock().unwrap();

        match *state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if timeout has passed
                if let Some(last_failure) = *self.last_failure_time.lock().unwrap() {
                    if last_failure.elapsed() >= self.config.timeout {
                        debug!("Circuit breaker transitioning from Open to HalfOpen");
                        *state = CircuitState::HalfOpen;
                        self.success_count.store(0, Ordering::Relaxed);
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true,
        }
    }

    /// Handle successful request
    async fn on_success(&self) {
        let success_count = self.success_count.fetch_add(1, Ordering::Relaxed) + 1;

        let mut state = self.state.lock().unwrap();
        if *state == CircuitState::HalfOpen && success_count >= self.config.success_threshold {
            debug!("Circuit breaker transitioning from HalfOpen to Closed");
            *state = CircuitState::Closed;
            self.failure_count.store(0, Ordering::Relaxed);
            self.success_count.store(0, Ordering::Relaxed);
        }
    }

    /// Handle failed request
    async fn on_failure(&self) {
        let failure_count = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;
        let request_count = self.request_count.load(Ordering::Relaxed);

        *self.last_failure_time.lock().unwrap() = Some(Instant::now());

        let mut state = self.state.lock().unwrap();

        // Update window if needed
        {
            let mut window_start = self.window_start.lock().unwrap();
            if window_start.elapsed() >= self.config.window_size {
                *window_start = Instant::now();
                self.failure_count.store(1, Ordering::Relaxed);
                self.request_count.store(1, Ordering::Relaxed);
                return;
            }
        }

        // Check if we should open the circuit
        if request_count >= self.config.min_requests
            && failure_count >= self.config.failure_threshold
            && *state != CircuitState::Open
        {
            warn!(
                "Circuit breaker opening due to {} failures out of {} requests",
                failure_count, request_count
            );
            *state = CircuitState::Open;
        }

        // Always open from half-open on failure
        if *state == CircuitState::HalfOpen {
            debug!("Circuit breaker transitioning from HalfOpen to Open due to failure");
            *state = CircuitState::Open;
        }
    }

    /// Get current circuit breaker state
    pub fn state(&self) -> CircuitState {
        self.state.lock().unwrap().clone()
    }

    /// Get current metrics
    pub fn metrics(&self) -> CircuitBreakerMetrics {
        CircuitBreakerMetrics {
            state: self.state(),
            failure_count: self.failure_count.load(Ordering::Relaxed),
            success_count: self.success_count.load(Ordering::Relaxed),
            request_count: self.request_count.load(Ordering::Relaxed),
        }
    }

    /// Reset the circuit breaker
    pub fn reset(&self) {
        let mut state = self.state.lock().unwrap();
        *state = CircuitState::Closed;
        self.failure_count.store(0, Ordering::Relaxed);
        self.success_count.store(0, Ordering::Relaxed);
        self.request_count.store(0, Ordering::Relaxed);
        *self.last_failure_time.lock().unwrap() = None;
        *self.window_start.lock().unwrap() = Instant::now();
        debug!("Circuit breaker reset");
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

/// Retry mechanism with exponential backoff
#[allow(dead_code)]
pub struct RetryPolicy {
    config: RetryConfig,
}

#[allow(dead_code)]
impl RetryPolicy {
    /// Create a new retry policy
    pub fn new(config: RetryConfig) -> Self {
        Self { config }
    }

    /// Execute a function with retry logic
    pub async fn call<F, Fut, R, E>(&self, mut f: F) -> std::result::Result<R, E>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = std::result::Result<R, E>>,
        E: std::fmt::Display + std::fmt::Debug,
    {
        let mut attempt = 0;
        let mut delay = self.config.base_delay;

        loop {
            attempt += 1;

            match f().await {
                Ok(result) => {
                    if attempt > 1 {
                        debug!("Retry succeeded on attempt {}", attempt);
                    }
                    return Ok(result);
                }
                Err(error) => {
                    if attempt >= self.config.max_attempts {
                        error!("Retry failed after {} attempts: {}", attempt, error);
                        return Err(error);
                    }

                    debug!(
                        "Attempt {} failed: {}, retrying in {:?}",
                        attempt, error, delay
                    );

                    // Sleep with optional jitter
                    let actual_delay = if self.config.jitter {
                        let jitter_factor = 0.1;
                        let jitter = delay.as_millis() as f64
                            * jitter_factor
                            * (rand::random::<f64>() - 0.5);
                        Duration::from_millis((delay.as_millis() as f64 + jitter) as u64)
                    } else {
                        delay
                    };

                    tokio::time::sleep(actual_delay).await;

                    // Calculate next delay with exponential backoff
                    delay = std::cmp::min(
                        Duration::from_millis(
                            (delay.as_millis() as f64 * self.config.backoff_multiplier) as u64,
                        ),
                        self.config.max_delay,
                    );
                }
            }
        }
    }
}

/// Timeout wrapper for async operations
#[allow(dead_code)]
pub struct TimeoutWrapper {
    timeout: Duration,
}

#[allow(dead_code)]
impl TimeoutWrapper {
    /// Create a new timeout wrapper
    pub fn new(timeout: Duration) -> Self {
        Self { timeout }
    }

    /// Execute a function with timeout protection
    pub async fn call<F, R>(&self, f: F) -> Result<R>
    where
        F: std::future::Future<Output = R>,
    {
        match tokio::time::timeout(self.timeout, f).await {
            Ok(result) => Ok(result),
            Err(_) => Err(GatewayError::Timeout(format!(
                "Operation timed out after {:?}",
                self.timeout
            ))),
        }
    }
}

/// Bulkhead pattern for resource isolation
#[allow(dead_code)]
pub struct Bulkhead {
    semaphore: Arc<tokio::sync::Semaphore>,
    name: String,
    max_concurrent: usize,
}

#[allow(dead_code)]
impl Bulkhead {
    /// Create a new bulkhead
    pub fn new(name: String, max_concurrent: usize) -> Self {
        Self {
            semaphore: Arc::new(tokio::sync::Semaphore::new(max_concurrent)),
            name,
            max_concurrent,
        }
    }

    /// Execute a function with bulkhead protection
    pub async fn call<F, R>(&self, f: F) -> Result<R>
    where
        F: std::future::Future<Output = Result<R>>,
    {
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|e| GatewayError::Internal(format!("Bulkhead acquire failed: {}", e)))?;

        debug!("Bulkhead '{}' acquired permit", self.name);

        let result = f.await;

        debug!("Bulkhead '{}' released permit", self.name);

        result
    }

    /// Get available permits
    pub fn available_permits(&self) -> usize {
        self.semaphore.available_permits()
    }

    /// Get maximum concurrent operations
    pub fn max_concurrent(&self) -> usize {
        self.max_concurrent
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicU32;

    #[tokio::test]
    async fn test_circuit_breaker_success() {
        let config = CircuitBreakerConfig::default();
        let breaker = CircuitBreaker::new(config);

        let result = breaker.call(async { Ok::<i32, &str>(42) }).await;
        assert!(result.is_ok());
        assert_eq!(breaker.state(), CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_failure() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            min_requests: 2,
            ..Default::default()
        };

        let breaker = CircuitBreaker::new(config);

        // First failure
        let _ = breaker.call(async { Err::<i32, &str>("error") }).await;
        assert_eq!(breaker.state(), CircuitState::Closed);

        // Second failure should open circuit
        let _ = breaker.call(async { Err::<i32, &str>("error") }).await;
        assert_eq!(breaker.state(), CircuitState::Open);

        // Next call should be rejected
        let result = breaker.call(async { Ok::<i32, &str>(42) }).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_retry_policy() {
        let config = RetryConfig {
            max_attempts: 3,
            base_delay: Duration::from_millis(1),
            ..Default::default()
        };
        let policy = RetryPolicy::new(config);

        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = policy
            .call(|| {
                let counter = counter_clone.clone();
                async move {
                    let count = counter.fetch_add(1, Ordering::Relaxed);
                    if count < 2 { Err("not yet") } else { Ok(42) }
                }
            })
            .await;

        assert_eq!(result, Ok(42));
        assert_eq!(counter.load(Ordering::Relaxed), 3);
    }

    #[tokio::test]
    async fn test_timeout_wrapper() {
        let wrapper = TimeoutWrapper::new(Duration::from_millis(10));

        // Fast operation should succeed
        let result = wrapper.call(async { 42 }).await;
        assert!(result.is_ok());

        // Slow operation should timeout
        let result = wrapper
            .call(async {
                tokio::time::sleep(Duration::from_millis(20)).await;
                42
            })
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_bulkhead() {
        let bulkhead = Bulkhead::new("test".to_string(), 2);

        assert_eq!(bulkhead.available_permits(), 2);

        let result = bulkhead.call(async { Ok(42) }).await;
        assert!(result.is_ok());

        assert_eq!(bulkhead.available_permits(), 2); // Permit should be released
    }
}
