//! DeepSeek Model Registry
//!
//! Model registry system with support for dynamic loading and feature detection

use std::collections::HashMap;
use std::sync::OnceLock;

use crate::core::providers::base::get_pricing_db;
use crate::core::types::common::ModelInfo;

/// Type alias for model definition tuple: (id, name, context_len, output_len, input_cost, output_cost, has_reasoning)
type ModelDefinition<'a> = (&'a str, &'a str, u32, Option<u32>, f64, f64, bool);

/// Model
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ModelFeature {
    /// Model
    ReasoningMode,
    /// Function calling support
    FunctionCalling,
    /// Vision support
    VisionSupport,
    /// Response
    StreamingSupport,
    /// System message support
    SystemMessages,
    /// Tool calling support
    ToolCalling,
}

/// Model
#[derive(Debug, Clone)]
pub struct ModelSpec {
    /// Model
    pub model_info: ModelInfo,
    /// Supported features
    pub features: Vec<ModelFeature>,
    /// Configuration
    pub config: ModelConfig,
}

/// Configuration
#[derive(Debug, Clone, Default)]
pub struct ModelConfig {
    /// Request
    pub requires_special_formatting: bool,
    /// Request
    pub max_concurrent_requests: Option<u32>,
    /// Custom parameter mapping
    pub custom_params: HashMap<String, String>,
}

/// Model
pub struct DeepSeekModelRegistry {
    models: HashMap<String, ModelSpec>,
}

impl Default for DeepSeekModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl DeepSeekModelRegistry {
    /// Create
    pub fn new() -> Self {
        let mut registry = Self {
            models: HashMap::new(),
        };
        registry.load_models();
        registry
    }

    /// Model
    fn load_models(&mut self) {
        let pricing_db = get_pricing_db();
        let model_ids = pricing_db.get_provider_models("deepseek");

        for model_id in &model_ids {
            if let Some(model_info) = pricing_db.to_model_info(model_id, "deepseek") {
                let features = self.detect_features(&model_info);
                let config = self.create_config(&model_info);

                self.models.insert(
                    model_id.clone(),
                    ModelSpec {
                        model_info,
                        features,
                        config,
                    },
                );
            }
        }

        // Default
        if self.models.is_empty() {
            self.add_default_models();
        }
    }

    /// Model
    fn detect_features(&self, model_info: &ModelInfo) -> Vec<ModelFeature> {
        let mut features = vec![ModelFeature::SystemMessages, ModelFeature::StreamingSupport];

        if model_info.supports_tools {
            features.push(ModelFeature::FunctionCalling);
            features.push(ModelFeature::ToolCalling);
        }

        if model_info.supports_multimodal {
            features.push(ModelFeature::VisionSupport);
        }

        // DeepSeek-specific reasoning mode detection
        if model_info.id.contains("reasoning") || model_info.id.contains("r1") {
            features.push(ModelFeature::ReasoningMode);
        }

        features
    }

    /// Create
    fn create_config(&self, model_info: &ModelInfo) -> ModelConfig {
        let mut config = ModelConfig::default();

        // Some DeepSeek models may require special formatting
        if model_info.id.contains("reasoning") {
            config.requires_special_formatting = true;
            config
                .custom_params
                .insert("reasoning_effort".to_string(), "medium".to_string());
        }

        // Settings
        config.max_concurrent_requests = Some(match model_info.id.as_str() {
            "deepseek-chat" => 10,    // Non-thinking mode can handle higher concurrency
            "deepseek-reasoner" => 3, // Thinking mode requires more resources, limit concurrency
            _ => 5,
        });

        config
    }

