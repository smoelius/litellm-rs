//! Provider configuration

use super::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Provider name
    pub name: String,
    /// Provider type (openai, anthropic, etc.)
    pub provider_type: String,
    /// API key
    pub api_key: String,
    /// Base URL
    pub base_url: Option<String>,
    /// API version
    pub api_version: Option<String>,
    /// Organization ID
    pub organization: Option<String>,
    /// Project ID
    pub project: Option<String>,
    /// Provider weight for load balancing
    #[serde(default = "default_weight")]
    pub weight: f32,
    /// Maximum requests per minute
    #[serde(default = "default_rpm")]
    pub rpm: u32,
    /// Maximum tokens per minute
    #[serde(default = "default_tpm")]
    pub tpm: u32,
    /// Maximum concurrent requests
    #[serde(default = "default_max_connections")]
    pub max_concurrent_requests: u32,
    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    /// Maximum retries
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    /// Retry configuration
    #[serde(default)]
    pub retry: RetryConfig,
    /// Health check configuration
    #[serde(default)]
    pub health_check: HealthCheckConfig,
    /// Provider-specific settings
    #[serde(default)]
    pub settings: HashMap<String, serde_json::Value>,
    /// Supported models
    #[serde(default)]
    pub models: Vec<String>,
    /// Tags for grouping providers
    #[serde(default)]
    pub tags: Vec<String>,
    /// Whether provider is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            provider_type: String::new(),
            api_key: String::new(),
            base_url: None,
            api_version: None,
            organization: None,
            project: None,
            weight: default_weight(),
            rpm: default_rpm(),
            tpm: default_tpm(),
            max_concurrent_requests: default_max_connections(),
            timeout: default_timeout(),
            max_retries: default_max_retries(),
            retry: RetryConfig::default(),
            health_check: HealthCheckConfig::default(),
            settings: HashMap::new(),
            models: Vec::new(),
            tags: Vec::new(),
            enabled: true,
        }
    }
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Base delay in milliseconds
    #[serde(default = "default_base_delay")]
    pub base_delay: u64,
    /// Maximum delay in milliseconds
    #[serde(default = "default_max_delay")]
    pub max_delay: u64,
    /// Backoff multiplier
    #[serde(default = "default_backoff_multiplier")]
    pub backoff_multiplier: f64,
    /// Jitter factor (0.0 to 1.0)
    #[serde(default)]
    pub jitter: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            base_delay: default_base_delay(),
            max_delay: default_max_delay(),
            backoff_multiplier: default_backoff_multiplier(),
            jitter: 0.1,
        }
    }
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Health check interval in seconds
    #[serde(default = "default_health_check_interval")]
    pub interval: u64,
    /// Failure threshold before marking unhealthy
    #[serde(default = "default_failure_threshold")]
    pub failure_threshold: u32,
    /// Recovery timeout in seconds
    #[serde(default = "default_recovery_timeout")]
    pub recovery_timeout: u64,
    /// Health check endpoint path
    pub endpoint: Option<String>,
    /// Expected status codes for healthy response
    #[serde(default)]
    pub expected_codes: Vec<u16>,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            interval: default_health_check_interval(),
            failure_threshold: default_failure_threshold(),
            recovery_timeout: default_recovery_timeout(),
            endpoint: None,
            expected_codes: vec![200],
        }
    }
}

fn default_true() -> bool {
    true
}
