//! Error recovery and resilience utilities
//!
//! This module provides utilities for error recovery, circuit breakers, and resilience patterns.

mod circuit_breaker;
mod resilience;
mod retry;
mod types;

// Re-export all public types and structs for backward compatibility
pub use circuit_breaker::CircuitBreaker;
pub use resilience::{Bulkhead, TimeoutWrapper};
pub use retry::RetryPolicy;
pub use types::{CircuitBreakerConfig, CircuitBreakerMetrics, CircuitState, RetryConfig};

// Include tests module
#[cfg(test)]
mod tests;
