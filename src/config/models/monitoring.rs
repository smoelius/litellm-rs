//! Monitoring configuration

use super::*;
use serde::{Deserialize, Serialize};

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MonitoringConfig {
    /// Metrics configuration
    #[serde(default)]
    pub metrics: MetricsConfig,
    /// Tracing configuration
    #[serde(default)]
    pub tracing: TracingConfig,
    /// Health check configuration
    #[serde(default)]
    pub health: HealthConfig,
}

#[allow(dead_code)]
impl MonitoringConfig {
    /// Merge monitoring configurations
    pub fn merge(mut self, other: Self) -> Self {
        self.metrics = self.metrics.merge(other.metrics);
        self.tracing = self.tracing.merge(other.tracing);
        self.health = self.health.merge(other.health);
        self
    }
}

/// Metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Enable metrics
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Metrics port
    #[serde(default = "default_metrics_port")]
    pub port: u16,
    /// Metrics path
    #[serde(default = "default_metrics_path")]
    pub path: String,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            port: default_metrics_port(),
            path: default_metrics_path(),
        }
    }
}

#[allow(dead_code)]
impl MetricsConfig {
    /// Merge metrics configurations
    pub fn merge(mut self, other: Self) -> Self {
        if !other.enabled {
            self.enabled = other.enabled;
        }
        if other.port != default_metrics_port() {
            self.port = other.port;
        }
        if other.path != default_metrics_path() {
            self.path = other.path;
        }
        self
    }
}

/// Tracing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingConfig {
    /// Enable tracing
    #[serde(default)]
    pub enabled: bool,
    /// Tracing endpoint
    pub endpoint: Option<String>,
    /// Service name
    #[serde(default = "default_service_name")]
    pub service_name: String,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            endpoint: None,
            service_name: default_service_name(),
        }
    }
}

#[allow(dead_code)]
impl TracingConfig {
    /// Merge tracing configurations
    pub fn merge(mut self, other: Self) -> Self {
        if other.enabled {
            self.enabled = other.enabled;
        }
        if other.endpoint.is_some() {
            self.endpoint = other.endpoint;
        }
        if other.service_name != default_service_name() {
            self.service_name = other.service_name;
        }
        self
    }
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthConfig {
    /// Health check path
    #[serde(default = "default_health_path")]
    pub path: String,
    /// Enable detailed health checks
    #[serde(default = "default_true")]
    pub detailed: bool,
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            path: default_health_path(),
            detailed: true,
        }
    }
}

#[allow(dead_code)]
impl HealthConfig {
    /// Merge health configurations
    pub fn merge(mut self, other: Self) -> Self {
        if other.path != default_health_path() {
            self.path = other.path;
        }
        if !other.detailed {
            self.detailed = other.detailed;
        }
        self
    }
}

fn default_true() -> bool {
    true
}
