//! Cost Calculation for Bedrock Models
//!
//! Provides accurate pricing information and cost calculation
//! for all supported Bedrock models.

use std::collections::HashMap;
use std::sync::LazyLock;

/// Model pricing information
#[derive(Debug, Clone)]
pub struct ModelPricing {
    pub input_cost_per_1k: f64,
    pub output_cost_per_1k: f64,
    pub currency: &'static str,
}

/// Comprehensive pricing database for all Bedrock models
static MODEL_PRICING: LazyLock<HashMap<&'static str, ModelPricing>> = LazyLock::new(|| {
        let mut pricing = HashMap::new();
        
        // Claude models
        pricing.insert("anthropic.claude-3-opus-20240229", ModelPricing {
            input_cost_per_1k: 0.015,
            output_cost_per_1k: 0.075,
            currency: "USD",
        });
        pricing.insert("anthropic.claude-3-sonnet-20240229", ModelPricing {
            input_cost_per_1k: 0.003,
            output_cost_per_1k: 0.015,
            currency: "USD",
        });
        pricing.insert("anthropic.claude-3-haiku-20240307", ModelPricing {
            input_cost_per_1k: 0.00025,
            output_cost_per_1k: 0.00125,
            currency: "USD",
        });
        pricing.insert("anthropic.claude-3-5-sonnet-20241022", ModelPricing {
            input_cost_per_1k: 0.003,
            output_cost_per_1k: 0.015,
            currency: "USD",
        });
        pricing.insert("anthropic.claude-3-5-haiku-20241022", ModelPricing {
            input_cost_per_1k: 0.001,
            output_cost_per_1k: 0.005,
            currency: "USD",
        });
        pricing.insert("anthropic.claude-v2:1", ModelPricing {
            input_cost_per_1k: 0.008,
            output_cost_per_1k: 0.024,
            currency: "USD",
        });
        pricing.insert("anthropic.claude-v2", ModelPricing {
            input_cost_per_1k: 0.008,
            output_cost_per_1k: 0.024,
            currency: "USD",
        });
        pricing.insert("anthropic.claude-instant-v1", ModelPricing {
            input_cost_per_1k: 0.00163,
            output_cost_per_1k: 0.00551,
            currency: "USD",
        });
        
        // Titan models
        pricing.insert("amazon.titan-text-express-v1", ModelPricing {
            input_cost_per_1k: 0.0002,
            output_cost_per_1k: 0.0006,
            currency: "USD",
        });
        pricing.insert("amazon.titan-text-lite-v1", ModelPricing {
            input_cost_per_1k: 0.00015,
            output_cost_per_1k: 0.0002,
            currency: "USD",
        });
        pricing.insert("amazon.titan-text-premier-v1:0", ModelPricing {
            input_cost_per_1k: 0.0005,
            output_cost_per_1k: 0.0015,
            currency: "USD",
        });
        
        // Nova models
        pricing.insert("amazon.nova-micro-v1:0", ModelPricing {
            input_cost_per_1k: 0.000035,
            output_cost_per_1k: 0.00014,
            currency: "USD",
        });
        pricing.insert("amazon.nova-lite-v1:0", ModelPricing {
            input_cost_per_1k: 0.00006,
            output_cost_per_1k: 0.00024,
            currency: "USD",
        });
        pricing.insert("amazon.nova-pro-v1:0", ModelPricing {
            input_cost_per_1k: 0.0008,
            output_cost_per_1k: 0.0032,
            currency: "USD",
        });
        
        // AI21 models
        pricing.insert("ai21.jamba-1-5-large-v1:0", ModelPricing {
            input_cost_per_1k: 0.002,
            output_cost_per_1k: 0.008,
            currency: "USD",
        });
        pricing.insert("ai21.jamba-1-5-mini-v1:0", ModelPricing {
            input_cost_per_1k: 0.0002,
            output_cost_per_1k: 0.0004,
            currency: "USD",
        });
        pricing.insert("ai21.jamba-instruct-v1:0", ModelPricing {
            input_cost_per_1k: 0.0005,
            output_cost_per_1k: 0.0007,
            currency: "USD",
        });
        
        // Cohere models
        pricing.insert("cohere.command-r-plus-v1:0", ModelPricing {
            input_cost_per_1k: 0.003,
            output_cost_per_1k: 0.015,
            currency: "USD",
        });
        pricing.insert("cohere.command-r-v1:0", ModelPricing {
            input_cost_per_1k: 0.0005,
            output_cost_per_1k: 0.0015,
            currency: "USD",
        });
        pricing.insert("cohere.command-text-v14", ModelPricing {
            input_cost_per_1k: 0.0015,
            output_cost_per_1k: 0.002,
            currency: "USD",
        });
        pricing.insert("cohere.command-light-text-v14", ModelPricing {
            input_cost_per_1k: 0.0003,
            output_cost_per_1k: 0.0006,
            currency: "USD",
        });
        
        // Mistral models
        pricing.insert("mistral.mistral-7b-instruct-v0:2", ModelPricing {
            input_cost_per_1k: 0.00015,
            output_cost_per_1k: 0.0002,
            currency: "USD",
        });
        pricing.insert("mistral.mixtral-8x7b-instruct-v0:1", ModelPricing {
            input_cost_per_1k: 0.00045,
            output_cost_per_1k: 0.0007,
            currency: "USD",
        });
        pricing.insert("mistral.mistral-large-2402-v1:0", ModelPricing {
            input_cost_per_1k: 0.004,
            output_cost_per_1k: 0.012,
            currency: "USD",
        });
        pricing.insert("mistral.mistral-large-2407-v1:0", ModelPricing {
            input_cost_per_1k: 0.002,
            output_cost_per_1k: 0.006,
            currency: "USD",
        });
        pricing.insert("mistral.mistral-small-2402-v1:0", ModelPricing {
            input_cost_per_1k: 0.001,
            output_cost_per_1k: 0.003,
            currency: "USD",
        });
        
        // Meta Llama models
        pricing.insert("meta.llama3-2-1b-instruct-v1:0", ModelPricing {
            input_cost_per_1k: 0.00001,
            output_cost_per_1k: 0.00001,
            currency: "USD",
        });
        pricing.insert("meta.llama3-2-3b-instruct-v1:0", ModelPricing {
            input_cost_per_1k: 0.000015,
            output_cost_per_1k: 0.000015,
            currency: "USD",
        });
        pricing.insert("meta.llama3-2-11b-instruct-v1:0", ModelPricing {
            input_cost_per_1k: 0.000032,
            output_cost_per_1k: 0.000032,
            currency: "USD",
        });
        pricing.insert("meta.llama3-2-90b-instruct-v1:0", ModelPricing {
            input_cost_per_1k: 0.00072,
            output_cost_per_1k: 0.00072,
            currency: "USD",
        });
        pricing.insert("meta.llama3-1-8b-instruct-v1:0", ModelPricing {
            input_cost_per_1k: 0.00022,
            output_cost_per_1k: 0.00022,
            currency: "USD",
        });
        pricing.insert("meta.llama3-1-70b-instruct-v1:0", ModelPricing {
            input_cost_per_1k: 0.00099,
            output_cost_per_1k: 0.00099,
            currency: "USD",
        });
        pricing.insert("meta.llama3-1-405b-instruct-v1:0", ModelPricing {
            input_cost_per_1k: 0.00532,
            output_cost_per_1k: 0.016,
            currency: "USD",
        });
        pricing.insert("meta.llama3-8b-instruct-v1:0", ModelPricing {
            input_cost_per_1k: 0.0003,
            output_cost_per_1k: 0.0006,
            currency: "USD",
        });
        pricing.insert("meta.llama3-70b-instruct-v1:0", ModelPricing {
            input_cost_per_1k: 0.00265,
            output_cost_per_1k: 0.0035,
            currency: "USD",
        });
        pricing.insert("meta.llama2-13b-chat-v1", ModelPricing {
            input_cost_per_1k: 0.00075,
            output_cost_per_1k: 0.001,
            currency: "USD",
        });
        pricing.insert("meta.llama2-70b-chat-v1", ModelPricing {
            input_cost_per_1k: 0.00195,
            output_cost_per_1k: 0.00256,
            currency: "USD",
        });
        
        pricing
});

