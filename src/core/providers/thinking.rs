//! Thinking/Reasoning Provider Trait
//!
//! This module defines the trait for providers that support thinking/reasoning capabilities.
//! It provides a unified interface for:
//! - OpenAI o1/o3/o4 reasoning
//! - Anthropic Claude extended thinking
//! - DeepSeek R1/Reasoner
//! - Gemini 2.0 Flash Thinking / 3.0 Deep Think
//! - OpenRouter passthrough

use serde_json::Value;

use crate::core::providers::unified_provider::ProviderError;
use crate::core::types::thinking::{
    ThinkingCapabilities, ThinkingConfig, ThinkingContent, ThinkingEffort, ThinkingUsage,
};

/// Trait for providers that support thinking/reasoning capabilities
///
/// This trait enables providers to:
/// 1. Advertise thinking support for specific models
/// 2. Transform thinking configuration to provider-specific format
/// 3. Extract thinking content from responses
/// 4. Track thinking token usage and costs
pub trait ThinkingProvider {
    /// Check if a specific model supports thinking
    ///
    /// # Arguments
    /// * `model` - The model identifier to check
    ///
    /// # Returns
    /// `true` if the model supports thinking, `false` otherwise
    fn supports_thinking(&self, model: &str) -> bool;

    /// Get thinking capabilities for a specific model
    ///
    /// Returns detailed information about what thinking features are supported.
    fn thinking_capabilities(&self, model: &str) -> ThinkingCapabilities;

    /// Transform thinking configuration to provider-specific format
    ///
    /// # Arguments
    /// * `config` - The unified thinking configuration
    /// * `model` - The model being used
    ///
    /// # Returns
    /// A JSON value with provider-specific thinking parameters
    fn transform_thinking_config(
        &self,
        config: &ThinkingConfig,
        model: &str,
    ) -> Result<Value, ProviderError>;

    /// Extract thinking content from a provider response
    ///
    /// # Arguments
    /// * `response` - The raw JSON response from the provider
    ///
    /// # Returns
    /// The extracted thinking content, if present
    fn extract_thinking(&self, response: &Value) -> Option<ThinkingContent>;

    /// Extract thinking usage statistics from a provider response
    ///
    /// # Arguments
    /// * `response` - The raw JSON response from the provider
    ///
    /// # Returns
    /// Thinking usage statistics, if available
    fn extract_thinking_usage(&self, response: &Value) -> Option<ThinkingUsage>;

    /// Get the default thinking effort for this provider
    ///
    /// Returns the default effort level when none is specified.
    fn default_thinking_effort(&self) -> ThinkingEffort {
        ThinkingEffort::Medium
    }

    /// Get maximum thinking tokens allowed for a model
    ///
    /// Returns `None` if there's no limit or it's unknown.
    fn max_thinking_tokens(&self, model: &str) -> Option<u32> {
        self.thinking_capabilities(model).max_thinking_tokens
    }

    /// Check if the provider supports streaming thinking content
    fn supports_streaming_thinking(&self, model: &str) -> bool {
        self.thinking_capabilities(model)
            .supports_streaming_thinking
    }
}

/// Default implementation helper for providers without thinking support
pub struct NoThinkingSupport;

impl ThinkingProvider for NoThinkingSupport {
    fn supports_thinking(&self, _model: &str) -> bool {
        false
    }

    fn thinking_capabilities(&self, _model: &str) -> ThinkingCapabilities {
        ThinkingCapabilities::unsupported()
    }

    fn transform_thinking_config(
        &self,
        _config: &ThinkingConfig,
        _model: &str,
    ) -> Result<Value, ProviderError> {
        Ok(Value::Object(serde_json::Map::new()))
    }

    fn extract_thinking(&self, _response: &Value) -> Option<ThinkingContent> {
        None
    }

    fn extract_thinking_usage(&self, _response: &Value) -> Option<ThinkingUsage> {
        None
    }
}

/// OpenAI-specific thinking implementation
pub mod openai_thinking {
    use super::*;

    /// OpenAI thinking models (o1, o3, o4 series)
    const OPENAI_THINKING_MODELS: &[&str] = &[
        "o1",
        "o1-preview",
        "o1-mini",
        "o3",
        "o3-mini",
        "o4",
        "o4-mini",
    ];

    /// Check if an OpenAI model supports thinking
    pub fn supports_thinking(model: &str) -> bool {
        let model_lower = model.to_lowercase();
        OPENAI_THINKING_MODELS
            .iter()
            .any(|m| model_lower.starts_with(m) || model_lower.contains(&format!("/{}", m)))
    }

    /// Get thinking capabilities for OpenAI models
    pub fn capabilities(model: &str) -> ThinkingCapabilities {
        if supports_thinking(model) {
            ThinkingCapabilities {
                supports_thinking: true,
                supports_streaming_thinking: false, // OpenAI doesn't stream reasoning
                max_thinking_tokens: Some(20_000),
                supported_efforts: vec![
                    ThinkingEffort::Low,
                    ThinkingEffort::Medium,
                    ThinkingEffort::High,
                ],
                thinking_models: OPENAI_THINKING_MODELS
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                can_return_thinking: true,
                thinking_always_on: false,
            }
        } else {
            ThinkingCapabilities::unsupported()
        }
    }

    /// Transform thinking config for OpenAI
    pub fn transform_config(config: &ThinkingConfig, _model: &str) -> Result<Value, ProviderError> {
        let mut params = serde_json::Map::new();

        if let Some(budget) = config.budget_tokens {
            // OpenAI max is 20,000
            let capped = budget.min(20_000);
            params.insert("max_reasoning_tokens".into(), capped.into());
        }

        if config.include_thinking {
            params.insert("include_reasoning".into(), true.into());
        }

        // Map effort to reasoning_effort
        if let Some(effort) = &config.effort {
            let effort_str = match effort {
                ThinkingEffort::Low => "low",
                ThinkingEffort::Medium => "medium",
                ThinkingEffort::High => "high",
            };
            params.insert("reasoning_effort".into(), effort_str.into());
        }

        Ok(Value::Object(params))
    }

