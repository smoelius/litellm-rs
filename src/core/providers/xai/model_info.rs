//! xAI Model Information
//!
//! Model configurations for Grok models

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;

/// xAI model identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum XAIModel {
    // Grok models
    Grok2,
    Grok2Mini,
    GrokBeta,
    GrokVision,
}

/// Model configuration
#[derive(Debug, Clone)]
pub struct ModelInfo {
    /// Model ID as used in API
    pub model_id: &'static str,
    /// Display name
    pub display_name: &'static str,
    /// Maximum context length
    pub context_length: u32,
    /// Maximum output tokens
    pub max_output_tokens: u32,
    /// Whether the model supports tools/functions
    pub supports_tools: bool,
    /// Whether the model supports vision
    pub supports_vision: bool,
    /// Whether the model supports web search
    pub supports_web_search: bool,
    /// Whether the model has reasoning capabilities
    pub supports_reasoning: bool,
    /// Input cost per million tokens (in USD)
    pub input_cost_per_million: f64,
    /// Output cost per million tokens (in USD)
    pub output_cost_per_million: f64,
    /// Reasoning tokens cost per million (if applicable)
    pub reasoning_cost_per_million: Option<f64>,
}

/// Static model configurations
static MODEL_CONFIGS: LazyLock<HashMap<&'static str, ModelInfo>> = LazyLock::new(|| {
    let mut configs = HashMap::new();

    // Grok-2 (flagship model)
    configs.insert(
        "grok-2",
        ModelInfo {
            model_id: "grok-2",
            display_name: "Grok-2",
            context_length: 131072, // 128K context
            max_output_tokens: 32768,
            supports_tools: true,
            supports_vision: false,
            supports_web_search: true,
            supports_reasoning: true,
            input_cost_per_million: 2.0,
            output_cost_per_million: 10.0,
            reasoning_cost_per_million: Some(10.0),
        },
    );

    // Grok-2 Mini (smaller, faster model)
    configs.insert(
        "grok-2-mini",
        ModelInfo {
            model_id: "grok-2-mini",
            display_name: "Grok-2 Mini",
            context_length: 131072, // 128K context
            max_output_tokens: 16384,
            supports_tools: true,
            supports_vision: false,
            supports_web_search: true,
            supports_reasoning: false,
            input_cost_per_million: 0.5,
            output_cost_per_million: 2.0,
            reasoning_cost_per_million: None,
        },
    );

    // Grok Beta (experimental features)
    configs.insert(
        "grok-beta",
        ModelInfo {
            model_id: "grok-beta",
            display_name: "Grok Beta",
            context_length: 131072,
            max_output_tokens: 32768,
            supports_tools: true,
            supports_vision: true,
            supports_web_search: true,
            supports_reasoning: true,
            input_cost_per_million: 5.0,
            output_cost_per_million: 15.0,
            reasoning_cost_per_million: Some(15.0),
        },
    );

    // Grok Vision (multimodal)
    configs.insert(
        "grok-vision-beta",
        ModelInfo {
            model_id: "grok-vision-beta",
            display_name: "Grok Vision Beta",
            context_length: 8192,
            max_output_tokens: 4096,
            supports_tools: true,
            supports_vision: true,
            supports_web_search: true,
            supports_reasoning: false,
            input_cost_per_million: 5.0,
            output_cost_per_million: 15.0,
            reasoning_cost_per_million: None,
        },
    );

    configs
});

/// Get model information by ID
pub fn get_model_info(model_id: &str) -> Option<&'static ModelInfo> {
    // Handle xai/ prefix
    let model_id = model_id.strip_prefix("xai/").unwrap_or(model_id);
    MODEL_CONFIGS.get(model_id)
}

/// Get all available model IDs
pub fn get_available_models() -> Vec<&'static str> {
    MODEL_CONFIGS.keys().copied().collect()
}

/// Check if a model supports reasoning tokens
pub fn supports_reasoning_tokens(model_id: &str) -> bool {
    get_model_info(model_id)
        .map(|info| info.supports_reasoning)
        .unwrap_or(false)
}

