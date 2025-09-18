//! Cloudflare Workers AI Configuration

use crate::core::traits::ProviderConfig;
use serde::{Deserialize, Serialize};

/// Cloudflare Workers AI provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudflareConfig {
    /// Cloudflare account ID
    pub account_id: Option<String>,

    /// API token for authentication
    pub api_token: Option<String>,

    /// API base URL (defaults to https://api.cloudflare.com/client/v4)
    pub api_base: Option<String>,

    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout: u64,

    /// Maximum number of retries
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// Enable debug mode
    #[serde(default)]
    pub debug: bool,
}

impl Default for CloudflareConfig {
    fn default() -> Self {
        Self {
            account_id: std::env::var("CLOUDFLARE_ACCOUNT_ID").ok(),
            api_token: std::env::var("CLOUDFLARE_API_TOKEN").ok(),
            api_base: None,
            timeout: default_timeout(),
            max_retries: default_max_retries(),
            debug: false,
        }
    }
}

impl ProviderConfig for CloudflareConfig {
    fn validate(&self) -> Result<(), String> {
        if self.account_id.is_none() {
            return Err("Cloudflare account ID is required".to_string());
        }

        if self.api_token.is_none() {
            return Err("Cloudflare API token is required".to_string());
        }

        if self.timeout == 0 {
            return Err("Timeout must be greater than 0".to_string());
        }

        Ok(())
    }

    fn api_key(&self) -> Option<&str> {
        self.api_token.as_deref()
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

impl CloudflareConfig {
    /// Get the API base URL
    pub fn get_api_base(&self) -> String {
        self.api_base
            .clone()
            .or_else(|| std::env::var("CLOUDFLARE_API_BASE").ok())
            .unwrap_or_else(|| "https://api.cloudflare.com/client/v4".to_string())
    }

    /// Get the account ID
    pub fn get_account_id(&self) -> Option<String> {
        self.account_id
            .clone()
            .or_else(|| std::env::var("CLOUDFLARE_ACCOUNT_ID").ok())
    }

    /// Get the API token
    pub fn get_api_token(&self) -> Option<String> {
        self.api_token
            .clone()
            .or_else(|| std::env::var("CLOUDFLARE_API_TOKEN").ok())
    }
}

fn default_timeout() -> u64 {
    30
}

fn default_max_retries() -> u32 {
    3
}
