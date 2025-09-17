//! Token counting utilities for the Gateway
//!
//! This module provides token counting functionality for different AI models.

use crate::core::models::openai::{ChatMessage, ContentPart, MessageContent};
use crate::utils::error::{GatewayError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Token counter for different models
#[derive(Debug, Clone)]
pub struct TokenCounter {
    /// Model-specific token counting configurations
    model_configs: HashMap<String, ModelTokenConfig>,
}

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

impl TokenCounter {
    /// Create a new token counter
    pub fn new() -> Self {
        Self {
            model_configs: Self::default_model_configs(),
        }
    }

    /// Count tokens in a chat completion request
    #[allow(dead_code)]
    pub fn count_chat_tokens(
        &self,
        model: &str,
        messages: &[ChatMessage],
    ) -> Result<TokenEstimate> {
        let config = self.get_model_config(model)?;
        let mut total_tokens = config.request_overhead;

        for message in messages {
            total_tokens += self.count_message_tokens(config, message)?;
        }

        Ok(TokenEstimate {
            input_tokens: total_tokens,
            output_tokens: None,
            total_tokens,
            is_approximate: true,
            confidence: 0.85, // Reasonable confidence for estimation
        })
    }

    /// Count tokens in a single message
    #[allow(dead_code)]
    fn count_message_tokens(
        &self,
        config: &ModelTokenConfig,
        message: &ChatMessage,
    ) -> Result<u32> {
        let mut tokens = config.message_overhead;

        // Count role tokens
        tokens += self.estimate_text_tokens(config, &ToString::to_string(&message.role));

        // Count content tokens
        if let Some(content) = &message.content {
            tokens += self.count_content_tokens(config, content)?;
        }

        // Count name tokens if present
        if let Some(name) = &message.name {
            tokens += self.estimate_text_tokens(config, name);
        }

        // Count function call tokens if present
        if let Some(function_call) = &message.function_call {
            tokens += self.estimate_text_tokens(config, &function_call.name);
            tokens += self.estimate_text_tokens(config, &function_call.arguments);
        }

        // Count tool calls tokens if present
        if let Some(tool_calls) = &message.tool_calls {
            for tool_call in tool_calls {
                tokens += self.estimate_text_tokens(config, &tool_call.id);
                tokens += self.estimate_text_tokens(config, &tool_call.tool_type);
                tokens += self.estimate_text_tokens(config, &tool_call.function.name);
                tokens += self.estimate_text_tokens(config, &tool_call.function.arguments);
            }
        }

        Ok(tokens)
    }

    /// Count tokens in message content
    #[allow(dead_code)]
    fn count_content_tokens(
        &self,
        config: &ModelTokenConfig,
        content: &MessageContent,
    ) -> Result<u32> {
        match content {
            MessageContent::Text(text) => Ok(self.estimate_text_tokens(config, text)),
            MessageContent::Parts(parts) => {
                let mut tokens = 0;
                for part in parts {
                    tokens += self.count_content_part_tokens(config, part)?;
                }
                Ok(tokens)
            }
        }
    }

    /// Count tokens in a content part
    #[allow(dead_code)]
    fn count_content_part_tokens(
        &self,
        config: &ModelTokenConfig,
        part: &ContentPart,
    ) -> Result<u32> {
        match part {
            ContentPart::Text { text } => Ok(self.estimate_text_tokens(config, text)),
            ContentPart::ImageUrl { image_url: _ } => {
                // Images typically use a fixed number of tokens
                // This is a simplified estimation
                Ok(85) // Base tokens for image processing
            }
            ContentPart::Audio { audio: _ } => {
                // Audio tokens depend on duration, but we don't have that info
                // Use a reasonable default
                Ok(100)
            }
        }
    }

    /// Estimate tokens for text content
    fn estimate_text_tokens(&self, config: &ModelTokenConfig, text: &str) -> u32 {
        if text.is_empty() {
            return 0;
        }

        // Simple character-based estimation
        let char_count = text.chars().count() as f64;
        let estimated_tokens = (char_count / config.chars_per_token).ceil() as u32;

        // Add some buffer for special tokens and encoding overhead
        (estimated_tokens as f64 * 1.1).ceil() as u32
    }

    /// Count tokens in completion request
    pub fn count_completion_tokens(&self, model: &str, prompt: &str) -> Result<TokenEstimate> {
        let config = self.get_model_config(model)?;
        let input_tokens = config.request_overhead + self.estimate_text_tokens(config, prompt);

        Ok(TokenEstimate {
            input_tokens,
            output_tokens: None,
            total_tokens: input_tokens,
            is_approximate: true,
            confidence: 0.8,
        })
    }

    /// Count tokens in embedding request
    #[allow(dead_code)]
    pub fn count_embedding_tokens(&self, model: &str, input: &[String]) -> Result<TokenEstimate> {
        let config = self.get_model_config(model)?;
        let mut total_tokens = config.request_overhead;

        for text in input {
            total_tokens += self.estimate_text_tokens(config, text);
        }

        Ok(TokenEstimate {
            input_tokens: total_tokens,
            output_tokens: None,
            total_tokens,
            is_approximate: true,
            confidence: 0.9, // Embeddings are more predictable
        })
    }

    /// Estimate output tokens based on max_tokens parameter
    #[allow(dead_code)]
    pub fn estimate_output_tokens(
        &self,
        max_tokens: Option<u32>,
        input_tokens: u32,
        model: &str,
    ) -> Result<u32> {
        let config = self.get_model_config(model)?;

        if let Some(max) = max_tokens {
            // Use the specified max_tokens, but cap at model's context window
            let available_tokens = config.max_context_tokens.saturating_sub(input_tokens);
            Ok(max.min(available_tokens))
        } else {
            // Use a reasonable default (e.g., 25% of remaining context)
            let available_tokens = config.max_context_tokens.saturating_sub(input_tokens);
            Ok((available_tokens as f64 * 0.25).ceil() as u32)
        }
    }

    /// Check if request fits within context window
    #[allow(dead_code)]
    pub fn check_context_window(
        &self,
        model: &str,
        input_tokens: u32,
        max_output_tokens: Option<u32>,
    ) -> Result<bool> {
        let config = self.get_model_config(model)?;
        let output_tokens = max_output_tokens.unwrap_or(0);
        let total_tokens = input_tokens + output_tokens;

        Ok(total_tokens <= config.max_context_tokens)
    }

    /// Get model configuration
    fn get_model_config(&self, model: &str) -> Result<&ModelTokenConfig> {
        // Try exact match first
        if let Some(config) = self.model_configs.get(model) {
            return Ok(config);
        }

        // Try to find a matching family
        let model_family = self.extract_model_family(model);
        if let Some(config) = self.model_configs.get(&model_family) {
            return Ok(config);
        }

        // Fall back to default
        self.model_configs.get("default").ok_or_else(|| {
            GatewayError::Config(format!("No token config found for model: {}", model))
        })
    }

    /// Extract model family from model name
    fn extract_model_family(&self, model: &str) -> String {
        // Remove provider prefix if present
        let model = if let Some(pos) = model.find('/') {
            &model[pos + 1..]
        } else {
            model
        };

        // Extract family name
        if model.starts_with("gpt-4") {
            "gpt-4".to_string()
        } else if model.starts_with("gpt-3.5") {
            "gpt-3.5-turbo".to_string()
        } else if model.starts_with("claude-3") {
            "claude-3".to_string()
        } else if model.starts_with("claude-2") {
            "claude-2".to_string()
        } else {
            "default".to_string()
        }
    }

    /// Default model configurations
    fn default_model_configs() -> HashMap<String, ModelTokenConfig> {
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

    /// Add or update model configuration
    #[allow(dead_code)]
    pub fn add_model_config(&mut self, config: ModelTokenConfig) {
        self.model_configs.insert(config.model.clone(), config);
    }

    /// Get supported models
    #[allow(dead_code)]
    pub fn get_supported_models(&self) -> Vec<String> {
        self.model_configs.keys().cloned().collect()
    }
}

impl Default for TokenCounter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::openai::{ChatMessage, MessageContent, MessageRole};

    #[test]
    fn test_text_token_estimation() {
        let counter = TokenCounter::new();
        let config = counter.get_model_config("gpt-3.5-turbo").unwrap();

        let tokens = counter.estimate_text_tokens(config, "Hello, world!");
        assert!(tokens > 0);
        assert!(tokens < 10); // Should be reasonable for short text
    }

    #[test]
    fn test_chat_token_counting() {
        let counter = TokenCounter::new();
        let messages = vec![ChatMessage {
            role: MessageRole::User,
            content: Some(MessageContent::Text("Hello, how are you?".to_string())),
            name: None,
            function_call: None,
            tool_calls: None,
            tool_call_id: None,
            audio: None,
        }];

        let estimate = counter
            .count_chat_tokens("gpt-3.5-turbo", &messages)
            .unwrap();
        assert!(estimate.input_tokens > 0);
        assert!(estimate.is_approximate);
    }

    #[test]
    fn test_context_window_check() {
        let counter = TokenCounter::new();

        // Should fit
        assert!(
            counter
                .check_context_window("gpt-3.5-turbo", 1000, Some(1000))
                .unwrap()
        );

        // Should not fit
        assert!(
            !counter
                .check_context_window("gpt-3.5-turbo", 3000, Some(2000))
                .unwrap()
        );
    }

    #[test]
    fn test_model_family_extraction() {
        let counter = TokenCounter::new();

        assert_eq!(counter.extract_model_family("gpt-4-turbo"), "gpt-4");
        assert_eq!(
            counter.extract_model_family("gpt-3.5-turbo-16k"),
            "gpt-3.5-turbo"
        );
        assert_eq!(counter.extract_model_family("claude-3-opus"), "claude-3");
        assert_eq!(counter.extract_model_family("unknown-model"), "default");
    }
}
