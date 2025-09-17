//! OpenAI Provider Cost Calculation
//!
//! Simple delegation to the unified cost calculation system

use crate::core::cost::{
    CostCalculator,
    calculator::{estimate_cost, generic_cost_per_token, get_model_pricing},
    types::{CostBreakdown, CostError, CostEstimate, ModelPricing, UsageTokens},
};
use async_trait::async_trait;

/// OpenAI Cost Calculator - delegates to generic implementation
#[derive(Debug, Clone)]
pub struct OpenAICostCalculator;

impl OpenAICostCalculator {
    pub fn new() -> Self {
        Self
    }

    /// Calculate image generation cost
    pub fn calculate_image_cost(
        &self,
        model: &str,
        size: &str,
        quality: Option<&str>,
        quantity: u32,
    ) -> Result<f64, CostError> {
        let pricing = get_model_pricing(model, "openai")?;

        if let Some(ref cost_per_image) = pricing.cost_per_image {
            let price_key = if model.contains("dall-e-3") && quality == Some("hd") {
                format!("{}-hd", size)
            } else {
                size.to_string()
            };

            if let Some(&cost) = cost_per_image.get(&price_key) {
                return Ok(cost * quantity as f64);
            }
        }

        Err(CostError::MissingPricing {
            model: model.to_string(),
        })
    }

    /// Calculate audio processing cost
    pub fn calculate_audio_cost(
        &self,
        model: &str,
        duration_minutes: f64,
    ) -> Result<f64, CostError> {
        let pricing = get_model_pricing(model, "openai")?;

        if let Some(cost_per_second) = pricing.cost_per_second {
            return Ok(duration_minutes * 60.0 * cost_per_second);
        }

        Err(CostError::MissingPricing {
            model: model.to_string(),
        })
    }
}

impl Default for OpenAICostCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CostCalculator for OpenAICostCalculator {
    type Error = CostError;

    async fn calculate_cost(
        &self,
        model: &str,
        usage: &UsageTokens,
    ) -> Result<CostBreakdown, Self::Error> {
        generic_cost_per_token(model, usage, "openai")
    }

    async fn estimate_cost(
        &self,
        model: &str,
        input_tokens: u32,
        max_output_tokens: Option<u32>,
    ) -> Result<CostEstimate, Self::Error> {
        estimate_cost(model, "openai", input_tokens, max_output_tokens)
    }

    fn get_model_pricing(&self, model: &str) -> Result<ModelPricing, Self::Error> {
        get_model_pricing(model, "openai")
    }

    fn provider_name(&self) -> &str {
        "openai"
    }
}

/// Helper function for easy cost calculation (maintains compatibility)
pub fn cost_per_token(model: &str, usage: &UsageTokens) -> Result<(f64, f64), CostError> {
    let breakdown = generic_cost_per_token(model, usage, "openai")?;
    Ok((breakdown.input_cost, breakdown.output_cost))
}

/// Get OpenAI model pricing (convenience function)
pub fn get_openai_model_pricing(model: &str) -> Result<ModelPricing, CostError> {
    get_model_pricing(model, "openai")
}
