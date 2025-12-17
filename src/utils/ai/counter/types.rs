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
