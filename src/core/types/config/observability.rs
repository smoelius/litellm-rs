//! Observability configuration types

use super::defaults::*;
use serde::{Deserialize, Serialize};

/// Observability configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    /// Metrics configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<MetricsConfig>,
    /// Tracing configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracing: Option<TracingConfig>,
    /// Logging configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logging: Option<LoggingConfig>,
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            metrics: Some(MetricsConfig {
                enabled: true,
                endpoint: default_metrics_endpoint(),
                interval_seconds: default_metrics_interval(),
            }),
            tracing: Some(TracingConfig {
                enabled: true,
                sampling_rate: default_sampling_rate(),
                jaeger: None,
            }),
            logging: Some(LoggingConfig {
                level: default_log_level(),
                format: default_log_format(),
                outputs: vec![LogOutput::Console],
            }),
        }
    }
}

/// Metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Endpoint path
    #[serde(default = "default_metrics_endpoint")]
    pub endpoint: String,
    /// Collection interval (seconds)
    #[serde(default = "default_metrics_interval")]
    pub interval_seconds: u64,
}

/// Tracing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingConfig {
    /// Enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Sampling rate (0.0-1.0)
    #[serde(default = "default_sampling_rate")]
    pub sampling_rate: f64,
    /// Jaeger configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jaeger: Option<JaegerConfig>,
}

/// Jaeger configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JaegerConfig {
    /// Agent endpoint
    pub agent_endpoint: String,
    /// Service name
    pub service_name: String,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level
    #[serde(default = "default_log_level")]
    pub level: String,
    /// Output format
    #[serde(default = "default_log_format")]
    pub format: LogFormat,
    /// Output targets
    #[serde(default)]
    pub outputs: Vec<LogOutput>,
}

/// Log format
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    Text,
    Json,
    Structured,
}

/// Log output
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum LogOutput {
    #[serde(rename = "console")]
    Console,
    #[serde(rename = "file")]
    File { path: String },
    #[serde(rename = "syslog")]
    Syslog { facility: String },
}
