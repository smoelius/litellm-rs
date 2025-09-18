//! Anthropic Model Registry
//!
//! Unified model registry system with integrated pricing and capability information

use std::collections::HashMap;
use std::sync::OnceLock;

use crate::core::types::common::ModelInfo;

/// Model
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ModelFeature {
    /// Multimodal support (images, documents)
    MultimodalSupport,
    /// Tool calling support
    ToolCalling,
    /// Function calling support
    FunctionCalling,
    /// Streaming response support
    StreamingSupport,
    /// Cache control support
    CacheControl,
    /// System message support
    SystemMessages,
    /// Batch processing support
    BatchProcessing,
    /// Thinking mode support
    ThinkingMode,
    /// Computer tool support
    ComputerUse,
}

/// Model
#[derive(Debug, Clone, PartialEq)]
pub enum AnthropicModelFamily {
    /// Claude 3.5 Sonnet models
    Claude35Sonnet,
    /// Claude 3 Opus models
    Claude3Opus,
    /// Claude 3 Sonnet models
    Claude3Sonnet,
    /// Claude 3 Haiku models
    Claude3Haiku,
    /// Claude 2.1 models
    Claude21,
    /// Claude 2 models
    Claude2,
    /// Claude Instant models
    ClaudeInstant,
}

/// Model pricing information
#[derive(Debug, Clone)]
pub struct ModelPricing {
    /// Input token price (USD per million tokens)
    pub input_price: f64,
    /// Output token price (USD per million tokens)
    pub output_price: f64,
    /// Cache write price (optional)
    pub cache_write_price: Option<f64>,
    /// Cache read price (optional)
    pub cache_read_price: Option<f64>,
    /// Batch processing discount
    pub batch_discount: Option<f64>,
}

/// Model limits and constraints
#[derive(Debug, Clone)]
pub struct ModelLimits {
    /// Maximum context length
    pub max_context_length: u32,
    /// Maximum output tokens
    pub max_output_tokens: u32,
    /// Maximum number of images
    pub max_images: Option<u32>,
    /// Maximum document size (MB)
    pub max_document_size_mb: Option<u32>,
}

/// Model specification
#[derive(Debug, Clone)]
pub struct ModelSpec {
    /// Model information
    pub model_info: ModelInfo,
    /// Model family
    pub family: AnthropicModelFamily,
    /// Supported features
    pub features: Vec<ModelFeature>,
    /// Pricing information
    pub pricing: ModelPricing,
    /// Limits information
    pub limits: ModelLimits,
    /// Model configuration
    pub config: ModelConfig,
}

/// Model configuration settings
#[derive(Debug, Clone, Default)]
pub struct ModelConfig {
    /// Requires special formatting
    pub requires_special_formatting: bool,
    /// Maximum concurrent requests
    pub max_concurrent_requests: Option<u32>,
    /// Custom parameter mapping
    pub custom_params: HashMap<String, String>,
}

/// Model registry
#[derive(Debug, Clone)]
pub struct AnthropicModelRegistry {
    models: HashMap<String, ModelSpec>,
}

impl AnthropicModelRegistry {
    /// Create
    pub fn new() -> Self {
        let mut registry = Self {
            models: HashMap::new(),
        };
        registry.initialize_models();
        registry
    }

