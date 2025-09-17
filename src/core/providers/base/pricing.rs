//! 统一的价格计算系统
//!
//! 与Pythonversion共享model_prices_and_context_window.json数据

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tracing::warn;

/// Model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPricing {
    /// 每个inputtoken的成本
    #[serde(default)]
    pub input_cost_per_token: f64,

    /// 每个outputtoken的成本
    #[serde(default)]
    pub output_cost_per_token: f64,

    /// Model
    #[serde(default)]
    pub output_cost_per_reasoning_token: f64,

    /// maximumtoken数（兼容旧字段）
    #[serde(default)]
    pub max_tokens: Option<u32>,

    /// maximuminputtoken数
    #[serde(default)]
    pub max_input_tokens: Option<u32>,

    /// maximumoutputtoken数
    #[serde(default)]
    pub max_output_tokens: Option<u32>,

    /// Provider名称
    #[serde(default)]
    pub litellm_provider: Option<String>,

    /// 模式（chat, embedding, completionetc）
    #[serde(default)]
    pub mode: Option<String>,

    /// 是否支持函数call
    #[serde(default)]
    pub supports_function_calling: Option<bool>,

    /// 是否支持视觉
    #[serde(default)]
    pub supports_vision: Option<bool>,
}

/// usage信息
#[derive(Debug, Clone)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
    pub reasoning_tokens: Option<u32>,
}

/// 价格数据库
#[derive(Debug, Clone)]
pub struct PricingDatabase {
    models: HashMap<String, ModelPricing>,
}

impl PricingDatabase {
    /// 从JSON文件加载价格数据
    pub fn from_json_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content =
            fs::read_to_string(path).map_err(|e| format!("Failed to read pricing file: {}", e))?;

        let all_data: HashMap<String, serde_json::Value> = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse pricing JSON: {}", e))?;

        // 过滤掉不是实际模型的条目（如 sample_spec）
        let mut models = HashMap::new();
        for (key, value) in all_data {
            // 跳过文档和示例条目
            if key == "sample_spec" || key.starts_with("_") || key.contains("example") {
                continue;
            }
            
            // 尝试将值parse为ModelPricing
            match serde_json::from_value::<ModelPricing>(value) {
                Ok(pricing) => {
                    models.insert(key, pricing);
                }
                Err(e) => {
                    warn!(
                        model = %key,
                        error = %e,
                        "Failed to parse model pricing data, skipping model"
                    );
                }
            }
        }