    /// Extract thinking from OpenAI response
    pub fn extract_thinking(response: &Value) -> Option<ThinkingContent> {
        response
            .pointer("/choices/0/message/reasoning")
            .and_then(|v| v.as_str())
            .map(|text| ThinkingContent::Text {
                text: text.to_string(),
                signature: None,
            })
    }

    /// Extract thinking usage from OpenAI response
    pub fn extract_usage(response: &Value) -> Option<ThinkingUsage> {
        response
            .pointer("/usage/reasoning_tokens")
            .map(|tokens| ThinkingUsage {
                thinking_tokens: tokens.as_u64().map(|t| t as u32),
                budget_tokens: None,
                thinking_cost: None,
                provider: Some("openai".to_string()),
            })
    }
}

/// Anthropic-specific thinking implementation
pub mod anthropic_thinking {
    use super::*;

    /// Anthropic models with thinking support
    const ANTHROPIC_THINKING_MODELS: &[&str] = &[
        "claude-3-opus",
        "claude-3-sonnet",
        "claude-3-haiku",
        "claude-3-5-sonnet",
        "claude-3-5-opus",
        "claude-4",
    ];

    /// Check if an Anthropic model supports thinking
    pub fn supports_thinking(model: &str) -> bool {
        let model_lower = model.to_lowercase();
        ANTHROPIC_THINKING_MODELS
            .iter()
            .any(|m| model_lower.contains(m))
    }

    /// Get thinking capabilities for Anthropic models
    pub fn capabilities(model: &str) -> ThinkingCapabilities {
        if supports_thinking(model) {
            ThinkingCapabilities {
                supports_thinking: true,
                supports_streaming_thinking: true, // Anthropic supports streaming thinking
                max_thinking_tokens: Some(100_000), // Anthropic allows larger budgets
                supported_efforts: vec![ThinkingEffort::Medium, ThinkingEffort::High],
                thinking_models: ANTHROPIC_THINKING_MODELS
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                can_return_thinking: true,
                thinking_always_on: false,
            }
        } else {
            ThinkingCapabilities::unsupported()
        }
    }

    /// Transform thinking config for Anthropic
    pub fn transform_config(config: &ThinkingConfig, _model: &str) -> Result<Value, ProviderError> {
        let mut params = serde_json::Map::new();

        if config.enabled {
            let mut thinking = serde_json::Map::new();
            thinking.insert("type".into(), "enabled".into());

            if let Some(budget) = config.budget_tokens {
                thinking.insert("budget_tokens".into(), budget.into());
            }

            params.insert("thinking".into(), Value::Object(thinking));
        }

        Ok(Value::Object(params))
    }

    /// Extract thinking from Anthropic response
    pub fn extract_thinking(response: &Value) -> Option<ThinkingContent> {
        response
            .pointer("/content")
            .and_then(|v| v.as_array())
            .and_then(|blocks| {
                blocks.iter().find_map(|block| {
                    if block.get("type")?.as_str()? == "thinking" {
                        Some(ThinkingContent::Block {
                            thinking: block.get("thinking")?.as_str()?.to_string(),
                            block_type: Some("thinking".to_string()),
                        })
                    } else {
                        None
                    }
                })
            })
    }

    /// Extract thinking usage from Anthropic response
    pub fn extract_usage(response: &Value) -> Option<ThinkingUsage> {
        let thinking_tokens = response
            .pointer("/usage/thinking_tokens")
            .and_then(|v| v.as_u64())
            .map(|t| t as u32);

        if thinking_tokens.is_some() {
            Some(ThinkingUsage {
                thinking_tokens,
                budget_tokens: response
                    .pointer("/usage/thinking_budget_tokens")
                    .and_then(|v| v.as_u64())
                    .map(|t| t as u32),
                thinking_cost: None,
                provider: Some("anthropic".to_string()),
            })
        } else {
            None
        }
    }
}

/// DeepSeek-specific thinking implementation
pub mod deepseek_thinking {
    use super::*;

    /// DeepSeek thinking models
    const DEEPSEEK_THINKING_MODELS: &[&str] = &["deepseek-r1", "deepseek-reasoner", "r1"];

    /// Check if a DeepSeek model supports thinking
    pub fn supports_thinking(model: &str) -> bool {
        let model_lower = model.to_lowercase();
        DEEPSEEK_THINKING_MODELS
            .iter()
            .any(|m| model_lower.contains(m))
    }

    /// Get thinking capabilities for DeepSeek models
    pub fn capabilities(model: &str) -> ThinkingCapabilities {
        if supports_thinking(model) {
            ThinkingCapabilities {
                supports_thinking: true,
                supports_streaming_thinking: true,
                max_thinking_tokens: None, // No documented limit
                supported_efforts: vec![
                    ThinkingEffort::Low,
                    ThinkingEffort::Medium,
                    ThinkingEffort::High,
                ],
                thinking_models: DEEPSEEK_THINKING_MODELS
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                can_return_thinking: true,
                thinking_always_on: true, // DeepSeek R1 always thinks
            }
        } else {
            ThinkingCapabilities::unsupported()
        }
    }

    /// Transform thinking config for DeepSeek
    pub fn transform_config(config: &ThinkingConfig, _model: &str) -> Result<Value, ProviderError> {
        let mut params = serde_json::Map::new();

        // DeepSeek uses reasoning_effort
        if let Some(effort) = &config.effort {
            let effort_str = match effort {
                ThinkingEffort::Low => "low",
                ThinkingEffort::Medium => "medium",
                ThinkingEffort::High => "high",
            };
            params.insert("reasoning_effort".into(), effort_str.into());
        }

        Ok(Value::Object(params))
    }

    /// Extract thinking from DeepSeek response
    pub fn extract_thinking(response: &Value) -> Option<ThinkingContent> {
        response
            .pointer("/choices/0/message/reasoning_content")
            .and_then(|v| v.as_str())
            .map(|text| ThinkingContent::Text {
                text: text.to_string(),
                signature: None,
            })
    }

    /// Extract thinking usage from DeepSeek response
    pub fn extract_usage(response: &Value) -> Option<ThinkingUsage> {
        response
            .pointer("/usage/reasoning_tokens")
            .map(|tokens| ThinkingUsage {
                thinking_tokens: tokens.as_u64().map(|t| t as u32),
                budget_tokens: None,
                thinking_cost: None,
                provider: Some("deepseek".to_string()),
            })
    }
}