    /// Initialize model registry
    fn initialize_models(&mut self) {
        // Claude 3.5 Sonnet
        self.register_model(
            "claude-3-5-sonnet-20241022",
            ModelSpec {
                model_info: ModelInfo {
                    id: "claude-3-5-sonnet-20241022".to_string(),
                    name: "Claude 3.5 Sonnet".to_string(),
                    provider: "anthropic".to_string(),
                    max_context_length: 200_000,
                    max_output_length: Some(8_192),
                    supports_streaming: true,
                    supports_tools: true,
                    supports_multimodal: true,
                    input_cost_per_1k_tokens: Some(0.003),
                    output_cost_per_1k_tokens: Some(0.015),
                    currency: "USD".to_string(),
                    capabilities: vec![
                        crate::core::types::common::ProviderCapability::ChatCompletion,
                        crate::core::types::common::ProviderCapability::ChatCompletionStream,
                        crate::core::types::common::ProviderCapability::ToolCalling,
                    ],
                    created_at: None,
                    updated_at: None,
                    metadata: std::collections::HashMap::new(),
                },
                family: AnthropicModelFamily::Claude35Sonnet,
                features: vec![
                    ModelFeature::MultimodalSupport,
                    ModelFeature::ToolCalling,
                    ModelFeature::FunctionCalling,
                    ModelFeature::StreamingSupport,
                    ModelFeature::CacheControl,
                    ModelFeature::SystemMessages,
                    ModelFeature::BatchProcessing,
                    ModelFeature::ThinkingMode,
                    ModelFeature::ComputerUse,
                ],
                pricing: ModelPricing {
                    input_price: 3.0,
                    output_price: 15.0,
                    cache_write_price: Some(3.75),
                    cache_read_price: Some(0.30),
                    batch_discount: Some(0.5),
                },
                limits: ModelLimits {
                    max_context_length: 200_000,
                    max_output_tokens: 8_192,
                    max_images: Some(20),
                    max_document_size_mb: Some(32),
                },
                config: ModelConfig::default(),
            },
        );

        // Claude 3 Opus
        self.register_model(
            "claude-3-opus-20240229",
            ModelSpec {
                model_info: ModelInfo {
                    id: "claude-3-opus-20240229".to_string(),
                    name: "Claude 3 Opus".to_string(),
                    provider: "anthropic".to_string(),
                    max_context_length: 200_000,
                    max_output_length: Some(4_096),
                    supports_streaming: true,
                    supports_tools: true,
                    supports_multimodal: true,
                    input_cost_per_1k_tokens: Some(0.015),
                    output_cost_per_1k_tokens: Some(0.075),
                    currency: "USD".to_string(),
                    capabilities: vec![
                        crate::core::types::common::ProviderCapability::ChatCompletion,
                        crate::core::types::common::ProviderCapability::ChatCompletionStream,
                        crate::core::types::common::ProviderCapability::ToolCalling,
                    ],
                    created_at: None,
                    updated_at: None,
                    metadata: std::collections::HashMap::new(),
                },
                family: AnthropicModelFamily::Claude3Opus,
                features: vec![
                    ModelFeature::MultimodalSupport,
                    ModelFeature::ToolCalling,
                    ModelFeature::FunctionCalling,
                    ModelFeature::StreamingSupport,
                    ModelFeature::CacheControl,
                    ModelFeature::SystemMessages,
                    ModelFeature::BatchProcessing,
                ],
                pricing: ModelPricing {
                    input_price: 15.0,
                    output_price: 75.0,
                    cache_write_price: Some(18.75),
                    cache_read_price: Some(1.50),
                    batch_discount: Some(0.5),
                },
                limits: ModelLimits {
                    max_context_length: 200_000,
                    max_output_tokens: 4_096,
                    max_images: Some(20),
                    max_document_size_mb: Some(32),
                },
                config: ModelConfig::default(),
            },
        );

        // Claude 3 Sonnet
        self.register_model(
            "claude-3-sonnet-20240229",
            ModelSpec {
                model_info: ModelInfo {
                    id: "claude-3-sonnet-20240229".to_string(),
                    name: "Claude 3 Sonnet".to_string(),
                    provider: "anthropic".to_string(),
                    max_context_length: 200_000,
                    max_output_length: Some(4_096),
                    supports_streaming: true,
                    supports_tools: true,
                    supports_multimodal: true,
                    input_cost_per_1k_tokens: Some(0.003),
                    output_cost_per_1k_tokens: Some(0.015),
                    currency: "USD".to_string(),
                    capabilities: vec![
                        crate::core::types::common::ProviderCapability::ChatCompletion,
                        crate::core::types::common::ProviderCapability::ChatCompletionStream,
                        crate::core::types::common::ProviderCapability::ToolCalling,
                    ],
                    created_at: None,
                    updated_at: None,
                    metadata: std::collections::HashMap::new(),
                },
                family: AnthropicModelFamily::Claude3Sonnet,
                features: vec![
                    ModelFeature::MultimodalSupport,
                    ModelFeature::ToolCalling,
                    ModelFeature::FunctionCalling,
                    ModelFeature::StreamingSupport,
                    ModelFeature::CacheControl,
                    ModelFeature::SystemMessages,
                    ModelFeature::BatchProcessing,
                ],
                pricing: ModelPricing {
                    input_price: 3.0,
                    output_price: 15.0,
                    cache_write_price: Some(3.75),
                    cache_read_price: Some(0.30),
                    batch_discount: Some(0.5),
                },
                limits: ModelLimits {
                    max_context_length: 200_000,
                    max_output_tokens: 4_096,
                    max_images: Some(20),
                    max_document_size_mb: Some(32),
                },
                config: ModelConfig::default(),
            },
        );

        // Claude 3 Haiku
        self.register_model(
            "claude-3-haiku-20240307",
            ModelSpec {
                model_info: ModelInfo {
                    id: "claude-3-haiku-20240307".to_string(),
                    name: "Claude 3 Haiku".to_string(),
                    provider: "anthropic".to_string(),
                    max_context_length: 200_000,
                    max_output_length: Some(4_096),
                    supports_streaming: true,
                    supports_tools: true,
                    supports_multimodal: true,
                    input_cost_per_1k_tokens: Some(0.00025),
                    output_cost_per_1k_tokens: Some(0.00125),
                    currency: "USD".to_string(),
                    capabilities: vec![
                        crate::core::types::common::ProviderCapability::ChatCompletion,
                        crate::core::types::common::ProviderCapability::ChatCompletionStream,
                        crate::core::types::common::ProviderCapability::ToolCalling,
                    ],
                    created_at: None,
                    updated_at: None,
                    metadata: std::collections::HashMap::new(),
                },
                family: AnthropicModelFamily::Claude3Haiku,
                features: vec![
                    ModelFeature::MultimodalSupport,
                    ModelFeature::ToolCalling,
                    ModelFeature::FunctionCalling,
                    ModelFeature::StreamingSupport,
                    ModelFeature::CacheControl,
                    ModelFeature::SystemMessages,
                    ModelFeature::BatchProcessing,
                ],
                pricing: ModelPricing {
                    input_price: 0.25,
                    output_price: 1.25,
                    cache_write_price: Some(0.30),
                    cache_read_price: Some(0.03),
                    batch_discount: Some(0.5),
                },
                limits: ModelLimits {
                    max_context_length: 200_000,
                    max_output_tokens: 4_096,
                    max_images: Some(20),
                    max_document_size_mb: Some(32),
                },
                config: ModelConfig::default(),
            },
        );

        // Claude 2.1
        self.register_model(
            "claude-2.1",
            ModelSpec {
                model_info: ModelInfo {
                    id: "claude-2.1".to_string(),
                    name: "Claude 2.1".to_string(),
                    provider: "anthropic".to_string(),
                    max_context_length: 200_000,
                    max_output_length: Some(4_096),
                    supports_streaming: true,
                    supports_tools: false,
                    supports_multimodal: false,
                    input_cost_per_1k_tokens: Some(0.008),
                    output_cost_per_1k_tokens: Some(0.024),
                    currency: "USD".to_string(),
                    capabilities: vec![
                        crate::core::types::common::ProviderCapability::ChatCompletion,
                        crate::core::types::common::ProviderCapability::ChatCompletionStream,
                    ],
                    created_at: None,
                    updated_at: None,
                    metadata: std::collections::HashMap::new(),
                },
                family: AnthropicModelFamily::Claude21,
                features: vec![ModelFeature::StreamingSupport, ModelFeature::SystemMessages],
                pricing: ModelPricing {
                    input_price: 8.0,
                    output_price: 24.0,
                    cache_write_price: None,
                    cache_read_price: None,
                    batch_discount: None,
                },
                limits: ModelLimits {
                    max_context_length: 200_000,
                    max_output_tokens: 4_096,
                    max_images: None,
                    max_document_size_mb: None,
                },
                config: ModelConfig::default(),
            },
        );

        // Claude Instant
        self.register_model(
            "claude-instant-1.2",
            ModelSpec {
                model_info: ModelInfo {
                    id: "claude-instant-1.2".to_string(),
                    name: "Claude Instant 1.2".to_string(),
                    provider: "anthropic".to_string(),
                    max_context_length: 100_000,
                    max_output_length: Some(4_096),
                    supports_streaming: true,
                    supports_tools: false,
                    supports_multimodal: false,
                    input_cost_per_1k_tokens: Some(0.0008),
                    output_cost_per_1k_tokens: Some(0.0024),
                    currency: "USD".to_string(),
                    capabilities: vec![
                        crate::core::types::common::ProviderCapability::ChatCompletion,
                        crate::core::types::common::ProviderCapability::ChatCompletionStream,
                    ],
                    created_at: None,
                    updated_at: None,
                    metadata: std::collections::HashMap::new(),
                },
                family: AnthropicModelFamily::ClaudeInstant,
                features: vec![ModelFeature::StreamingSupport, ModelFeature::SystemMessages],
                pricing: ModelPricing {
                    input_price: 0.80,
                    output_price: 2.40,
                    cache_write_price: None,
                    cache_read_price: None,
                    batch_discount: None,
                },
                limits: ModelLimits {
                    max_context_length: 100_000,
                    max_output_tokens: 4_096,
                    max_images: None,
                    max_document_size_mb: None,
                },
                config: ModelConfig::default(),
            },
        );
    }