        Ok(Self { models })
    }

    /// 从Python的JSON文件加载（自动查找）
    pub fn from_python_json() -> Result<Self, String> {
        // 尝试多个可能的路径
        let possible_paths = vec![
            "model_prices_and_context_window.json",
            "../model_prices_and_context_window.json",
            "../../model_prices_and_context_window.json",
            "../../../model_prices_and_context_window.json",
            "/Users/vibercoder/Desktop/code/Work/Common/Lib/litellm/litellm/model_prices_and_context_window.json",
        ];

        for path in &possible_paths {
            if Path::new(path).exists() {
                return Self::from_json_file(path);
            }
        }

        // Default
        Ok(Self::default())
    }

    /// 计算成本
    pub fn calculate(&self, model: &str, usage: &Usage) -> f64 {
        // 直接查找模型
        if let Some(pricing) = self.models.get(model) {
            return self.calculate_with_pricing(pricing, usage);
        }

        // Handle
        for (key, pricing) in &self.models {
            if model.contains(key) || key.contains(model) {
                return self.calculate_with_pricing(pricing, usage);
            }
        }

        // 找不到价格信息
        0.0
    }

    /// usage指定的价格信息计算成本
    fn calculate_with_pricing(&self, pricing: &ModelPricing, usage: &Usage) -> f64 {
        let mut cost = 0.0;

        // inputtoken成本
        cost += usage.prompt_tokens as f64 * pricing.input_cost_per_token;

        // outputtoken成本
        cost += usage.completion_tokens as f64 * pricing.output_cost_per_token;

        // 推理token成本（如果有）
        if let Some(reasoning_tokens) = usage.reasoning_tokens {
            cost += reasoning_tokens as f64 * pricing.output_cost_per_reasoning_token;
        }

        cost
    }

    /// Model
    pub fn get_model_info(&self, model: &str) -> Option<&ModelPricing> {
        self.models.get(model)
    }

    /// Model
    pub fn get_max_tokens(&self, model: &str) -> Option<u32> {
        self.get_model_info(model).and_then(|info| {
            info.max_tokens
                .or(info.max_input_tokens)
                .or(info.max_output_tokens)
        })
    }

    /// Model
    pub fn get_provider_models(&self, provider: &str) -> Vec<String> {
        self.models
            .iter()
            .filter_map(|(model_id, pricing)| {
                if let Some(ref provider_name) = pricing.litellm_provider {
                    if provider_name.to_lowercase() == provider.to_lowercase() {
                        Some(model_id.clone())
                    } else {
                        None
                    }
                } else if model_id.to_lowercase().contains(&provider.to_lowercase()) {
                    // 如果没有显式的provider字段，通过模型名称推断
                    Some(model_id.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Create
    pub fn to_model_info(
        &self,
        model_id: &str,
        provider: &str,
    ) -> Option<crate::core::types::common::ModelInfo> {
        use crate::core::types::common::ModelInfo;
        use std::collections::HashMap;

        let pricing = self.get_model_info(model_id)?;

        Some(ModelInfo {
            id: model_id.to_string(),
            name: model_id.replace(['-', '_'], " "), // 简单的名称转换
            provider: provider.to_string(),
            max_context_length: pricing
                .max_input_tokens
                .unwrap_or_else(|| pricing.max_tokens.unwrap_or(4096)),
            max_output_length: pricing.max_output_tokens,
            supports_streaming: true, // 大多数现代模型都支持流式
            supports_tools: pricing.supports_function_calling.unwrap_or(false),
            supports_multimodal: pricing.supports_vision.unwrap_or(false),
            input_cost_per_1k_tokens: Some(pricing.input_cost_per_token * 1000.0),
            output_cost_per_1k_tokens: Some(pricing.output_cost_per_token * 1000.0),
            currency: "USD".to_string(),
            capabilities: vec![], // 可以后续扩展
            created_at: None,
            updated_at: None,
            metadata: HashMap::new(),
        })
    }

    /// Check
    pub fn supports_feature(&self, model: &str, feature: &str) -> bool {
        self.get_model_info(model)
            .map(|info| match feature {
                "function_calling" => info.supports_function_calling.unwrap_or(false),
                "vision" => info.supports_vision.unwrap_or(false),
                _ => false,
            })
            .unwrap_or(false)
    }
}

impl Default for PricingDatabase {
    fn default() -> Self {
        // 内置一些常用模型的价格作为备用
        let mut models = HashMap::new();

        // OpenAI models
        models.insert(
            "gpt-4".to_string(),
            ModelPricing {
                input_cost_per_token: 0.00003,
                output_cost_per_token: 0.00006,
                output_cost_per_reasoning_token: 0.0,
                max_tokens: Some(8192),
                max_input_tokens: Some(8192),
                max_output_tokens: Some(4096),
                litellm_provider: Some("openai".to_string()),
                mode: Some("chat".to_string()),
                supports_function_calling: Some(true),
                supports_vision: Some(false),
            },
        );

        models.insert(
            "gpt-4-turbo".to_string(),
            ModelPricing {
                input_cost_per_token: 0.00001,
                output_cost_per_token: 0.00003,
                output_cost_per_reasoning_token: 0.0,
                max_tokens: Some(128000),
                max_input_tokens: Some(128000),
                max_output_tokens: Some(4096),
                litellm_provider: Some("openai".to_string()),
                mode: Some("chat".to_string()),
                supports_function_calling: Some(true),
                supports_vision: Some(true),
            },
        );

        models.insert(
            "gpt-3.5-turbo".to_string(),
            ModelPricing {
                input_cost_per_token: 0.0000005,
                output_cost_per_token: 0.0000015,
                output_cost_per_reasoning_token: 0.0,
                max_tokens: Some(16385),
                max_input_tokens: Some(16385),
                max_output_tokens: Some(4096),
                litellm_provider: Some("openai".to_string()),
                mode: Some("chat".to_string()),
                supports_function_calling: Some(true),
                supports_vision: Some(false),
            },
        );

        // Anthropic models
        models.insert(
            "claude-3-opus".to_string(),
            ModelPricing {
                input_cost_per_token: 0.000015,
                output_cost_per_token: 0.000075,
                output_cost_per_reasoning_token: 0.0,
                max_tokens: Some(200000),
                max_input_tokens: Some(200000),
                max_output_tokens: Some(4096),
                litellm_provider: Some("anthropic".to_string()),
                mode: Some("chat".to_string()),
                supports_function_calling: Some(true),
                supports_vision: Some(true),
            },
        );

        models.insert(
            "claude-3-sonnet".to_string(),
            ModelPricing {
                input_cost_per_token: 0.000003,
                output_cost_per_token: 0.000015,
                output_cost_per_reasoning_token: 0.0,
                max_tokens: Some(200000),
                max_input_tokens: Some(200000),
                max_output_tokens: Some(4096),
                litellm_provider: Some("anthropic".to_string()),
                mode: Some("chat".to_string()),
                supports_function_calling: Some(true),
                supports_vision: Some(true),
            },
        );

        // DeepSeek models - updated pricing
        models.insert(
            "deepseek-chat".to_string(),
            ModelPricing {
                input_cost_per_token: 0.00000056,  // $0.56 per 1M tokens
                output_cost_per_token: 0.00000168, // $1.68 per 1M tokens
                output_cost_per_reasoning_token: 0.0,
                max_tokens: Some(128000),
                max_input_tokens: Some(128000),
                max_output_tokens: Some(8192),
                litellm_provider: Some("deepseek".to_string()),
                mode: Some("chat".to_string()),
                supports_function_calling: Some(true),
                supports_vision: Some(false),
            },
        );

        models.insert(
            "deepseek-reasoner".to_string(),
            ModelPricing {
                input_cost_per_token: 0.00000056,  // $0.56 per 1M tokens
                output_cost_per_token: 0.00000168, // $1.68 per 1M tokens
                output_cost_per_reasoning_token: 0.0,
                max_tokens: Some(128000),
                max_input_tokens: Some(128000),
                max_output_tokens: Some(8192),
                litellm_provider: Some("deepseek".to_string()),
                mode: Some("chat".to_string()),
                supports_function_calling: Some(true),
                supports_vision: Some(false),
            },
        );

        Self { models }
    }
}

// 全局价格数据库（懒加载）
pub static GLOBAL_PRICING_DB: Lazy<PricingDatabase> = Lazy::new(|| {
    PricingDatabase::from_python_json().unwrap_or_else(|e| {
        warn!(
            error = %e,
            "Failed to load pricing data from file, using built-in defaults"
        );
        PricingDatabase::default()
    })
});

/// Get
pub fn get_pricing_db() -> &'static PricingDatabase {
    &GLOBAL_PRICING_DB
}

/// 快速计算成本
pub fn calculate_cost(model: &str, prompt_tokens: u32, completion_tokens: u32) -> f64 {
    let usage = Usage {
        prompt_tokens,
        completion_tokens,
        total_tokens: prompt_tokens + completion_tokens,
        reasoning_tokens: None,
    };

    GLOBAL_PRICING_DB.calculate(model, &usage)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_pricing() {
        let db = PricingDatabase::default();

        let usage = Usage {
            prompt_tokens: 1000,
            completion_tokens: 500,
            total_tokens: 1500,
            reasoning_tokens: None,
        };

        // Test GPT-4 pricing
        let cost = db.calculate("gpt-4", &usage);
        assert!(cost > 0.0);
        assert_eq!(cost, 1000.0 * 0.00003 + 500.0 * 0.00006);

        // Test Claude pricing
        let cost = db.calculate("claude-3-opus", &usage);
        assert!(cost > 0.0);
    }

    #[test]
    fn test_model_info() {
        let db = PricingDatabase::default();

        assert!(db.get_model_info("gpt-4").is_some());
        assert!(db.get_model_info("non-existent-model").is_none());

        assert_eq!(db.get_max_tokens("gpt-4"), Some(8192));
        assert!(db.supports_feature("gpt-4", "function_calling"));
        assert!(!db.supports_feature("gpt-4", "vision"));
        assert!(db.supports_feature("gpt-4-turbo", "vision"));
    }

    #[test]
    fn test_quick_calculate() {
        let cost = calculate_cost("gpt-3.5-turbo", 1000, 500);
        assert!(cost > 0.0);
    }
}
