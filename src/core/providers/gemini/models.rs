//! Gemini Model Registry
//!
//! 统一的模型注册表系统，包含所有Gemini模型的能力和定价信息

use std::collections::HashMap;
use std::sync::OnceLock;

use crate::core::types::common::ModelInfo;

/// Model
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ModelFeature {
    /// 多模态支持（图像、视频、音频）
    MultimodalSupport,
    /// tool_call支持
    ToolCalling,
    /// 函数call支持
    FunctionCalling,
    /// Response
    StreamingSupport,
    /// 上下文cache支持
    ContextCaching,
    /// 系统指令支持
    SystemInstructions,
    /// Handle
    BatchProcessing,
    /// JSON模式支持
    JsonMode,
    /// 代码执行支持
    CodeExecution,
    /// 搜索增强支持
    SearchGrounding,
    /// 视频理解支持
    VideoUnderstanding,
    /// 音频理解支持  
    AudioUnderstanding,
    /// 实时流式支持
    RealtimeStreaming,
}

/// Model
#[derive(Debug, Clone, PartialEq)]
pub enum GeminiModelFamily {
    /// Gemini 2.0 系列
    Gemini20Flash,
    Gemini20FlashThinking,
    
    /// Gemini 1.5 系列
    Gemini15Pro,
    Gemini15Flash,
    Gemini15Flash8B,
    
    /// Gemini 1.0 系列
    Gemini10Pro,
    Gemini10ProVision,
    
    /// Model
    GeminiExperimental,
}

/// Model
#[derive(Debug, Clone)]
pub struct ModelPricing {
    /// inputtoken价格（每百万token美元）
    pub input_price: f64,
    /// outputtoken价格（每百万token美元）
    pub output_price: f64,
    /// cacheinput价格（optional）
    pub cached_input_price: Option<f64>,
    /// 图像价格（每张）
    pub image_price: Option<f64>,
    /// 视频价格（每seconds）
    pub video_price_per_second: Option<f64>,
    /// 音频价格（每seconds）
    pub audio_price_per_second: Option<f64>,
}

/// Model
#[derive(Debug, Clone)]
pub struct ModelLimits {
    /// maximum上下文长度
    pub max_context_length: u32,
    /// maximumoutputtoken数
    pub max_output_tokens: u32,
    /// maximumimagecount
    pub max_images: Option<u32>,
    /// maximum视频长度（seconds）
    pub max_video_seconds: Option<u32>,
    /// maximum音频长度（seconds）
    pub max_audio_seconds: Option<u32>,
    /// Request
    pub rpm_limit: Option<u32>,
    /// Token limit per minute
    pub tpm_limit: Option<u32>,
}

/// Model
#[derive(Debug, Clone)]
pub struct ModelSpec {
    /// Model
    pub model_info: ModelInfo,
    /// Model
    pub family: GeminiModelFamily,
    /// 支持的特性
    pub features: Vec<ModelFeature>,
    /// 定价信息
    pub pricing: ModelPricing,
    /// 限制信息
    pub limits: ModelLimits,
}

/// Model
#[derive(Debug, Clone)]
pub struct GeminiModelRegistry {
    models: HashMap<String, ModelSpec>,
}

impl GeminiModelRegistry {
    /// Create
    pub fn new() -> Self {
        let mut registry = Self {
            models: HashMap::new(),
        };
        registry.initialize_models();
        registry
    }

