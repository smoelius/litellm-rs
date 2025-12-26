//! Token counter types and configurations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Model token counting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelTokenConfig {
    /// Model name
    pub model: String,
    /// Average characters per token
    pub chars_per_token: f64,
    /// Overhead tokens per message
    pub message_overhead: u32,
    /// Overhead tokens per request
    pub request_overhead: u32,
    /// Maximum context window
    pub max_context_tokens: u32,
    /// Special token handling
    pub special_tokens: HashMap<String, u32>,
}

/// Token estimation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenEstimate {
    /// Estimated input tokens
    pub input_tokens: u32,
    /// Estimated output tokens (if applicable)
    pub output_tokens: Option<u32>,
    /// Total estimated tokens
    pub total_tokens: u32,
    /// Whether the estimate is approximate
    pub is_approximate: bool,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f64,
}

impl ModelTokenConfig {
    /// Create default model configurations
    pub(super) fn default_configs() -> HashMap<String, ModelTokenConfig> {
        let mut configs = HashMap::new();

        // GPT-4 family
        configs.insert(
            "gpt-4".to_string(),
            ModelTokenConfig {
                model: "gpt-4".to_string(),
                chars_per_token: 4.0,
                message_overhead: 3,
                request_overhead: 3,
                max_context_tokens: 8192,
                special_tokens: HashMap::new(),
            },
        );

        // GPT-3.5 family
        configs.insert(
            "gpt-3.5-turbo".to_string(),
            ModelTokenConfig {
                model: "gpt-3.5-turbo".to_string(),
                chars_per_token: 4.0,
                message_overhead: 3,
                request_overhead: 3,
                max_context_tokens: 4096,
                special_tokens: HashMap::new(),
            },
        );

        // Claude family
        configs.insert(
            "claude-3".to_string(),
            ModelTokenConfig {
                model: "claude-3".to_string(),
                chars_per_token: 3.5,
                message_overhead: 4,
                request_overhead: 5,
                max_context_tokens: 200000,
                special_tokens: HashMap::new(),
            },
        );

        configs.insert(
            "claude-2".to_string(),
            ModelTokenConfig {
                model: "claude-2".to_string(),
                chars_per_token: 3.5,
                message_overhead: 4,
                request_overhead: 5,
                max_context_tokens: 100000,
                special_tokens: HashMap::new(),
            },
        );

        // Default configuration
        configs.insert(
            "default".to_string(),
            ModelTokenConfig {
                model: "default".to_string(),
                chars_per_token: 4.0,
                message_overhead: 3,
                request_overhead: 3,
                max_context_tokens: 4096,
                special_tokens: HashMap::new(),
            },
        );

        configs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== ModelTokenConfig Tests ====================

    #[test]
    fn test_model_token_config_structure() {
        let config = ModelTokenConfig {
            model: "test-model".to_string(),
            chars_per_token: 4.0,
            message_overhead: 3,
            request_overhead: 5,
            max_context_tokens: 8192,
            special_tokens: HashMap::new(),
        };
        assert_eq!(config.model, "test-model");
        assert!((config.chars_per_token - 4.0).abs() < f64::EPSILON);
        assert_eq!(config.message_overhead, 3);
        assert_eq!(config.max_context_tokens, 8192);
    }

    #[test]
    fn test_model_token_config_with_special_tokens() {
        let mut special_tokens = HashMap::new();
        special_tokens.insert("<|endoftext|>".to_string(), 1);
        special_tokens.insert("<|im_start|>".to_string(), 1);

        let config = ModelTokenConfig {
            model: "gpt-4".to_string(),
            chars_per_token: 4.0,
            message_overhead: 3,
            request_overhead: 3,
            max_context_tokens: 8192,
            special_tokens,
        };
        assert_eq!(config.special_tokens.len(), 2);
        assert_eq!(config.special_tokens.get("<|endoftext|>"), Some(&1));
    }

    #[test]
    fn test_model_token_config_clone() {
        let config = ModelTokenConfig {
            model: "clone-test".to_string(),
            chars_per_token: 3.5,
            message_overhead: 4,
            request_overhead: 5,
            max_context_tokens: 100000,
            special_tokens: HashMap::new(),
        };
        let cloned = config.clone();
        assert_eq!(config.model, cloned.model);
        assert!((config.chars_per_token - cloned.chars_per_token).abs() < f64::EPSILON);
    }

    #[test]
    fn test_model_token_config_serialization() {
        let config = ModelTokenConfig {
            model: "ser-test".to_string(),
            chars_per_token: 4.0,
            message_overhead: 3,
            request_overhead: 3,
            max_context_tokens: 4096,
            special_tokens: HashMap::new(),
        };
        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["model"], "ser-test");
        assert_eq!(json["max_context_tokens"], 4096);
    }

