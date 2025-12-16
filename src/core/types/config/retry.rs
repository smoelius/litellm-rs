//! Retry configuration types

use super::defaults::*;
use serde::{Deserialize, Serialize};

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum retries
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    /// Initial delay (milliseconds)
    #[serde(default = "default_initial_delay_ms")]
    pub initial_delay_ms: u64,
    /// Maximum delay (milliseconds)
    #[serde(default = "default_max_delay_ms")]
    pub max_delay_ms: u64,
    /// Use exponential backoff
    #[serde(default = "default_true")]
    pub exponential_backoff: bool,
    /// Add random jitter
    #[serde(default = "default_true")]
    pub jitter: bool,
    /// Retryable error types
    #[serde(default)]
    pub retryable_errors: Vec<String>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: default_max_retries(),
            initial_delay_ms: default_initial_delay_ms(),
            max_delay_ms: default_max_delay_ms(),
            exponential_backoff: true,
            jitter: true,
            retryable_errors: vec![
                "network_error".to_string(),
                "timeout_error".to_string(),
                "rate_limit_error".to_string(),
                "server_error".to_string(),
            ],
        }
    }
}
