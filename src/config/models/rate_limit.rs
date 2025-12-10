//! Rate limiting configuration

use super::*;
use serde::{Deserialize, Serialize};

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Enable rate limiting
    #[serde(default)]
    pub enabled: bool,
    /// Default requests per minute
    #[serde(default = "default_rpm")]
    pub default_rpm: u32,
    /// Default tokens per minute
    #[serde(default = "default_tpm")]
    pub default_tpm: u32,
    /// Rate limiting strategy
    #[serde(default)]
    pub strategy: RateLimitStrategy,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            default_rpm: default_rpm(),
            default_tpm: default_tpm(),
            strategy: RateLimitStrategy::default(),
        }
    }
}

#[allow(dead_code)]
impl RateLimitConfig {
    /// Merge rate limit configurations
    pub fn merge(mut self, other: Self) -> Self {
        if other.enabled {
            self.enabled = other.enabled;
        }
        if other.default_rpm != default_rpm() {
            self.default_rpm = other.default_rpm;
        }
        if other.default_tpm != default_tpm() {
            self.default_tpm = other.default_tpm;
        }
        self.strategy = other.strategy;
        self
    }
}

/// Rate limiting strategy
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RateLimitStrategy {
    /// Token bucket algorithm
    #[default]
    TokenBucket,
    /// Fixed window
    FixedWindow,
    /// Sliding window
    SlidingWindow,
}