    /// Register a model
    fn register_model(&mut self, id: &str, spec: ModelSpec) {
        self.models.insert(id.to_string(), spec);
    }

    /// Get model specification
    pub fn get_model_spec(&self, model_id: &str) -> Option<&ModelSpec> {
        self.models.get(model_id)
    }

    /// List all models
    pub fn list_models(&self) -> Vec<&ModelSpec> {
        self.models.values().collect()
    }

    /// Check if model supports feature
    pub fn supports_feature(&self, model_id: &str, feature: &ModelFeature) -> bool {
        self.get_model_spec(model_id)
            .map(|spec| spec.features.contains(feature))
            .unwrap_or(false)
    }

    /// Get model family
    pub fn get_model_family(&self, model_id: &str) -> Option<&AnthropicModelFamily> {
        self.get_model_spec(model_id).map(|spec| &spec.family)
    }

    /// Get model pricing
    pub fn get_model_pricing(&self, model_id: &str) -> Option<&ModelPricing> {
        self.get_model_spec(model_id).map(|spec| &spec.pricing)
    }

    /// Get model limits
    pub fn get_model_limits(&self, model_id: &str) -> Option<&ModelLimits> {
        self.get_model_spec(model_id).map(|spec| &spec.limits)
    }

    /// Get model family from name
    pub fn from_model_name(model_name: &str) -> Option<AnthropicModelFamily> {
        let model_lower = model_name.to_lowercase();

        if model_lower.contains("claude-3-5-sonnet") || model_lower.contains("claude-3.5-sonnet") {
            Some(AnthropicModelFamily::Claude35Sonnet)
        } else if model_lower.contains("claude-3-opus") {
            Some(AnthropicModelFamily::Claude3Opus)
        } else if model_lower.contains("claude-3-sonnet") {
            Some(AnthropicModelFamily::Claude3Sonnet)
        } else if model_lower.contains("claude-3-haiku") {
            Some(AnthropicModelFamily::Claude3Haiku)
        } else if model_lower.contains("claude-2.1") {
            Some(AnthropicModelFamily::Claude21)
        } else if model_lower.contains("claude-2") {
            Some(AnthropicModelFamily::Claude2)
        } else if model_lower.contains("claude-instant") {
            Some(AnthropicModelFamily::ClaudeInstant)
        } else {
            None
        }
    }
}

