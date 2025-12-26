//! Rate limit configuration types

use serde::{Deserialize, Serialize};

/// Rate limit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Algorithm type
    pub algorithm: RateLimitAlgorithm,
    /// Requests per second
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requests_per_second: Option<u32>,
    /// Requests per minute
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requests_per_minute: Option<u32>,
    /// Tokens per minute
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens_per_minute: Option<u32>,
    /// Burst size
    #[serde(skip_serializing_if = "Option::is_none")]
    pub burst_size: Option<u32>,
}

/// Rate limit algorithm
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RateLimitAlgorithm {
    TokenBucket,
    SlidingWindow,
    FixedWindow,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== RateLimitAlgorithm Tests ====================

    #[test]
    fn test_rate_limit_algorithm_variants() {
        let token_bucket = RateLimitAlgorithm::TokenBucket;
        let sliding_window = RateLimitAlgorithm::SlidingWindow;
        let fixed_window = RateLimitAlgorithm::FixedWindow;

        assert_eq!(token_bucket, RateLimitAlgorithm::TokenBucket);
        assert_eq!(sliding_window, RateLimitAlgorithm::SlidingWindow);
        assert_eq!(fixed_window, RateLimitAlgorithm::FixedWindow);
    }

    #[test]
    fn test_rate_limit_algorithm_serialization() {
        assert_eq!(
            serde_json::to_string(&RateLimitAlgorithm::TokenBucket).unwrap(),
            "\"token_bucket\""
        );
        assert_eq!(
            serde_json::to_string(&RateLimitAlgorithm::SlidingWindow).unwrap(),
            "\"sliding_window\""
        );
        assert_eq!(
            serde_json::to_string(&RateLimitAlgorithm::FixedWindow).unwrap(),
            "\"fixed_window\""
        );
    }

    #[test]
    fn test_rate_limit_algorithm_deserialization() {
        let token_bucket: RateLimitAlgorithm =
            serde_json::from_str("\"token_bucket\"").unwrap();
        assert_eq!(token_bucket, RateLimitAlgorithm::TokenBucket);

        let sliding_window: RateLimitAlgorithm =
            serde_json::from_str("\"sliding_window\"").unwrap();
        assert_eq!(sliding_window, RateLimitAlgorithm::SlidingWindow);

        let fixed_window: RateLimitAlgorithm =
            serde_json::from_str("\"fixed_window\"").unwrap();
        assert_eq!(fixed_window, RateLimitAlgorithm::FixedWindow);
    }

    #[test]
    fn test_rate_limit_algorithm_clone() {
        let algorithm = RateLimitAlgorithm::SlidingWindow;
        let cloned = algorithm.clone();
        assert_eq!(algorithm, cloned);
    }

    // ==================== RateLimitConfig Tests ====================

    #[test]
    fn test_rate_limit_config_structure() {
        let config = RateLimitConfig {
            algorithm: RateLimitAlgorithm::TokenBucket,
            requests_per_second: Some(100),
            requests_per_minute: Some(6000),
            tokens_per_minute: Some(100000),
            burst_size: Some(200),
        };
        assert_eq!(config.algorithm, RateLimitAlgorithm::TokenBucket);
        assert_eq!(config.requests_per_second, Some(100));
        assert_eq!(config.requests_per_minute, Some(6000));
        assert_eq!(config.tokens_per_minute, Some(100000));
        assert_eq!(config.burst_size, Some(200));
    }

    #[test]
    fn test_rate_limit_config_minimal() {
        let config = RateLimitConfig {
            algorithm: RateLimitAlgorithm::FixedWindow,
            requests_per_second: None,
            requests_per_minute: Some(1000),
            tokens_per_minute: None,
            burst_size: None,
        };
        assert!(config.requests_per_second.is_none());
        assert!(config.tokens_per_minute.is_none());
        assert!(config.burst_size.is_none());
    }

    #[test]
    fn test_rate_limit_config_serialization() {
        let config = RateLimitConfig {
            algorithm: RateLimitAlgorithm::SlidingWindow,
            requests_per_second: Some(50),
            requests_per_minute: None,
            tokens_per_minute: Some(50000),
            burst_size: Some(100),
        };
        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["algorithm"], "sliding_window");
        assert_eq!(json["requests_per_second"], 50);
        assert_eq!(json["tokens_per_minute"], 50000);
        assert_eq!(json["burst_size"], 100);
    }

    #[test]
    fn test_rate_limit_config_skip_none_serialization() {
        let config = RateLimitConfig {
            algorithm: RateLimitAlgorithm::TokenBucket,
            requests_per_second: None,
            requests_per_minute: Some(1000),
            tokens_per_minute: None,
            burst_size: None,
        };
        let json = serde_json::to_value(&config).unwrap();
        let obj = json.as_object().unwrap();
        assert!(!obj.contains_key("requests_per_second"));
        assert!(!obj.contains_key("tokens_per_minute"));
        assert!(!obj.contains_key("burst_size"));
        assert!(obj.contains_key("requests_per_minute"));
    }

    #[test]
    fn test_rate_limit_config_deserialization() {
        let json = r#"{
            "algorithm": "fixed_window",
            "requests_per_second": 25,
            "requests_per_minute": 1500,
            "burst_size": 50
        }"#;
        let config: RateLimitConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.algorithm, RateLimitAlgorithm::FixedWindow);
        assert_eq!(config.requests_per_second, Some(25));
        assert_eq!(config.requests_per_minute, Some(1500));
        assert_eq!(config.burst_size, Some(50));
    }

    #[test]
    fn test_rate_limit_config_deserialization_minimal() {
        let json = r#"{"algorithm": "token_bucket"}"#;
        let config: RateLimitConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.algorithm, RateLimitAlgorithm::TokenBucket);
        assert!(config.requests_per_second.is_none());
        assert!(config.requests_per_minute.is_none());
    }

    #[test]
    fn test_rate_limit_config_clone() {
        let config = RateLimitConfig {
            algorithm: RateLimitAlgorithm::TokenBucket,
            requests_per_second: Some(100),
            requests_per_minute: Some(6000),
            tokens_per_minute: Some(100000),
            burst_size: Some(200),
        };
        let cloned = config.clone();
        assert_eq!(config.algorithm, cloned.algorithm);
        assert_eq!(config.requests_per_second, cloned.requests_per_second);
        assert_eq!(config.burst_size, cloned.burst_size);
    }
}