/// Cost calculator for Bedrock models
pub struct CostCalculator;

impl CostCalculator {
    /// Calculate cost for a specific model and token usage
    pub fn calculate_cost(
        model_id: &str,
        input_tokens: u32,
        output_tokens: u32,
    ) -> Option<f64> {
        MODEL_PRICING.get(model_id).map(|pricing| {
            let input_cost = (input_tokens as f64 / 1000.0) * pricing.input_cost_per_1k;
            let output_cost = (output_tokens as f64 / 1000.0) * pricing.output_cost_per_1k;
            input_cost + output_cost
        })
    }
    
    /// Get pricing information for a model
    pub fn get_model_pricing(model_id: &str) -> Option<&'static ModelPricing> {
        MODEL_PRICING.get(model_id)
    }
    
    /// Get all available models with pricing
    pub fn get_all_models() -> Vec<&'static str> {
        MODEL_PRICING.keys().copied().collect()
    }
    
    /// Calculate cost with breakdown
    pub fn calculate_detailed_cost(
        model_id: &str,
        input_tokens: u32,
        output_tokens: u32,
    ) -> Option<CostBreakdown> {
        MODEL_PRICING.get(model_id).map(|pricing| {
            let input_cost = (input_tokens as f64 / 1000.0) * pricing.input_cost_per_1k;
            let output_cost = (output_tokens as f64 / 1000.0) * pricing.output_cost_per_1k;
            
            CostBreakdown {
                input_tokens,
                output_tokens,
                input_cost,
                output_cost,
                total_cost: input_cost + output_cost,
                currency: pricing.currency,
            }
        })
    }
}

