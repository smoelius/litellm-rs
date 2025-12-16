//! Provider configuration types

use super::defaults::*;
use super::health::HealthCheckConfig;
use super::rate_limit::RateLimitConfig;
use super::retry::RetryConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use url::Url;

/// Provider configuration entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfigEntry {
    /// Provider name (unique identifier)
    pub name: String,
    /// Provider type
    pub provider_type: String,
    /// Enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Routing weight (0.0-1.0)
    #[serde(default = "default_weight")]
    pub weight: f64,
    /// Provider-specific configuration
    pub config: serde_json::Value,
    /// Labels (for routing and filtering)
    #[serde(default)]
    pub tags: HashMap<String, String>,
    /// Health check configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health_check: Option<HealthCheckConfig>,
    /// Retry configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry: Option<RetryConfig>,
    /// Rate limit configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limit: Option<RateLimitConfig>,
}

/// OpenAI provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIProviderConfig {
    /// API key
    pub api_key: String,
    /// API base URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_base: Option<String>,
    /// Organization ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization: Option<String>,
    /// Request timeout (seconds)
    #[serde(default = "default_timeout_seconds")]
    pub timeout_seconds: u64,
    /// Maximum retries
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    /// Supported models
    #[serde(default)]
    pub models: Vec<String>,
    /// Custom headers
    #[serde(default)]
    pub headers: HashMap<String, String>,
}

impl crate::core::traits::ProviderConfig for OpenAIProviderConfig {
    fn validate(&self) -> Result<(), String> {
        if self.api_key.is_empty() {
            return Err("API key is required".to_string());
        }

        if let Some(base_url) = &self.api_base {
            if Url::parse(base_url).is_err() {
                return Err("Invalid API base URL".to_string());
            }
        }

        if self.timeout_seconds == 0 {
            return Err("Timeout must be greater than 0".to_string());
        }

        Ok(())
    }

    fn api_key(&self) -> Option<&str> {
        Some(&self.api_key)
    }

    fn api_base(&self) -> Option<&str> {
        self.api_base.as_deref()
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(self.timeout_seconds)
    }

    fn max_retries(&self) -> u32 {
        self.max_retries
    }
}
