//! Router configuration validators
//!
//! This module provides validation implementations for router-related configuration
//! structures including RouterConfig, CircuitBreakerConfig, and RetryConfig.

use super::trait_def::Validate;
use crate::config::models::*;
use tracing::debug;

impl Validate for RouterConfig {
    fn validate(&self) -> Result<(), String> {
        debug!("Validating router configuration");

        self.circuit_breaker.validate()?;
        self.load_balancer.validate()?;

        Ok(())
    }
}

impl Validate for CircuitBreakerConfig {
    fn validate(&self) -> Result<(), String> {
        if self.failure_threshold == 0 {
            return Err("Circuit breaker failure threshold must be greater than 0".to_string());
        }

        if self.recovery_timeout == 0 {
            return Err("Circuit breaker recovery timeout must be greater than 0".to_string());
        }

        if self.min_requests == 0 {
            return Err("Circuit breaker min requests must be greater than 0".to_string());
        }

        Ok(())
    }
}

impl Validate for RetryConfig {
    fn validate(&self) -> Result<(), String> {
        if self.base_delay == 0 {
            return Err("Retry base delay must be greater than 0".to_string());
        }

        if self.max_delay <= self.base_delay {
            return Err("Retry max delay must be greater than base delay".to_string());
        }

        if self.backoff_multiplier <= 1.0 {
            return Err("Retry backoff multiplier must be greater than 1.0".to_string());
        }

        Ok(())
    }
}

impl Validate for LoadBalancerConfig {
    fn validate(&self) -> Result<(), String> {
        // Basic validation for load balancer config
        // Specific validation can be added based on strategy type
        Ok(())
    }
}