/// Calculate cost including reasoning tokens
pub fn calculate_cost_with_reasoning(
    model_id: &str,
    input_tokens: u32,
    output_tokens: u32,
    reasoning_tokens: Option<u32>,
) -> Option<f64> {
    let model_info = get_model_info(model_id)?;

    let input_cost = (input_tokens as f64) * (model_info.input_cost_per_million / 1_000_000.0);
    let output_cost = (output_tokens as f64) * (model_info.output_cost_per_million / 1_000_000.0);

    let reasoning_cost = if let (Some(reasoning_tokens), Some(reasoning_rate)) =
        (reasoning_tokens, model_info.reasoning_cost_per_million)
    {
        (reasoning_tokens as f64) * (reasoning_rate / 1_000_000.0)
    } else {
        0.0
    };

    Some(input_cost + output_cost + reasoning_cost)
}

impl XAIModel {
    /// Get the API model ID
    pub fn model_id(&self) -> &'static str {
        match self {
            XAIModel::Grok2 => "grok-2",
            XAIModel::Grok2Mini => "grok-2-mini",
            XAIModel::GrokBeta => "grok-beta",
            XAIModel::GrokVision => "grok-vision-beta",
        }
    }

    /// Get model information
    pub fn info(&self) -> &'static ModelInfo {
        get_model_info(self.model_id()).expect("Model info should exist for enum variant")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_info() {
        // Test Grok-2 model info
        let info = get_model_info("grok-2").unwrap();
        assert_eq!(info.model_id, "grok-2");
        assert_eq!(info.context_length, 131072);
        assert!(info.supports_reasoning);
        assert!(info.supports_web_search);
        assert!(info.reasoning_cost_per_million.is_some());

        // Test Grok-2-Mini model info
        let info = get_model_info("grok-2-mini").unwrap();
        assert_eq!(info.model_id, "grok-2-mini");
        assert!(!info.supports_reasoning);
        assert!(info.reasoning_cost_per_million.is_none());

        // Test with xai/ prefix
        let info = get_model_info("xai/grok-2").unwrap();
        assert_eq!(info.model_id, "grok-2");
    }

    #[test]
    fn test_available_models() {
        let models = get_available_models();
        assert!(models.contains(&"grok-2"));
        assert!(models.contains(&"grok-2-mini"));
        assert!(models.contains(&"grok-beta"));
        assert!(models.contains(&"grok-vision-beta"));
    }

    #[test]
    fn test_supports_reasoning() {
        assert!(supports_reasoning_tokens("grok-2"));
        assert!(supports_reasoning_tokens("grok-beta"));
        assert!(!supports_reasoning_tokens("grok-2-mini"));
        assert!(!supports_reasoning_tokens("grok-vision-beta"));
    }

    #[test]
    fn test_cost_calculation() {
        // Test basic cost calculation
        let cost = calculate_cost_with_reasoning("grok-2", 1000, 500, None).unwrap();
        let expected = (1000.0 * 2.0 / 1_000_000.0) + (500.0 * 10.0 / 1_000_000.0);
        assert!((cost - expected).abs() < 0.0001);

        // Test with reasoning tokens
        let cost = calculate_cost_with_reasoning("grok-2", 1000, 500, Some(200)).unwrap();
        let expected = (1000.0 * 2.0 / 1_000_000.0)
            + (500.0 * 10.0 / 1_000_000.0)
            + (200.0 * 10.0 / 1_000_000.0);
        assert!((cost - expected).abs() < 0.0001);

        // Test model without reasoning support
        let cost = calculate_cost_with_reasoning("grok-2-mini", 1000, 500, Some(200)).unwrap();
        let expected = (1000.0 * 0.5 / 1_000_000.0) + (500.0 * 2.0 / 1_000_000.0); // reasoning ignored
        assert!((cost - expected).abs() < 0.0001);
    }

    #[test]
    fn test_xai_model_enum() {
        assert_eq!(XAIModel::Grok2.model_id(), "grok-2");
        assert_eq!(XAIModel::Grok2Mini.model_id(), "grok-2-mini");
        assert_eq!(XAIModel::GrokBeta.model_id(), "grok-beta");
        assert_eq!(XAIModel::GrokVision.model_id(), "grok-vision-beta");

        let info = XAIModel::Grok2.info();
        assert_eq!(info.display_name, "Grok-2");
    }
}