impl Default for AnthropicModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Get global model registry
pub fn get_anthropic_registry() -> &'static AnthropicModelRegistry {
    static REGISTRY: OnceLock<AnthropicModelRegistry> = OnceLock::new();
    REGISTRY.get_or_init(AnthropicModelRegistry::new)
}

/// Cost calculation utility
pub struct CostCalculator;

impl CostCalculator {
    /// Calculate basic cost
    pub fn calculate_cost(
        model_id: &str,
        prompt_tokens: u32,
        completion_tokens: u32,
    ) -> Option<f64> {
        let registry = get_anthropic_registry();
        let pricing = registry.get_model_pricing(model_id)?;

        let input_cost = (prompt_tokens as f64 / 1_000_000.0) * pricing.input_price;
        let output_cost = (completion_tokens as f64 / 1_000_000.0) * pricing.output_price;

        Some(input_cost + output_cost)
    }

    /// Calculate extended cost (including cache)
    pub fn calculate_extended_cost(
        model_id: &str,
        prompt_tokens: u32,
        completion_tokens: u32,
        cache_read_tokens: Option<u32>,
        cache_write_tokens: Option<u32>,
        is_batch: bool,
    ) -> Option<f64> {
        let registry = get_anthropic_registry();
        let pricing = registry.get_model_pricing(model_id)?;

        let batch_multiplier = if is_batch {
            pricing.batch_discount.unwrap_or(1.0)
        } else {
            1.0
        };

        let mut total_cost = 0.0;
        let mut remaining_prompt_tokens = prompt_tokens;

        // Handle cache read tokens
        if let (Some(cache_read), Some(cache_read_price)) =
            (cache_read_tokens, pricing.cache_read_price)
        {
            let cache_cost = (cache_read as f64 / 1_000_000.0) * cache_read_price;
            total_cost += cache_cost;
            remaining_prompt_tokens = remaining_prompt_tokens.saturating_sub(cache_read);
        }

        // Handle cache write tokens
        if let (Some(cache_write), Some(cache_write_price)) =
            (cache_write_tokens, pricing.cache_write_price)
        {
            let cache_write_cost =
                (cache_write as f64 / 1_000_000.0) * cache_write_price * batch_multiplier;
            total_cost += cache_write_cost;
            remaining_prompt_tokens = remaining_prompt_tokens.saturating_sub(cache_write);
        }

        // Regular input tokens
        let input_cost =
            (remaining_prompt_tokens as f64 / 1_000_000.0) * pricing.input_price * batch_multiplier;
        total_cost += input_cost;

        // Output tokens
        let output_cost =
            (completion_tokens as f64 / 1_000_000.0) * pricing.output_price * batch_multiplier;
        total_cost += output_cost;

        Some(total_cost)
    }

