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
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
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

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== RateLimitStrategy Tests ====================

    #[test]
    fn test_rate_limit_strategy_default() {
        let strategy = RateLimitStrategy::default();
        assert_eq!(strategy, RateLimitStrategy::TokenBucket);
    }

    #[test]
    fn test_rate_limit_strategy_variants() {
        let token_bucket = RateLimitStrategy::TokenBucket;
        let fixed_window = RateLimitStrategy::FixedWindow;
        let sliding_window = RateLimitStrategy::SlidingWindow;

        assert_eq!(token_bucket, RateLimitStrategy::TokenBucket);
        assert_eq!(fixed_window, RateLimitStrategy::FixedWindow);
        assert_eq!(sliding_window, RateLimitStrategy::SlidingWindow);
    }

    #[test]
    fn test_rate_limit_strategy_serialization() {
        assert_eq!(
            serde_json::to_string(&RateLimitStrategy::TokenBucket).unwrap(),
            "\"token_bucket\""
        );
        assert_eq!(
            serde_json::to_string(&RateLimitStrategy::FixedWindow).unwrap(),
            "\"fixed_window\""
        );
        assert_eq!(
            serde_json::to_string(&RateLimitStrategy::SlidingWindow).unwrap(),
            "\"sliding_window\""
        );
    }

    #[test]
    fn test_rate_limit_strategy_deserialization() {
        let token_bucket: RateLimitStrategy =
            serde_json::from_str("\"token_bucket\"").unwrap();
        assert_eq!(token_bucket, RateLimitStrategy::TokenBucket);

        let fixed_window: RateLimitStrategy =
            serde_json::from_str("\"fixed_window\"").unwrap();
        assert_eq!(fixed_window, RateLimitStrategy::FixedWindow);

        let sliding_window: RateLimitStrategy =
            serde_json::from_str("\"sliding_window\"").unwrap();
        assert_eq!(sliding_window, RateLimitStrategy::SlidingWindow);
    }

    #[test]
    fn test_rate_limit_strategy_clone() {
        let strategy = RateLimitStrategy::FixedWindow;
        let cloned = strategy.clone();
        assert_eq!(strategy, cloned);
    }

    // ==================== RateLimitConfig Default Tests ====================

    #[test]
    fn test_rate_limit_config_default() {
        let config = RateLimitConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.default_rpm, 1000);
        assert_eq!(config.default_tpm, 100_000);
        assert_eq!(config.strategy, RateLimitStrategy::TokenBucket);
    }

    #[test]
    fn test_rate_limit_config_structure() {
        let config = RateLimitConfig {
            enabled: true,
            default_rpm: 500,
            default_tpm: 50_000,
            strategy: RateLimitStrategy::SlidingWindow,
        };
        assert!(config.enabled);
        assert_eq!(config.default_rpm, 500);
        assert_eq!(config.default_tpm, 50_000);
        assert_eq!(config.strategy, RateLimitStrategy::SlidingWindow);
    }

    // ==================== RateLimitConfig Serialization Tests ====================

    #[test]
    fn test_rate_limit_config_serialization() {
        let config = RateLimitConfig {
            enabled: true,
            default_rpm: 600,
            default_tpm: 60_000,
            strategy: RateLimitStrategy::FixedWindow,
        };
        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["enabled"], true);
        assert_eq!(json["default_rpm"], 600);
        assert_eq!(json["default_tpm"], 60_000);
        assert_eq!(json["strategy"], "fixed_window");
    }

    #[test]
    fn test_rate_limit_config_deserialization() {
        let json = r#"{
            "enabled": true,
            "default_rpm": 200,
            "default_tpm": 20000,
            "strategy": "sliding_window"
        }"#;
        let config: RateLimitConfig = serde_json::from_str(json).unwrap();
        assert!(config.enabled);
        assert_eq!(config.default_rpm, 200);
        assert_eq!(config.default_tpm, 20000);
        assert_eq!(config.strategy, RateLimitStrategy::SlidingWindow);
    }

    #[test]
    fn test_rate_limit_config_deserialization_defaults() {
        let json = r#"{}"#;
        let config: RateLimitConfig = serde_json::from_str(json).unwrap();
        assert!(!config.enabled);
        assert_eq!(config.default_rpm, 1000);
        assert_eq!(config.default_tpm, 100_000);
    }

    // ==================== RateLimitConfig Merge Tests ====================

    #[test]
    fn test_rate_limit_config_merge_enabled() {
        let base = RateLimitConfig::default();
        let other = RateLimitConfig {
            enabled: true,
            default_rpm: 1000,
            default_tpm: 100_000,
            strategy: RateLimitStrategy::TokenBucket,
        };
        let merged = base.merge(other);
        assert!(merged.enabled);
    }

    #[test]
    fn test_rate_limit_config_merge_rpm() {
        let base = RateLimitConfig::default();
        let other = RateLimitConfig {
            enabled: false,
            default_rpm: 500,
            default_tpm: 100_000,
            strategy: RateLimitStrategy::TokenBucket,
        };
        let merged = base.merge(other);
        assert_eq!(merged.default_rpm, 500);
    }

    #[test]
    fn test_rate_limit_config_merge_tpm() {
        let base = RateLimitConfig::default();
        let other = RateLimitConfig {
            enabled: false,
            default_rpm: 1000,
            default_tpm: 50_000,
            strategy: RateLimitStrategy::TokenBucket,
        };
        let merged = base.merge(other);
        assert_eq!(merged.default_tpm, 50_000);
    }

    #[test]
    fn test_rate_limit_config_merge_strategy() {
        let base = RateLimitConfig::default();
        let other = RateLimitConfig {
            enabled: false,
            default_rpm: 1000,
            default_tpm: 100_000,
            strategy: RateLimitStrategy::SlidingWindow,
        };
        let merged = base.merge(other);
        assert_eq!(merged.strategy, RateLimitStrategy::SlidingWindow);
    }

    #[test]
    fn test_rate_limit_config_merge_no_change() {
        let base = RateLimitConfig::default();
        let other = RateLimitConfig::default();
        let merged = base.merge(other);
        assert!(!merged.enabled);
        assert_eq!(merged.default_rpm, 1000);
        assert_eq!(merged.default_tpm, 100_000);
    }

    // ==================== RateLimitConfig Clone Tests ====================

    #[test]
    fn test_rate_limit_config_clone() {
        let config = RateLimitConfig {
            enabled: true,
            default_rpm: 750,
            default_tpm: 75_000,
            strategy: RateLimitStrategy::FixedWindow,
        };
        let cloned = config.clone();
        assert_eq!(config.enabled, cloned.enabled);
        assert_eq!(config.default_rpm, cloned.default_rpm);
        assert_eq!(config.default_tpm, cloned.default_tpm);
        assert_eq!(config.strategy, cloned.strategy);
    }
}