/// Detailed cost breakdown
#[derive(Debug, Clone)]
pub struct CostBreakdown {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub input_cost: f64,
    pub output_cost: f64,
    pub total_cost: f64,
    pub currency: &'static str,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_calculation() {
        // Test Claude Opus pricing
        let cost = CostCalculator::calculate_cost(
            "anthropic.claude-3-opus-20240229",
            1000, // 1k input tokens
            500,  // 500 output tokens
        ).unwrap();
        
        // Expected: (1000/1000 * 0.015) + (500/1000 * 0.075) = 0.015 + 0.0375 = 0.0525
        assert!((cost - 0.0525).abs() < 0.0001);
    }

    #[test]
    fn test_model_pricing_lookup() {
        let pricing = CostCalculator::get_model_pricing("anthropic.claude-3-opus-20240229").unwrap();
        assert_eq!(pricing.input_cost_per_1k, 0.015);
        assert_eq!(pricing.output_cost_per_1k, 0.075);
        assert_eq!(pricing.currency, "USD");
    }

    #[test]
    fn test_detailed_cost_breakdown() {
        let breakdown = CostCalculator::calculate_detailed_cost(
            "amazon.titan-text-express-v1",
            2000,
            1000,
        ).unwrap();
        
        assert_eq!(breakdown.input_tokens, 2000);
        assert_eq!(breakdown.output_tokens, 1000);
        assert_eq!(breakdown.currency, "USD");
        assert!(breakdown.total_cost > 0.0);
    }

    #[test]
    fn test_unknown_model() {
        let cost = CostCalculator::calculate_cost("unknown-model", 1000, 500);
        assert!(cost.is_none());
    }

    #[test]
    fn test_all_models_list() {
        let models = CostCalculator::get_all_models();
        assert!(!models.is_empty());
        assert!(models.contains(&"anthropic.claude-3-opus-20240229"));
        assert!(models.contains(&"amazon.titan-text-express-v1"));
    }
}