    #[test]
    fn test_default_configs_contains_gpt4() {
        let configs = ModelTokenConfig::default_configs();
        assert!(configs.contains_key("gpt-4"));
        let gpt4 = configs.get("gpt-4").unwrap();
        assert_eq!(gpt4.max_context_tokens, 8192);
    }

    #[test]
    fn test_default_configs_contains_gpt35() {
        let configs = ModelTokenConfig::default_configs();
        assert!(configs.contains_key("gpt-3.5-turbo"));
        let gpt35 = configs.get("gpt-3.5-turbo").unwrap();
        assert_eq!(gpt35.max_context_tokens, 4096);
    }

    #[test]
    fn test_default_configs_contains_claude() {
        let configs = ModelTokenConfig::default_configs();
        assert!(configs.contains_key("claude-3"));
        let claude3 = configs.get("claude-3").unwrap();
        assert_eq!(claude3.max_context_tokens, 200000);
    }

    #[test]
    fn test_default_configs_contains_default() {
        let configs = ModelTokenConfig::default_configs();
        assert!(configs.contains_key("default"));
    }

    // ==================== TokenEstimate Tests ====================

    #[test]
    fn test_token_estimate_structure() {
        let estimate = TokenEstimate {
            input_tokens: 100,
            output_tokens: Some(50),
            total_tokens: 150,
            is_approximate: true,
            confidence: 0.85,
        };
        assert_eq!(estimate.input_tokens, 100);
        assert_eq!(estimate.output_tokens, Some(50));
        assert_eq!(estimate.total_tokens, 150);
        assert!(estimate.is_approximate);
    }

    #[test]
    fn test_token_estimate_no_output() {
        let estimate = TokenEstimate {
            input_tokens: 200,
            output_tokens: None,
            total_tokens: 200,
            is_approximate: false,
            confidence: 1.0,
        };
        assert!(estimate.output_tokens.is_none());
        assert!(!estimate.is_approximate);
    }

    #[test]
    fn test_token_estimate_clone() {
        let estimate = TokenEstimate {
            input_tokens: 50,
            output_tokens: Some(25),
            total_tokens: 75,
            is_approximate: true,
            confidence: 0.9,
        };
        let cloned = estimate.clone();
        assert_eq!(estimate.input_tokens, cloned.input_tokens);
        assert_eq!(estimate.confidence, cloned.confidence);
    }

    #[test]
    fn test_token_estimate_serialization() {
        let estimate = TokenEstimate {
            input_tokens: 100,
            output_tokens: Some(50),
            total_tokens: 150,
            is_approximate: true,
            confidence: 0.85,
        };
        let json = serde_json::to_value(&estimate).unwrap();
        assert_eq!(json["input_tokens"], 100);
        assert_eq!(json["total_tokens"], 150);
        assert_eq!(json["is_approximate"], true);
    }

    #[test]
    fn test_token_estimate_deserialization() {
        let json = r#"{
            "input_tokens": 200,
            "output_tokens": null,
            "total_tokens": 200,
            "is_approximate": false,
            "confidence": 1.0
        }"#;
        let estimate: TokenEstimate = serde_json::from_str(json).unwrap();
        assert_eq!(estimate.input_tokens, 200);
        assert!(estimate.output_tokens.is_none());
    }
}