    /// Estimate token count
    pub fn estimate_tokens(text: &str) -> u32 {
        // Anthropic uses approximately 4 characters = 1 token ratio (English)
        (text.len() as f32 / 4.0).ceil() as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_registry() {
        let registry = get_anthropic_registry();

        // Test Claude 3.5 Sonnet
        let sonnet_spec = registry
            .get_model_spec("claude-3-5-sonnet-20241022")
            .unwrap();
        assert_eq!(sonnet_spec.family, AnthropicModelFamily::Claude35Sonnet);
        assert!(
            sonnet_spec
                .features
                .contains(&ModelFeature::MultimodalSupport)
        );
        assert!(sonnet_spec.features.contains(&ModelFeature::ComputerUse));

        // Test pricing
        assert_eq!(sonnet_spec.pricing.input_price, 3.0);
        assert_eq!(sonnet_spec.pricing.output_price, 15.0);
    }

    #[test]
    fn test_model_family_detection() {
        assert_eq!(
            AnthropicModelRegistry::from_model_name("claude-3-5-sonnet-20241022"),
            Some(AnthropicModelFamily::Claude35Sonnet)
        );

        assert_eq!(
            AnthropicModelRegistry::from_model_name("claude-3-opus-20240229"),
            Some(AnthropicModelFamily::Claude3Opus)
        );

        assert_eq!(
            AnthropicModelRegistry::from_model_name("unknown-model"),
            None
        );
    }

    #[test]
    fn test_cost_calculation() {
        let cost = CostCalculator::calculate_cost("claude-3-5-sonnet-20241022", 1000, 500);
        assert!(cost.is_some());

        let cost_value = cost.unwrap();
        // Expected: (1000/1M * $3) + (500/1M * $15) = $0.003 + $0.0075 = $0.0105
        assert!((cost_value - 0.0105).abs() < 0.0001);
    }

    #[test]
    fn test_feature_support() {
        let registry = get_anthropic_registry();

        // Claude 3.5 Sonnet supports computer tools
        assert!(
            registry.supports_feature("claude-3-5-sonnet-20241022", &ModelFeature::ComputerUse)
        );

        // Claude 2.1 does not support computer tools
        assert!(!registry.supports_feature("claude-2.1", &ModelFeature::ComputerUse));
    }
}