/// Gemini-specific thinking implementation
pub mod gemini_thinking {
    use super::*;

    /// Gemini thinking models
    const GEMINI_THINKING_MODELS: &[&str] = &[
        "gemini-2.0-flash-thinking",
        "gemini-3.0-deep-think",
        "gemini-thinking",
    ];

    /// Check if a Gemini model supports thinking
    pub fn supports_thinking(model: &str) -> bool {
        let model_lower = model.to_lowercase();
        GEMINI_THINKING_MODELS
            .iter()
            .any(|m| model_lower.contains(m))
            || model_lower.contains("thinking")
            || model_lower.contains("deep-think")
    }

    /// Get thinking capabilities for Gemini models
    pub fn capabilities(model: &str) -> ThinkingCapabilities {
        if supports_thinking(model) {
            ThinkingCapabilities {
                supports_thinking: true,
                supports_streaming_thinking: true,
                max_thinking_tokens: Some(32_000),
                supported_efforts: vec![ThinkingEffort::Medium, ThinkingEffort::High],
                thinking_models: GEMINI_THINKING_MODELS
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                can_return_thinking: true,
                thinking_always_on: false,
            }
        } else {
            ThinkingCapabilities::unsupported()
        }
    }

    /// Transform thinking config for Gemini
    pub fn transform_config(config: &ThinkingConfig, _model: &str) -> Result<Value, ProviderError> {
        let mut params = serde_json::Map::new();

        if config.enabled {
            params.insert("enableThinking".into(), true.into());

            if let Some(budget) = config.budget_tokens {
                params.insert("thinkingBudget".into(), budget.into());
            }
        }

        Ok(Value::Object(params))
    }

    /// Extract thinking from Gemini response
    pub fn extract_thinking(response: &Value) -> Option<ThinkingContent> {
        // Try thoughts field first
        response
            .pointer("/candidates/0/content/thoughts")
            .and_then(|v| v.as_str())
            .map(|text| ThinkingContent::Text {
                text: text.to_string(),
                signature: None,
            })
            // Also try thinking field
            .or_else(|| {
                response
                    .pointer("/candidates/0/content/thinking")
                    .and_then(|v| v.as_str())
                    .map(|text| ThinkingContent::Text {
                        text: text.to_string(),
                        signature: None,
                    })
            })
    }

    /// Extract thinking usage from Gemini response
    pub fn extract_usage(response: &Value) -> Option<ThinkingUsage> {
        response
            .pointer("/usageMetadata/thinkingTokenCount")
            .map(|tokens| ThinkingUsage {
                thinking_tokens: tokens.as_u64().map(|t| t as u32),
                budget_tokens: None,
                thinking_cost: None,
                provider: Some("gemini".to_string()),
            })
    }
}

/// OpenRouter passthrough implementation
///
/// OpenRouter routes to multiple providers, so we detect the underlying provider
/// and use the appropriate thinking extraction.
pub mod openrouter_thinking {
    use super::*;

    /// Check if a model supports thinking through OpenRouter
    pub fn supports_thinking(model: &str) -> bool {
        let model_lower = model.to_lowercase();

        // Check for OpenAI reasoning models
        if model_lower.contains("o1") || model_lower.contains("o3") || model_lower.contains("o4") {
            return true;
        }

        // Check for Anthropic models
        if model_lower.contains("claude") {
            return true;
        }

        // Check for DeepSeek reasoning models
        if model_lower.contains("deepseek-r1") || model_lower.contains("reasoner") {
            return true;
        }

        // Check for Gemini thinking models
        if model_lower.contains("gemini") && model_lower.contains("thinking") {
            return true;
        }

        false
    }

    /// Detect the underlying provider from model name
    pub fn detect_provider(model: &str) -> &'static str {
        let model_lower = model.to_lowercase();

