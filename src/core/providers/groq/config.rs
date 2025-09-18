//! Groq Provider Configuration
//!
//! Configuration for Groq API access including authentication and model settings.

use crate::core::traits::ProviderConfig;
use serde::{Deserialize, Serialize};

/// Groq provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroqConfig {
    /// API key for Groq authentication
    pub api_key: Option<String>,

    /// API base URL (default: https://api.groq.com/openai/v1)
    pub api_base: Option<String>,

    /// Organization ID for Groq
    pub organization_id: Option<String>,

    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout: u64,

    /// Maximum number of retries for failed requests
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// Whether to enable debug logging
    #[serde(default)]
    pub debug: bool,
}

impl Default for GroqConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            api_base: None,
            organization_id: None,
            timeout: default_timeout(),
            max_retries: default_max_retries(),
            debug: false,
        }
    }
}

impl ProviderConfig for GroqConfig {
    fn validate(&self) -> Result<(), String> {
        // API key can come from environment variable
        if self.api_key.is_none() && std::env::var("GROQ_API_KEY").is_err() {
            return Err(
                "Groq API key not provided and GROQ_API_KEY environment variable not set"
                    .to_string(),
            );
        }

        // Validate timeout
        if self.timeout == 0 {
            return Err("Timeout must be greater than 0".to_string());
        }

        Ok(())
    }

    fn api_key(&self) -> Option<&str> {
        self.api_key.as_deref()
    }

    fn api_base(&self) -> Option<&str> {
        self.api_base.as_deref()
    }

    fn timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.timeout)
    }

    fn max_retries(&self) -> u32 {
        self.max_retries
    }
}

impl GroqConfig {
    /// Get API key with environment variable fallback
    pub fn get_api_key(&self) -> Option<String> {
        self.api_key
            .clone()
            .or_else(|| std::env::var("GROQ_API_KEY").ok())
    }

    /// Get API base with environment variable fallback
    pub fn get_api_base(&self) -> String {
        self.api_base
            .clone()
            .or_else(|| std::env::var("GROQ_API_BASE").ok())
            .unwrap_or_else(|| "https://api.groq.com/openai/v1".to_string())
    }
}

fn default_timeout() -> u64 {
    30
}

fn default_max_retries() -> u32 {
    3
}