    /// Model
    fn initialize_models(&mut self) {
        // Gemini 2.0 Flash
        self.register_model("gemini-2.0-flash-exp", ModelSpec {
            model_info: ModelInfo {
                id: "gemini-2.0-flash-exp".to_string(),
                name: "Gemini 2.0 Flash".to_string(),
                provider: "gemini".to_string(),
                max_context_length: 1_000_000,
                max_output_length: Some(8192),
                supports_streaming: true,
                supports_tools: true,
                supports_multimodal: true,
                input_cost_per_1k_tokens: Some(0.00001),
                output_cost_per_1k_tokens: Some(0.00004),
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
            family: GeminiModelFamily::Gemini20Flash,
            features: vec![
                ModelFeature::MultimodalSupport,
                ModelFeature::ToolCalling,
                ModelFeature::FunctionCalling,
                ModelFeature::StreamingSupport,
                ModelFeature::ContextCaching,
                ModelFeature::SystemInstructions,
                ModelFeature::BatchProcessing,
                ModelFeature::JsonMode,
                ModelFeature::CodeExecution,
                ModelFeature::SearchGrounding,
                ModelFeature::VideoUnderstanding,
                ModelFeature::AudioUnderstanding,
            ],
            pricing: ModelPricing {
                input_price: 0.01,  // $0.01 per 1M tokens
                output_price: 0.04,  // $0.04 per 1M tokens
                cached_input_price: Some(0.0025),
                image_price: Some(0.0001),
                video_price_per_second: Some(0.001),
                audio_price_per_second: Some(0.0001),
            },
            limits: ModelLimits {
                max_context_length: 1_000_000,
                max_output_tokens: 8192,
                max_images: Some(3000),
                max_video_seconds: Some(3600),
                max_audio_seconds: Some(9600),
                rpm_limit: Some(2000),
                tpm_limit: Some(4_000_000),
            },
        });

        // Gemini 2.0 Flash Thinking (实验性)
        self.register_model("gemini-2.0-flash-thinking-exp", ModelSpec {
            model_info: ModelInfo {
                id: "gemini-2.0-flash-thinking-exp".to_string(),
                name: "Gemini 2.0 Flash Thinking".to_string(),
                provider: "gemini".to_string(),
                max_context_length: 32_000,
                max_output_length: Some(8192),
                supports_streaming: true,
                supports_tools: true,
                supports_multimodal: true,
                input_cost_per_1k_tokens: Some(0.00001),
                output_cost_per_1k_tokens: Some(0.00004),
                currency: "USD".to_string(),
                capabilities: vec![
                    crate::core::types::common::ProviderCapability::ChatCompletion,
                    crate::core::types::common::ProviderCapability::ChatCompletionStream,
                ],
                created_at: None,
                updated_at: None,
                metadata: std::collections::HashMap::new(),
            },
            family: GeminiModelFamily::Gemini20FlashThinking,
            features: vec![
                ModelFeature::MultimodalSupport,
                ModelFeature::StreamingSupport,
                ModelFeature::SystemInstructions,
            ],
            pricing: ModelPricing {
                input_price: 0.01,
                output_price: 0.04,
                cached_input_price: None,
                image_price: Some(0.0001),
                video_price_per_second: None,
                audio_price_per_second: None,
            },
            limits: ModelLimits {
                max_context_length: 32_000,
                max_output_tokens: 8192,
                max_images: Some(50),
                max_video_seconds: None,
                max_audio_seconds: None,
                rpm_limit: Some(100),
                tpm_limit: Some(100_000),
            },
        });

        // Gemini 1.5 Pro
        self.register_model("gemini-1.5-pro", ModelSpec {
            model_info: ModelInfo {
                id: "gemini-1.5-pro".to_string(),
                name: "Gemini 1.5 Pro".to_string(),
                provider: "gemini".to_string(),
                max_context_length: 2_000_000,
                max_output_length: Some(8192),
                supports_streaming: true,
                supports_tools: true,
                supports_multimodal: true,
                input_cost_per_1k_tokens: Some(0.00125),
                output_cost_per_1k_tokens: Some(0.005),
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
            family: GeminiModelFamily::Gemini15Pro,
            features: vec![
                ModelFeature::MultimodalSupport,
                ModelFeature::ToolCalling,
                ModelFeature::FunctionCalling,
                ModelFeature::StreamingSupport,
                ModelFeature::ContextCaching,
                ModelFeature::SystemInstructions,
                ModelFeature::BatchProcessing,
                ModelFeature::JsonMode,
                ModelFeature::CodeExecution,
                ModelFeature::SearchGrounding,
                ModelFeature::VideoUnderstanding,
                ModelFeature::AudioUnderstanding,
            ],
            pricing: ModelPricing {
                input_price: 1.25,  // $1.25 per 1M tokens (<=128K)
                output_price: 5.0,   // $5.00 per 1M tokens (<=128K)
                cached_input_price: Some(0.3125),
                image_price: Some(0.002625),
                video_price_per_second: Some(0.002625),
                audio_price_per_second: Some(0.000125),
            },
            limits: ModelLimits {
                max_context_length: 2_000_000,
                max_output_tokens: 8192,
                max_images: Some(3000),
                max_video_seconds: Some(3600),
                max_audio_seconds: Some(9600),
                rpm_limit: Some(360),
                tpm_limit: Some(4_000_000),
            },
        });

        // Gemini 1.5 Flash
        self.register_model("gemini-1.5-flash", ModelSpec {
            model_info: ModelInfo {
                id: "gemini-1.5-flash".to_string(),
                name: "Gemini 1.5 Flash".to_string(),
                provider: "gemini".to_string(),
                max_context_length: 1_000_000,
                max_output_length: Some(8192),
                supports_streaming: true,
                supports_tools: true,
                supports_multimodal: true,
                input_cost_per_1k_tokens: Some(0.000075),
                output_cost_per_1k_tokens: Some(0.0003),
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
            family: GeminiModelFamily::Gemini15Flash,
            features: vec![
                ModelFeature::MultimodalSupport,
                ModelFeature::ToolCalling,
                ModelFeature::FunctionCalling,
                ModelFeature::StreamingSupport,
                ModelFeature::ContextCaching,
                ModelFeature::SystemInstructions,
                ModelFeature::BatchProcessing,
                ModelFeature::JsonMode,
                ModelFeature::CodeExecution,
                ModelFeature::SearchGrounding,
                ModelFeature::VideoUnderstanding,
                ModelFeature::AudioUnderstanding,
            ],
            pricing: ModelPricing {
                input_price: 0.075,  // $0.075 per 1M tokens (<=128K)
                output_price: 0.30,   // $0.30 per 1M tokens (<=128K)
                cached_input_price: Some(0.01875),
                image_price: Some(0.0002),
                video_price_per_second: Some(0.0002),
                audio_price_per_second: Some(0.0001),
            },
            limits: ModelLimits {
                max_context_length: 1_000_000,
                max_output_tokens: 8192,
                max_images: Some(3000),
                max_video_seconds: Some(3600),
                max_audio_seconds: Some(9600),
                rpm_limit: Some(1500),
                tpm_limit: Some(4_000_000),
            },
        });

        // Gemini 1.5 Flash-8B
        self.register_model("gemini-1.5-flash-8b", ModelSpec {
            model_info: ModelInfo {
                id: "gemini-1.5-flash-8b".to_string(),
                name: "Gemini 1.5 Flash 8B".to_string(),
                provider: "gemini".to_string(),
                max_context_length: 1_000_000,
                max_output_length: Some(8192),
                supports_streaming: true,
                supports_tools: true,
                supports_multimodal: true,
                input_cost_per_1k_tokens: Some(0.0000375),
                output_cost_per_1k_tokens: Some(0.00015),
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
            family: GeminiModelFamily::Gemini15Flash8B,
            features: vec![
                ModelFeature::MultimodalSupport,
                ModelFeature::ToolCalling,
                ModelFeature::FunctionCalling,
                ModelFeature::StreamingSupport,
                ModelFeature::ContextCaching,
                ModelFeature::SystemInstructions,
                ModelFeature::BatchProcessing,
                ModelFeature::JsonMode,
                ModelFeature::VideoUnderstanding,
                ModelFeature::AudioUnderstanding,
            ],
            pricing: ModelPricing {
                input_price: 0.0375,  // $0.0375 per 1M tokens
                output_price: 0.15,    // $0.15 per 1M tokens
                cached_input_price: Some(0.01),
                image_price: Some(0.0001),
                video_price_per_second: Some(0.0001),
                audio_price_per_second: Some(0.00005),
            },
            limits: ModelLimits {
                max_context_length: 1_000_000,
                max_output_tokens: 8192,
                max_images: Some(3000),
                max_video_seconds: Some(3600),
                max_audio_seconds: Some(9600),
                rpm_limit: Some(4000),
                tpm_limit: Some(4_000_000),
            },
        });

        // Gemini 1.0 Pro
        self.register_model("gemini-1.0-pro", ModelSpec {
            model_info: ModelInfo {
                id: "gemini-1.0-pro".to_string(),
                name: "Gemini 1.0 Pro".to_string(),
                provider: "gemini".to_string(),
                max_context_length: 32_000,
                max_output_length: Some(8192),
                supports_streaming: true,
                supports_tools: true,
                supports_multimodal: false,
                input_cost_per_1k_tokens: Some(0.0005),
                output_cost_per_1k_tokens: Some(0.0015),
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
            family: GeminiModelFamily::Gemini10Pro,
            features: vec![
                ModelFeature::ToolCalling,
                ModelFeature::FunctionCalling,
                ModelFeature::StreamingSupport,
                ModelFeature::SystemInstructions,
                ModelFeature::BatchProcessing,
            ],
            pricing: ModelPricing {
                input_price: 0.50,   // $0.50 per 1M tokens
                output_price: 1.50,  // $1.50 per 1M tokens
                cached_input_price: None,
                image_price: None,
                video_price_per_second: None,
                audio_price_per_second: None,
            },
            limits: ModelLimits {
                max_context_length: 32_000,
                max_output_tokens: 8192,
                max_images: None,
                max_video_seconds: None,
                max_audio_seconds: None,
                rpm_limit: Some(300),
                tpm_limit: Some(300_000),
            },
        });
    }

    /// Model
    fn register_model(&mut self, id: &str, spec: ModelSpec) {
        self.models.insert(id.to_string(), spec);
    }

    /// Model
    pub fn get_model_spec(&self, model_id: &str) -> Option<&ModelSpec> {
        self.models.get(model_id)
    }

    /// Model
    pub fn list_models(&self) -> Vec<&ModelSpec> {
        self.models.values().collect()
    }

    /// Check
    pub fn supports_feature(&self, model_id: &str, feature: &ModelFeature) -> bool {
        self.get_model_spec(model_id)
            .map(|spec| spec.features.contains(feature))
            .unwrap_or(false)
    }

    /// Model
    pub fn get_model_family(&self, model_id: &str) -> Option<&GeminiModelFamily> {
        self.get_model_spec(model_id).map(|spec| &spec.family)
    }

    /// Model
    pub fn get_model_pricing(&self, model_id: &str) -> Option<&ModelPricing> {
        self.get_model_spec(model_id).map(|spec| &spec.pricing)
    }

    /// Model
    pub fn get_model_limits(&self, model_id: &str) -> Option<&ModelLimits> {
        self.get_model_spec(model_id).map(|spec| &spec.limits)
    }

    /// Model
    pub fn from_model_name(model_name: &str) -> Option<GeminiModelFamily> {
        let model_lower = model_name.to_lowercase();
        
        if model_lower.contains("gemini-2.0-flash-thinking") {
            Some(GeminiModelFamily::Gemini20FlashThinking)
        } else if model_lower.contains("gemini-2.0-flash") || model_lower.contains("gemini-2-flash") {
            Some(GeminiModelFamily::Gemini20Flash)
        } else if model_lower.contains("gemini-1.5-pro") || model_lower.contains("gemini-15-pro") {
            Some(GeminiModelFamily::Gemini15Pro)
        } else if model_lower.contains("gemini-1.5-flash-8b") {
            Some(GeminiModelFamily::Gemini15Flash8B)
        } else if model_lower.contains("gemini-1.5-flash") || model_lower.contains("gemini-15-flash") {
            Some(GeminiModelFamily::Gemini15Flash)
        } else if model_lower.contains("gemini-1.0-pro-vision") {
            Some(GeminiModelFamily::Gemini10ProVision)
        } else if model_lower.contains("gemini-1.0-pro") || model_lower.contains("gemini-pro") {
            Some(GeminiModelFamily::Gemini10Pro)
        } else if model_lower.contains("gemini-exp") {
            Some(GeminiModelFamily::GeminiExperimental)
        } else {
            None
        }
    }
}

impl Default for GeminiModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Model
pub fn get_gemini_registry() -> &'static GeminiModelRegistry {
    static REGISTRY: OnceLock<GeminiModelRegistry> = OnceLock::new();
    REGISTRY.get_or_init(GeminiModelRegistry::new)
}

/// 成本计算工具
pub struct CostCalculator;

impl CostCalculator {
    /// 计算基础成本
    pub fn calculate_cost(
        model_id: &str,
        prompt_tokens: u32,
        completion_tokens: u32,
    ) -> Option<f64> {
        let registry = get_gemini_registry();
        let pricing = registry.get_model_pricing(model_id)?;
        
        let input_cost = (prompt_tokens as f64 / 1_000_000.0) * pricing.input_price;
        let output_cost = (completion_tokens as f64 / 1_000_000.0) * pricing.output_price;
        
        Some(input_cost + output_cost)
    }

    /// 计算多模态成本
    pub fn calculate_multimodal_cost(
        model_id: &str,
        prompt_tokens: u32,
        completion_tokens: u32,
        cached_tokens: Option<u32>,
        images: Option<u32>,
        video_seconds: Option<u32>,
        audio_seconds: Option<u32>,
    ) -> Option<f64> {
        let registry = get_gemini_registry();
        let pricing = registry.get_model_pricing(model_id)?;
        
        let mut total_cost = 0.0;
        let mut remaining_prompt_tokens = prompt_tokens;
        
        // Handle
        if let (Some(cached), Some(cached_price)) = (cached_tokens, pricing.cached_input_price) {
            let cached_cost = (cached as f64 / 1_000_000.0) * cached_price;
            total_cost += cached_cost;
            remaining_prompt_tokens = remaining_prompt_tokens.saturating_sub(cached);
        }
        
        // 普通inputtoken
        let input_cost = (remaining_prompt_tokens as f64 / 1_000_000.0) * pricing.input_price;
        total_cost += input_cost;
        
        // outputtoken
        let output_cost = (completion_tokens as f64 / 1_000_000.0) * pricing.output_price;
        total_cost += output_cost;
        
        // 图像成本
        if let (Some(img_count), Some(img_price)) = (images, pricing.image_price) {
            total_cost += img_count as f64 * img_price;
        }
        
        // 视频成本
        if let (Some(video_secs), Some(video_price)) = (video_seconds, pricing.video_price_per_second) {
            total_cost += video_secs as f64 * video_price;
        }
        
        // 音频成本
        if let (Some(audio_secs), Some(audio_price)) = (audio_seconds, pricing.audio_price_per_second) {
            total_cost += audio_secs as f64 * audio_price;
        }
        
        Some(total_cost)
    }

    /// 估算tokencount
    pub fn estimate_tokens(text: &str) -> u32 {
        // Geminiusage约4个字符=1个token的比例（英文）
        (text.len() as f32 / 4.0).ceil() as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_registry() {
        let registry = get_gemini_registry();
        
        // 测试Gemini 2.0 Flash
        let flash_spec = registry.get_model_spec("gemini-2.0-flash-exp").unwrap();
        assert_eq!(flash_spec.family, GeminiModelFamily::Gemini20Flash);
        assert!(flash_spec.features.contains(&ModelFeature::MultimodalSupport));
        assert!(flash_spec.features.contains(&ModelFeature::VideoUnderstanding));
        
        // 测试定价
        assert_eq!(flash_spec.pricing.input_price, 0.01);
        assert_eq!(flash_spec.pricing.output_price, 0.04);
    }

    #[test]
    fn test_model_family_detection() {
        assert_eq!(
            GeminiModelRegistry::from_model_name("gemini-2.0-flash-exp"),
            Some(GeminiModelFamily::Gemini20Flash)
        );
        
        assert_eq!(
            GeminiModelRegistry::from_model_name("gemini-1.5-pro-latest"),
            Some(GeminiModelFamily::Gemini15Pro)
        );
        
        assert_eq!(
            GeminiModelRegistry::from_model_name("unknown-model"),
            None
        );
    }

    #[test]
    fn test_cost_calculation() {
        let cost = CostCalculator::calculate_cost("gemini-1.5-flash", 1000, 500);
        assert!(cost.is_some());
        
        let cost_value = cost.unwrap();
        // 预期: (1000/1M * $0.075) + (500/1M * $0.30) = $0.000075 + $0.00015 = $0.000225
        assert!((cost_value - 0.000225).abs() < 0.000001);
    }

    #[test]
    fn test_feature_support() {
        let registry = get_gemini_registry();
        
        // Gemini 2.0 Flash支持视频理解
        assert!(registry.supports_feature("gemini-2.0-flash-exp", &ModelFeature::VideoUnderstanding));
        
        // Gemini 1.0 Pro不支持多模态
        assert!(!registry.supports_feature("gemini-1.0-pro", &ModelFeature::VideoUnderstanding));
    }
}