        if model_lower.contains("openai")
            || model_lower.starts_with("o1")
            || model_lower.starts_with("o3")
            || model_lower.starts_with("o4")
        {
            "openai"
        } else if model_lower.contains("anthropic") || model_lower.contains("claude") {
            "anthropic"
        } else if model_lower.contains("deepseek") {
            "deepseek"
        } else if model_lower.contains("gemini") || model_lower.contains("google") {
            "gemini"
        } else {
            "unknown"
        }
    }

    /// Get thinking capabilities for OpenRouter models
    pub fn capabilities(model: &str) -> ThinkingCapabilities {
        match detect_provider(model) {
            "openai" => openai_thinking::capabilities(model),
            "anthropic" => anthropic_thinking::capabilities(model),
            "deepseek" => deepseek_thinking::capabilities(model),
            "gemini" => gemini_thinking::capabilities(model),
            _ => ThinkingCapabilities::unsupported(),
        }
    }

    /// Transform thinking config for OpenRouter
    pub fn transform_config(config: &ThinkingConfig, model: &str) -> Result<Value, ProviderError> {
        match detect_provider(model) {
            "openai" => openai_thinking::transform_config(config, model),
            "anthropic" => anthropic_thinking::transform_config(config, model),
            "deepseek" => deepseek_thinking::transform_config(config, model),
            "gemini" => gemini_thinking::transform_config(config, model),
            _ => Ok(Value::Object(serde_json::Map::new())),
        }
    }

    /// Extract thinking from OpenRouter response
    ///
    /// Tries multiple extraction patterns since the response format
    /// depends on the underlying provider.
    pub fn extract_thinking(response: &Value) -> Option<ThinkingContent> {
        // Try OpenAI style
        if let Some(thinking) = openai_thinking::extract_thinking(response) {
            return Some(thinking);
        }

        // Try DeepSeek style
        if let Some(thinking) = deepseek_thinking::extract_thinking(response) {
            return Some(thinking);
        }

        // Try Anthropic style
        if let Some(thinking) = anthropic_thinking::extract_thinking(response) {
            return Some(thinking);
        }

        // Try Gemini style
        if let Some(thinking) = gemini_thinking::extract_thinking(response) {
            return Some(thinking);
        }

        None
    }

    /// Extract thinking usage from OpenRouter response
    pub fn extract_usage(response: &Value) -> Option<ThinkingUsage> {
        // Try OpenAI style
        if let Some(mut usage) = openai_thinking::extract_usage(response) {
            usage.provider = Some("openrouter".to_string());
            return Some(usage);
        }

        // Try DeepSeek style
        if let Some(mut usage) = deepseek_thinking::extract_usage(response) {
            usage.provider = Some("openrouter".to_string());
            return Some(usage);
        }

        // Try Anthropic style
        if let Some(mut usage) = anthropic_thinking::extract_usage(response) {
            usage.provider = Some("openrouter".to_string());
            return Some(usage);
        }

        // Try Gemini style
        if let Some(mut usage) = gemini_thinking::extract_usage(response) {
            usage.provider = Some("openrouter".to_string());
            return Some(usage);
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // NoThinkingSupport Tests
    // ============================================================================

    #[test]
    fn test_no_thinking_support() {
        let no_support = NoThinkingSupport;
        assert!(!no_support.supports_thinking("any-model"));
        assert!(
            no_support
                .extract_thinking(&serde_json::json!({}))
                .is_none()
        );
    }

    #[test]
    fn test_no_thinking_support_capabilities() {
        let no_support = NoThinkingSupport;
        let caps = no_support.thinking_capabilities("any-model");
        assert!(!caps.supports_thinking);
        assert!(!caps.supports_streaming_thinking);
        assert!(!caps.can_return_thinking);
    }

    #[test]
    fn test_no_thinking_support_transform_config() {
        let no_support = NoThinkingSupport;
        let config = ThinkingConfig {
            enabled: true,
            budget_tokens: Some(1000),
            effort: Some(ThinkingEffort::High),
            include_thinking: true,
            extra_params: Default::default(),
        };
        let result = no_support
            .transform_thinking_config(&config, "model")
            .unwrap();
        assert!(result.as_object().unwrap().is_empty());
    }

    #[test]
    fn test_no_thinking_support_extract_usage() {
        let no_support = NoThinkingSupport;
        let response = serde_json::json!({
            "usage": {
                "thinking_tokens": 100
            }
        });
        assert!(no_support.extract_thinking_usage(&response).is_none());
    }

    // ============================================================================
    // OpenAI Thinking Tests
    // ============================================================================

    #[test]
    fn test_openai_thinking_detection() {
        assert!(openai_thinking::supports_thinking("o1"));
        assert!(openai_thinking::supports_thinking("o1-preview"));
        assert!(openai_thinking::supports_thinking("o3-mini"));
        assert!(openai_thinking::supports_thinking("O1-PREVIEW")); // Case insensitive
        assert!(openai_thinking::supports_thinking("o4"));
        assert!(openai_thinking::supports_thinking("openai/o1-preview")); // With prefix
        assert!(!openai_thinking::supports_thinking("gpt-4"));
        assert!(!openai_thinking::supports_thinking("gpt-4o"));
    }

    #[test]
    fn test_openai_capabilities() {
        let caps = openai_thinking::capabilities("o1-preview");
        assert!(caps.supports_thinking);
        assert!(!caps.supports_streaming_thinking);
        assert_eq!(caps.max_thinking_tokens, Some(20_000));
        assert_eq!(caps.supported_efforts.len(), 3);
        assert!(caps.can_return_thinking);
        assert!(!caps.thinking_always_on);
    }

    #[test]
    fn test_openai_capabilities_non_thinking_model() {
        let caps = openai_thinking::capabilities("gpt-4");
        assert!(!caps.supports_thinking);
    }

    #[test]
    fn test_openai_config_transform() {
        let config = ThinkingConfig {
            enabled: true,
            budget_tokens: Some(10000),
            effort: Some(ThinkingEffort::High),
            include_thinking: true,
            extra_params: Default::default(),
        };

        let result = openai_thinking::transform_config(&config, "o1").unwrap();
        assert!(result.get("max_reasoning_tokens").is_some());
        assert_eq!(
            result.get("max_reasoning_tokens").unwrap().as_u64(),
            Some(10000)
        );
        assert!(result.get("include_reasoning").is_some());
        assert_eq!(
            result.get("reasoning_effort").unwrap().as_str(),
            Some("high")
        );
    }

    #[test]
    fn test_openai_config_transform_budget_capping() {
        let config = ThinkingConfig {
            enabled: true,
            budget_tokens: Some(50000), // Over the 20k limit
            effort: Some(ThinkingEffort::Medium),
            include_thinking: false,
            extra_params: Default::default(),
        };

        let result = openai_thinking::transform_config(&config, "o1").unwrap();
        assert_eq!(
            result.get("max_reasoning_tokens").unwrap().as_u64(),
            Some(20000)
        );
        assert_eq!(
            result.get("reasoning_effort").unwrap().as_str(),
            Some("medium")
        );
        assert!(result.get("include_reasoning").is_none());
    }

    #[test]
    fn test_openai_config_transform_minimal() {
        let config = ThinkingConfig {
            enabled: true,
            budget_tokens: None,
            effort: None,
            include_thinking: false,
            extra_params: Default::default(),
        };

        let result = openai_thinking::transform_config(&config, "o1").unwrap();
        assert!(result.get("max_reasoning_tokens").is_none());
        assert!(result.get("reasoning_effort").is_none());
        assert!(result.get("include_reasoning").is_none());
    }

    #[test]
    fn test_openai_config_transform_low_effort() {
        let config = ThinkingConfig {
            enabled: true,
            budget_tokens: None,
            effort: Some(ThinkingEffort::Low),
            include_thinking: false,
            extra_params: Default::default(),
        };

        let result = openai_thinking::transform_config(&config, "o1").unwrap();
        assert_eq!(
            result.get("reasoning_effort").unwrap().as_str(),
            Some("low")
        );
    }

    #[test]
    fn test_openai_thinking_extraction() {
        let openai_response = serde_json::json!({
            "choices": [{
                "message": {
                    "content": "The answer is 42.",
                    "reasoning": "Let me think about this step by step..."
                }
            }],
            "usage": {
                "reasoning_tokens": 150
            }
        });

        let thinking = openai_thinking::extract_thinking(&openai_response);
        assert!(thinking.is_some());
        if let Some(ThinkingContent::Text { text, .. }) = thinking {
            assert!(text.contains("step by step"));
        }
    }

    #[test]
    fn test_openai_thinking_extraction_missing() {
        let response = serde_json::json!({
            "choices": [{
                "message": {
                    "content": "The answer is 42."
                }
            }]
        });

        let thinking = openai_thinking::extract_thinking(&response);
        assert!(thinking.is_none());
    }

    #[test]
    fn test_openai_usage_extraction() {
        let openai_response = serde_json::json!({
            "usage": {
                "reasoning_tokens": 150
            }
        });

        let usage = openai_thinking::extract_usage(&openai_response);
        assert!(usage.is_some());
        let usage = usage.unwrap();
        assert_eq!(usage.thinking_tokens, Some(150));
        assert_eq!(usage.provider, Some("openai".to_string()));
    }

    #[test]
    fn test_openai_usage_extraction_missing() {
        let response = serde_json::json!({
            "usage": {
                "total_tokens": 100
            }
        });

        let usage = openai_thinking::extract_usage(&response);
        assert!(usage.is_none());
    }

    // ============================================================================
    // Anthropic Thinking Tests
    // ============================================================================

    #[test]
    fn test_anthropic_thinking_detection() {
        assert!(anthropic_thinking::supports_thinking("claude-3-opus"));
        assert!(anthropic_thinking::supports_thinking(
            "claude-3-5-sonnet-20241022"
        ));
        assert!(anthropic_thinking::supports_thinking("Claude-3-Opus")); // Case insensitive
        assert!(anthropic_thinking::supports_thinking("claude-4"));
        assert!(!anthropic_thinking::supports_thinking("claude-2"));
    }

    #[test]
    fn test_anthropic_capabilities() {
        let caps = anthropic_thinking::capabilities("claude-3-opus");
        assert!(caps.supports_thinking);
        assert!(caps.supports_streaming_thinking);
        assert_eq!(caps.max_thinking_tokens, Some(100_000));
        assert_eq!(caps.supported_efforts.len(), 2);
        assert!(caps.can_return_thinking);
        assert!(!caps.thinking_always_on);
    }

    #[test]
    fn test_anthropic_capabilities_non_thinking_model() {
        let caps = anthropic_thinking::capabilities("claude-2");
        assert!(!caps.supports_thinking);
    }

    #[test]
    fn test_anthropic_config_transform_enabled() {
        let config = ThinkingConfig {
            enabled: true,
            budget_tokens: Some(50000),
            effort: Some(ThinkingEffort::High),
            include_thinking: true,
            extra_params: Default::default(),
        };

        let result = anthropic_thinking::transform_config(&config, "claude-3-opus").unwrap();
        let thinking_obj = result.get("thinking").unwrap();
        assert_eq!(thinking_obj.get("type").unwrap().as_str(), Some("enabled"));
        assert_eq!(
            thinking_obj.get("budget_tokens").unwrap().as_u64(),
            Some(50000)
        );
    }

    #[test]
    fn test_anthropic_config_transform_disabled() {
        let config = ThinkingConfig {
            enabled: false,
            budget_tokens: Some(50000),
            effort: Some(ThinkingEffort::High),
            include_thinking: true,
            extra_params: Default::default(),
        };

        let result = anthropic_thinking::transform_config(&config, "claude-3-opus").unwrap();
        assert!(result.get("thinking").is_none());
    }

    #[test]
    fn test_anthropic_config_transform_no_budget() {
        let config = ThinkingConfig {
            enabled: true,
            budget_tokens: None,
            effort: Some(ThinkingEffort::High),
            include_thinking: true,
            extra_params: Default::default(),
        };

        let result = anthropic_thinking::transform_config(&config, "claude-3-opus").unwrap();
        let thinking_obj = result.get("thinking").unwrap();
        assert_eq!(thinking_obj.get("type").unwrap().as_str(), Some("enabled"));
        assert!(thinking_obj.get("budget_tokens").is_none());
    }

    #[test]
    fn test_anthropic_thinking_extraction() {
        let response = serde_json::json!({
            "content": [
                {
                    "type": "thinking",
                    "thinking": "Let me analyze this carefully..."
                },
                {
                    "type": "text",
                    "text": "The answer is 42."
                }
            ]
        });

        let thinking = anthropic_thinking::extract_thinking(&response);
        assert!(thinking.is_some());
        if let Some(ThinkingContent::Block { thinking, .. }) = thinking {
            assert!(thinking.contains("analyze"));
        }
    }

    #[test]
    fn test_anthropic_thinking_extraction_no_thinking_block() {
        let response = serde_json::json!({
            "content": [
                {
                    "type": "text",
                    "text": "The answer is 42."
                }
            ]
        });

        let thinking = anthropic_thinking::extract_thinking(&response);
        assert!(thinking.is_none());
    }

    #[test]
    fn test_anthropic_usage_extraction() {
        let response = serde_json::json!({
            "usage": {
                "thinking_tokens": 500,
                "thinking_budget_tokens": 100000
            }
        });

        let usage = anthropic_thinking::extract_usage(&response);
        assert!(usage.is_some());
        let usage = usage.unwrap();
        assert_eq!(usage.thinking_tokens, Some(500));
        assert_eq!(usage.budget_tokens, Some(100000));
        assert_eq!(usage.provider, Some("anthropic".to_string()));
    }

    #[test]
    fn test_anthropic_usage_extraction_partial() {
        let response = serde_json::json!({
            "usage": {
                "thinking_tokens": 500
            }
        });

        let usage = anthropic_thinking::extract_usage(&response);
        assert!(usage.is_some());
        let usage = usage.unwrap();
        assert_eq!(usage.thinking_tokens, Some(500));
        assert!(usage.budget_tokens.is_none());
    }

    #[test]
    fn test_anthropic_usage_extraction_missing() {
        let response = serde_json::json!({
            "usage": {
                "total_tokens": 1000
            }
        });

        let usage = anthropic_thinking::extract_usage(&response);
        assert!(usage.is_none());
    }

    // ============================================================================
    // DeepSeek Thinking Tests
    // ============================================================================

    #[test]
    fn test_deepseek_thinking_detection() {
        assert!(deepseek_thinking::supports_thinking("deepseek-r1"));
        assert!(deepseek_thinking::supports_thinking("deepseek-reasoner"));
        assert!(deepseek_thinking::supports_thinking("r1"));
        assert!(deepseek_thinking::supports_thinking("DeepSeek-R1")); // Case insensitive
        assert!(!deepseek_thinking::supports_thinking("deepseek-chat"));
    }

    #[test]
    fn test_deepseek_capabilities() {
        let caps = deepseek_thinking::capabilities("deepseek-r1");
        assert!(caps.supports_thinking);
        assert!(caps.supports_streaming_thinking);
        assert!(caps.max_thinking_tokens.is_none());
        assert_eq!(caps.supported_efforts.len(), 3);
        assert!(caps.can_return_thinking);
        assert!(caps.thinking_always_on);
    }

    #[test]
    fn test_deepseek_capabilities_non_thinking_model() {
        let caps = deepseek_thinking::capabilities("deepseek-chat");
        assert!(!caps.supports_thinking);
    }

    #[test]
    fn test_deepseek_config_transform_all_efforts() {
        for (effort, expected) in [
            (ThinkingEffort::Low, "low"),
            (ThinkingEffort::Medium, "medium"),
            (ThinkingEffort::High, "high"),
        ] {
            let config = ThinkingConfig {
                enabled: true,
                budget_tokens: None,
                effort: Some(effort),
                include_thinking: true,
                extra_params: Default::default(),
            };

            let result = deepseek_thinking::transform_config(&config, "deepseek-r1").unwrap();
            assert_eq!(
                result.get("reasoning_effort").unwrap().as_str(),
                Some(expected)
            );
        }
    }

    #[test]
    fn test_deepseek_config_transform_no_effort() {
        let config = ThinkingConfig {
            enabled: true,
            budget_tokens: None,
            effort: None,
            include_thinking: true,
            extra_params: Default::default(),
        };

        let result = deepseek_thinking::transform_config(&config, "deepseek-r1").unwrap();
        assert!(result.get("reasoning_effort").is_none());
    }

    #[test]
    fn test_deepseek_thinking_extraction() {
        let response = serde_json::json!({
            "choices": [{
                "message": {
                    "content": "The answer is 42.",
                    "reasoning_content": "Step 1: Analyze the question..."
                }
            }]
        });

        let thinking = deepseek_thinking::extract_thinking(&response);
        assert!(thinking.is_some());
        if let Some(ThinkingContent::Text { text, .. }) = thinking {
            assert!(text.contains("Step 1"));
        }
    }

    #[test]
    fn test_deepseek_thinking_extraction_missing() {
        let response = serde_json::json!({
            "choices": [{
                "message": {
                    "content": "The answer is 42."
                }
            }]
        });

        let thinking = deepseek_thinking::extract_thinking(&response);
        assert!(thinking.is_none());
    }

    #[test]
    fn test_deepseek_usage_extraction() {
        let response = serde_json::json!({
            "usage": {
                "reasoning_tokens": 800
            }
        });

        let usage = deepseek_thinking::extract_usage(&response);
        assert!(usage.is_some());
        let usage = usage.unwrap();
        assert_eq!(usage.thinking_tokens, Some(800));
        assert_eq!(usage.provider, Some("deepseek".to_string()));
    }

    #[test]
    fn test_deepseek_usage_extraction_missing() {
        let response = serde_json::json!({
            "usage": {
                "total_tokens": 1000
            }
        });

        let usage = deepseek_thinking::extract_usage(&response);
        assert!(usage.is_none());
    }

    // ============================================================================
    // Gemini Thinking Tests
    // ============================================================================

    #[test]
    fn test_gemini_thinking_detection() {
        assert!(gemini_thinking::supports_thinking(
            "gemini-2.0-flash-thinking-exp"
        ));
        assert!(gemini_thinking::supports_thinking("gemini-thinking"));
        assert!(gemini_thinking::supports_thinking("gemini-3.0-deep-think"));
        assert!(gemini_thinking::supports_thinking("Gemini-Thinking")); // Case insensitive
        assert!(gemini_thinking::supports_thinking("gemini-deep-think"));
        assert!(!gemini_thinking::supports_thinking("gemini-pro"));
    }

    #[test]
    fn test_gemini_capabilities() {
        let caps = gemini_thinking::capabilities("gemini-2.0-flash-thinking");
        assert!(caps.supports_thinking);
        assert!(caps.supports_streaming_thinking);
        assert_eq!(caps.max_thinking_tokens, Some(32_000));
        assert_eq!(caps.supported_efforts.len(), 2);
        assert!(caps.can_return_thinking);
        assert!(!caps.thinking_always_on);
    }

    #[test]
    fn test_gemini_capabilities_non_thinking_model() {
        let caps = gemini_thinking::capabilities("gemini-pro");
        assert!(!caps.supports_thinking);
    }

    #[test]
    fn test_gemini_config_transform_enabled() {
        let config = ThinkingConfig {
            enabled: true,
            budget_tokens: Some(10000),
            effort: Some(ThinkingEffort::High),
            include_thinking: true,
            extra_params: Default::default(),
        };

        let result = gemini_thinking::transform_config(&config, "gemini-thinking").unwrap();
        assert_eq!(result.get("enableThinking").unwrap().as_bool(), Some(true));
        assert_eq!(result.get("thinkingBudget").unwrap().as_u64(), Some(10000));
    }

    #[test]
    fn test_gemini_config_transform_disabled() {
        let config = ThinkingConfig {
            enabled: false,
            budget_tokens: Some(10000),
            effort: Some(ThinkingEffort::High),
            include_thinking: true,
            extra_params: Default::default(),
        };

        let result = gemini_thinking::transform_config(&config, "gemini-thinking").unwrap();
        assert!(result.get("enableThinking").is_none());
    }

    #[test]
    fn test_gemini_config_transform_no_budget() {
        let config = ThinkingConfig {
            enabled: true,
            budget_tokens: None,
            effort: Some(ThinkingEffort::High),
            include_thinking: true,
            extra_params: Default::default(),
        };

        let result = gemini_thinking::transform_config(&config, "gemini-thinking").unwrap();
        assert_eq!(result.get("enableThinking").unwrap().as_bool(), Some(true));
        assert!(result.get("thinkingBudget").is_none());
    }

    #[test]
    fn test_gemini_thinking_extraction_thoughts() {
        let response = serde_json::json!({
            "candidates": [{
                "content": {
                    "thoughts": "Let me think through this problem..."
                }
            }]
        });

        let thinking = gemini_thinking::extract_thinking(&response);
        assert!(thinking.is_some());
        if let Some(ThinkingContent::Text { text, .. }) = thinking {
            assert!(text.contains("think through"));
        }
    }

    #[test]
    fn test_gemini_thinking_extraction_thinking() {
        let response = serde_json::json!({
            "candidates": [{
                "content": {
                    "thinking": "Analyzing the data..."
                }
            }]
        });

        let thinking = gemini_thinking::extract_thinking(&response);
        assert!(thinking.is_some());
        if let Some(ThinkingContent::Text { text, .. }) = thinking {
            assert!(text.contains("Analyzing"));
        }
    }

    #[test]
    fn test_gemini_thinking_extraction_missing() {
        let response = serde_json::json!({
            "candidates": [{
                "content": {
                    "text": "The answer is 42."
                }
            }]
        });

        let thinking = gemini_thinking::extract_thinking(&response);
        assert!(thinking.is_none());
    }

    #[test]
    fn test_gemini_usage_extraction() {
        let response = serde_json::json!({
            "usageMetadata": {
                "thinkingTokenCount": 1200
            }
        });

        let usage = gemini_thinking::extract_usage(&response);
        assert!(usage.is_some());
        let usage = usage.unwrap();
        assert_eq!(usage.thinking_tokens, Some(1200));
        assert_eq!(usage.provider, Some("gemini".to_string()));
    }

    #[test]
    fn test_gemini_usage_extraction_missing() {
        let response = serde_json::json!({
            "usageMetadata": {
                "totalTokenCount": 2000
            }
        });

        let usage = gemini_thinking::extract_usage(&response);
        assert!(usage.is_none());
    }

    // ============================================================================
    // OpenRouter Thinking Tests
    // ============================================================================

    #[test]
    fn test_openrouter_thinking_detection() {
        assert!(openrouter_thinking::supports_thinking("openai/o1-preview"));
        assert!(openrouter_thinking::supports_thinking("o1-mini"));
        assert!(openrouter_thinking::supports_thinking(
            "anthropic/claude-3-opus"
        ));
        assert!(openrouter_thinking::supports_thinking("claude-3-sonnet"));
        assert!(openrouter_thinking::supports_thinking(
            "deepseek/deepseek-r1"
        ));
        assert!(openrouter_thinking::supports_thinking("deepseek-reasoner"));
        assert!(openrouter_thinking::supports_thinking(
            "google/gemini-thinking"
        ));
        assert!(openrouter_thinking::supports_thinking(
            "gemini-2.0-flash-thinking"
        ));
        assert!(!openrouter_thinking::supports_thinking("gpt-4"));
    }

    #[test]
    fn test_openrouter_provider_detection() {
        assert_eq!(
            openrouter_thinking::detect_provider("openai/o1-preview"),
            "openai"
        );
        assert_eq!(openrouter_thinking::detect_provider("o1-mini"), "openai");
        assert_eq!(openrouter_thinking::detect_provider("o3-mini"), "openai");
        assert_eq!(
            openrouter_thinking::detect_provider("anthropic/claude-3-opus"),
            "anthropic"
        );
        assert_eq!(
            openrouter_thinking::detect_provider("claude-3-5-sonnet"),
            "anthropic"
        );
        assert_eq!(
            openrouter_thinking::detect_provider("deepseek/deepseek-r1"),
            "deepseek"
        );
        assert_eq!(
            openrouter_thinking::detect_provider("google/gemini-thinking"),
            "gemini"
        );
        assert_eq!(openrouter_thinking::detect_provider("gemini-pro"), "gemini");
        assert_eq!(
            openrouter_thinking::detect_provider("unknown-model"),
            "unknown"
        );
    }

    #[test]
    fn test_openrouter_capabilities_openai() {
        let caps = openrouter_thinking::capabilities("openai/o1-preview");
        assert!(caps.supports_thinking);
        assert!(!caps.supports_streaming_thinking);
    }

    #[test]
    fn test_openrouter_capabilities_anthropic() {
        let caps = openrouter_thinking::capabilities("anthropic/claude-3-opus");
        assert!(caps.supports_thinking);
        assert!(caps.supports_streaming_thinking);
    }

    #[test]
    fn test_openrouter_capabilities_deepseek() {
        let caps = openrouter_thinking::capabilities("deepseek/deepseek-r1");
        assert!(caps.supports_thinking);
        assert!(caps.thinking_always_on);
    }

    #[test]
    fn test_openrouter_capabilities_gemini() {
        let caps = openrouter_thinking::capabilities("google/gemini-thinking");
        assert!(caps.supports_thinking);
    }

    #[test]
    fn test_openrouter_capabilities_unknown() {
        let caps = openrouter_thinking::capabilities("unknown-model");
        assert!(!caps.supports_thinking);
    }

    #[test]
    fn test_openrouter_transform_config_openai() {
        let config = ThinkingConfig {
            enabled: true,
            budget_tokens: Some(10000),
            effort: Some(ThinkingEffort::High),
            include_thinking: true,
            extra_params: Default::default(),
        };

        let result = openrouter_thinking::transform_config(&config, "openai/o1-preview").unwrap();
        assert!(result.get("max_reasoning_tokens").is_some());
        assert!(result.get("reasoning_effort").is_some());
    }

    #[test]
    fn test_openrouter_transform_config_anthropic() {
        let config = ThinkingConfig {
            enabled: true,
            budget_tokens: Some(50000),
            effort: Some(ThinkingEffort::High),
            include_thinking: true,
            extra_params: Default::default(),
        };

        let result =
            openrouter_thinking::transform_config(&config, "anthropic/claude-3-opus").unwrap();
        assert!(result.get("thinking").is_some());
    }

    #[test]
    fn test_openrouter_transform_config_deepseek() {
        let config = ThinkingConfig {
            enabled: true,
            budget_tokens: Some(5000),
            effort: Some(ThinkingEffort::Medium),
            include_thinking: true,
            extra_params: Default::default(),
        };

        let result =
            openrouter_thinking::transform_config(&config, "deepseek/deepseek-r1").unwrap();
        assert!(result.get("reasoning_effort").is_some());
    }

    #[test]
    fn test_openrouter_transform_config_gemini() {
        let config = ThinkingConfig {
            enabled: true,
            budget_tokens: Some(15000),
            effort: Some(ThinkingEffort::High),
            include_thinking: true,
            extra_params: Default::default(),
        };

        let result =
            openrouter_thinking::transform_config(&config, "google/gemini-thinking").unwrap();
        assert!(result.get("enableThinking").is_some());
    }

    #[test]
    fn test_openrouter_transform_config_unknown() {
        let config = ThinkingConfig {
            enabled: true,
            budget_tokens: Some(10000),
            effort: Some(ThinkingEffort::High),
            include_thinking: true,
            extra_params: Default::default(),
        };

        let result = openrouter_thinking::transform_config(&config, "unknown-model").unwrap();
        assert!(result.as_object().unwrap().is_empty());
    }

    #[test]
    fn test_openrouter_extract_thinking_openai() {
        let response = serde_json::json!({
            "choices": [{
                "message": {
                    "reasoning": "OpenAI reasoning content"
                }
            }]
        });

        let thinking = openrouter_thinking::extract_thinking(&response);
        assert!(thinking.is_some());
    }

    #[test]
    fn test_openrouter_extract_thinking_deepseek() {
        let response = serde_json::json!({
            "choices": [{
                "message": {
                    "reasoning_content": "DeepSeek reasoning content"
                }
            }]
        });

        let thinking = openrouter_thinking::extract_thinking(&response);
        assert!(thinking.is_some());
    }

    #[test]
    fn test_openrouter_extract_thinking_anthropic() {
        let response = serde_json::json!({
            "content": [
                {
                    "type": "thinking",
                    "thinking": "Anthropic thinking content"
                }
            ]
        });

        let thinking = openrouter_thinking::extract_thinking(&response);
        assert!(thinking.is_some());
    }

    #[test]
    fn test_openrouter_extract_thinking_gemini() {
        let response = serde_json::json!({
            "candidates": [{
                "content": {
                    "thoughts": "Gemini thoughts content"
                }
            }]
        });

        let thinking = openrouter_thinking::extract_thinking(&response);
        assert!(thinking.is_some());
    }

    #[test]
    fn test_openrouter_extract_thinking_none() {
        let response = serde_json::json!({
            "choices": [{
                "message": {
                    "content": "Regular response"
                }
            }]
        });

        let thinking = openrouter_thinking::extract_thinking(&response);
        assert!(thinking.is_none());
    }

    #[test]
    fn test_openrouter_extract_usage_openai() {
        let response = serde_json::json!({
            "usage": {
                "reasoning_tokens": 500
            }
        });

        let usage = openrouter_thinking::extract_usage(&response);
        assert!(usage.is_some());
        let usage = usage.unwrap();
        assert_eq!(usage.thinking_tokens, Some(500));
        assert_eq!(usage.provider, Some("openrouter".to_string()));
    }

    #[test]
    fn test_openrouter_extract_usage_deepseek() {
        let response = serde_json::json!({
            "usage": {
                "reasoning_tokens": 800
            }
        });

        let usage = openrouter_thinking::extract_usage(&response);
        assert!(usage.is_some());
        assert_eq!(usage.unwrap().provider, Some("openrouter".to_string()));
    }

    #[test]
    fn test_openrouter_extract_usage_anthropic() {
        let response = serde_json::json!({
            "usage": {
                "thinking_tokens": 600
            }
        });

        let usage = openrouter_thinking::extract_usage(&response);
        assert!(usage.is_some());
        assert_eq!(usage.unwrap().provider, Some("openrouter".to_string()));
    }

    #[test]
    fn test_openrouter_extract_usage_gemini() {
        let response = serde_json::json!({
            "usageMetadata": {
                "thinkingTokenCount": 1000
            }
        });

        let usage = openrouter_thinking::extract_usage(&response);
        assert!(usage.is_some());
        assert_eq!(usage.unwrap().provider, Some("openrouter".to_string()));
    }

    #[test]
    fn test_openrouter_extract_usage_none() {
        let response = serde_json::json!({
            "usage": {
                "total_tokens": 100
            }
        });

        let usage = openrouter_thinking::extract_usage(&response);
        assert!(usage.is_none());
    }

    // ============================================================================
    // ThinkingProvider Trait Default Methods Tests
    // ============================================================================

    struct MockThinkingProvider;

    impl ThinkingProvider for MockThinkingProvider {
        fn supports_thinking(&self, _model: &str) -> bool {
            true
        }

        fn thinking_capabilities(&self, _model: &str) -> ThinkingCapabilities {
            ThinkingCapabilities {
                supports_thinking: true,
                supports_streaming_thinking: true,
                max_thinking_tokens: Some(5000),
                supported_efforts: vec![ThinkingEffort::Medium, ThinkingEffort::High],
                thinking_models: vec!["test-model".to_string()],
                can_return_thinking: true,
                thinking_always_on: false,
            }
        }

        fn transform_thinking_config(
            &self,
            _config: &ThinkingConfig,
            _model: &str,
        ) -> Result<Value, ProviderError> {
            Ok(serde_json::json!({}))
        }

        fn extract_thinking(&self, _response: &Value) -> Option<ThinkingContent> {
            None
        }

        fn extract_thinking_usage(&self, _response: &Value) -> Option<ThinkingUsage> {
            None
        }
    }

    #[test]
    fn test_thinking_provider_default_effort() {
        let provider = MockThinkingProvider;
        assert_eq!(provider.default_thinking_effort(), ThinkingEffort::Medium);
    }

    #[test]
    fn test_thinking_provider_max_thinking_tokens() {
        let provider = MockThinkingProvider;
        assert_eq!(provider.max_thinking_tokens("test-model"), Some(5000));
    }

    #[test]
    fn test_thinking_provider_supports_streaming_thinking() {
        let provider = MockThinkingProvider;
        assert!(provider.supports_streaming_thinking("test-model"));
    }
}
