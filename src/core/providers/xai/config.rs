//! xAI Provider Configuration

use serde::{Deserialize, Serialize};
use crate::core::traits::ProviderConfig;

/// xAI provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XAIConfig {
    /// API key for authentication
    pub api_key: Option<String>,

    /// API base URL (defaults to https://api.x.ai)
    pub api_base: Option<String>,

    /// Organization ID (optional)
    pub organization_id: Option<String>,

    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout: u64,

    /// Maximum number of retries
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// Enable debug mode
    #[serde(default)]
    pub debug: bool,

    /// Enable web search capability for Grok models
    #[serde(default = "default_web_search")]
    pub enable_web_search: bool,
}

impl Default for XAIConfig {
    fn default() -> Self {
        Self {
            api_key: std::env::var("XAI_API_KEY").ok(),
            api_base: None,
            organization_id: None,
            timeout: default_timeout(),
            max_retries: default_max_retries(),
            debug: false,
            enable_web_search: default_web_search(),
        }
    }
}

impl ProviderConfig for XAIConfig {
    fn validate(&self) -> Result<(), String> {
        if self.api_key.is_none() {
            return Err("XAI API key is required".to_string());
        }

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

impl XAIConfig {
    /// Get the API base URL
    pub fn get_api_base(&self) -> String {
        self.api_base.clone()
            .or_else(|| std::env::var("XAI_API_BASE").ok())
            .unwrap_or_else(|| "https://api.x.ai/v1".to_string())
    }

    /// Get the API key
    pub fn get_api_key(&self) -> Option<String> {
        self.api_key.clone()
            .or_else(|| std::env::var("XAI_API_KEY").ok())
    }
}

fn default_timeout() -> u64 {
    30
}

fn default_max_retries() -> u32 {
    3
}

fn default_web_search() -> bool {
    true
}