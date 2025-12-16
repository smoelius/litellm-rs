//! Monitoring configuration validators
//!
//! This module provides validation implementations for monitoring-related
//! configuration structures including MonitoringConfig, MetricsConfig,
//! TracingConfig, and HealthConfig.

use super::trait_def::Validate;
use crate::config::models::*;
use tracing::debug;

impl Validate for MonitoringConfig {
    fn validate(&self) -> Result<(), String> {
        debug!("Validating monitoring configuration");

        self.metrics.validate()?;
        self.tracing.validate()?;
        self.health.validate()?;

        Ok(())
    }
}

impl Validate for MetricsConfig {
    fn validate(&self) -> Result<(), String> {
        if self.enabled && self.port == 0 {
            return Err("Metrics port must be greater than 0 when metrics are enabled".to_string());
        }

        if self.path.is_empty() {
            return Err("Metrics path cannot be empty".to_string());
        }

        if !self.path.starts_with('/') {
            return Err("Metrics path must start with '/'".to_string());
        }

        Ok(())
    }
}

impl Validate for TracingConfig {
    fn validate(&self) -> Result<(), String> {
        if self.enabled && self.endpoint.is_none() {
            return Err("Tracing endpoint must be specified when tracing is enabled".to_string());
        }

        if self.service_name.is_empty() {
            return Err("Service name cannot be empty".to_string());
        }

        Ok(())
    }
}

impl Validate for HealthConfig {
    fn validate(&self) -> Result<(), String> {
        if self.path.is_empty() {
            return Err("Health check path cannot be empty".to_string());
        }

        if !self.path.starts_with('/') {
            return Err("Health check path must start with '/'".to_string());
        }

        Ok(())
    }
}