    /// Default models for DeepSeek
    fn add_default_models(&mut self) {
        // Model data: (id, name, context_len, output_len, input_cost, output_cost, has_reasoning)
        let default_models: Vec<ModelDefinition> = vec![
            // DeepSeek V3 (December 2024 - Latest)
            (
                "deepseek-chat",
                "DeepSeek V3",
                128_000,
                Some(8_192),
                0.00014,   // $0.14/1M input (cache miss)
                0.00028,   // $0.28/1M output
                false,
            ),
            (
                "deepseek-coder",
                "DeepSeek Coder V3",
                128_000,
                Some(8_192),
                0.00014,
                0.00028,
                false,
            ),
            // DeepSeek R1 (Reasoning - January 2025)
            (
                "deepseek-r1",
                "DeepSeek R1",
                128_000,
                Some(64_000),
                0.00055,   // $0.55/1M input (cache miss)
                0.00219,   // $2.19/1M output
                true,
            ),
            (
                "deepseek-r1-lite",
                "DeepSeek R1 Lite Preview",
                128_000,
                Some(32_000),
                0.00014,   // Same as V3 pricing
                0.00028,
                true,
            ),
            (
                "deepseek-reasoner",
                "DeepSeek Reasoner (R1)",
                128_000,
                Some(64_000),
                0.00055,
                0.00219,
                true,
            ),
            // DeepSeek R1 Distilled Models
            (
                "deepseek-r1-distill-llama-70b",
                "DeepSeek R1 Distill Llama 70B",
                128_000,
                Some(32_000),
                0.00055,
                0.00219,
                true,
            ),
            (
                "deepseek-r1-distill-qwen-32b",
                "DeepSeek R1 Distill Qwen 32B",
                128_000,
                Some(32_000),
                0.00014,
                0.00028,
                true,
            ),
            (
                "deepseek-r1-distill-qwen-14b",
                "DeepSeek R1 Distill Qwen 14B",
                128_000,
                Some(32_000),
                0.00014,
                0.00028,
                true,
            ),
            (
                "deepseek-r1-distill-qwen-7b",
                "DeepSeek R1 Distill Qwen 7B",
                128_000,
                Some(16_000),
                0.00014,
                0.00028,
                true,
            ),
            (
                "deepseek-r1-distill-llama-8b",
                "DeepSeek R1 Distill Llama 8B",
                128_000,
                Some(16_000),
                0.00014,
                0.00028,
                true,
            ),
            (
                "deepseek-r1-distill-qwen-1.5b",
                "DeepSeek R1 Distill Qwen 1.5B",
                128_000,
                Some(8_000),
                0.00014,
                0.00028,
                true,
            ),
        ];

        for (id, name, context_len, output_len, input_cost, output_cost, has_reasoning) in default_models {
            let model_info = ModelInfo {
                id: id.to_string(),
                name: name.to_string(),
                provider: "deepseek".to_string(),
                max_context_length: context_len,
                max_output_length: output_len,
                supports_streaming: true,
                supports_tools: true,
                supports_multimodal: false,
                input_cost_per_1k_tokens: Some(input_cost),
                output_cost_per_1k_tokens: Some(output_cost),
                currency: "USD".to_string(),
                capabilities: vec![],
                created_at: None,
                updated_at: None,
                metadata: {
                    let mut m = HashMap::new();
                    if has_reasoning {
                        m.insert("has_reasoning".to_string(), serde_json::Value::Bool(true));
                    }
                    m
                },
            };

            let features = self.detect_features(&model_info);
            let config = self.create_config(&model_info);

            self.models.insert(
                id.to_string(),
                ModelSpec {
                    model_info,
                    features,
                    config,
                },
            );
        }
    }

    /// Model
    pub fn get_all_models(&self) -> Vec<ModelInfo> {
        self.models
            .values()
            .map(|spec| spec.model_info.clone())
            .collect()
    }

    /// Model
    pub fn get_model_spec(&self, model_id: &str) -> Option<&ModelSpec> {
        self.models.get(model_id)
    }

    /// Check
    pub fn supports_feature(&self, model_id: &str, feature: &ModelFeature) -> bool {
        self.models
            .get(model_id)
            .map(|spec| spec.features.contains(feature))
            .unwrap_or(false)
    }

    /// Model
    pub fn get_models_with_feature(&self, feature: &ModelFeature) -> Vec<String> {
        self.models
            .iter()
            .filter_map(|(id, spec)| {
                if spec.features.contains(feature) {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Model
    pub fn get_custom_params(&self, model_id: &str) -> Option<&HashMap<String, String>> {
        self.models
            .get(model_id)
            .map(|spec| &spec.config.custom_params)
    }
}

/// Model
static DEEPSEEK_REGISTRY: OnceLock<DeepSeekModelRegistry> = OnceLock::new();

/// Model
pub fn get_deepseek_registry() -> &'static DeepSeekModelRegistry {
    DEEPSEEK_REGISTRY.get_or_init(DeepSeekModelRegistry::new)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_registry_creation() {
        let registry = DeepSeekModelRegistry::new();
        assert!(!registry.get_all_models().is_empty());
    }

    #[test]
    fn test_feature_detection() {
        let registry = get_deepseek_registry();
        let models = registry.get_all_models();

        // Should have at least one model
        assert!(!models.is_empty());

        // Check
        for model in &models {
            assert!(registry.supports_feature(&model.id, &ModelFeature::SystemMessages));
            assert!(registry.supports_feature(&model.id, &ModelFeature::StreamingSupport));
        }
    }

    #[test]
    fn test_models_with_feature() {
        let registry = get_deepseek_registry();
        let tool_models = registry.get_models_with_feature(&ModelFeature::ToolCalling);
        assert!(!tool_models.is_empty());
    }
}
