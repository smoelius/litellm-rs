//! Anthropic Provider Cost Calculation
//!
//! Simple delegation to the unified cost calculation system

use crate::core::cost::{
    CostCalculator,
    calculator::{estimate_cost, generic_cost_per_token, get_model_pricing},
    types::{CostBreakdown, CostError, CostEstimate, ModelPricing, UsageTokens},
};
use async_trait::async_trait;

/// Anthropic Cost Calculator - delegates to generic implementation
#[derive(Debug, Clone)]
pub struct AnthropicCostCalculator;

impl AnthropicCostCalculator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AnthropicCostCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CostCalculator for AnthropicCostCalculator {
    type Error = CostError;

    async fn calculate_cost(
        &self,
        model: &str,
        usage: &UsageTokens,
    ) -> Result<CostBreakdown, Self::Error> {
        generic_cost_per_token(model, usage, "anthropic")
    }

    async fn estimate_cost(
        &self,
        model: &str,
        input_tokens: u32,
        max_output_tokens: Option<u32>,
    ) -> Result<CostEstimate, Self::Error> {
        estimate_cost(model, "anthropic", input_tokens, max_output_tokens)
    }

    fn get_model_pricing(&self, model: &str) -> Result<ModelPricing, Self::Error> {
        get_model_pricing(model, "anthropic")
    }

    fn provider_name(&self) -> &str {
        "anthropic"
    }
}

/// Helper function for easy cost calculation (maintains compatibility)
pub fn cost_per_token(model: &str, usage: &UsageTokens) -> Result<(f64, f64), CostError> {
    let breakdown = generic_cost_per_token(model, usage, "anthropic")?;
    Ok((breakdown.input_cost, breakdown.output_cost))
}

/// Get Anthropic model pricing (convenience function)
pub fn get_anthropic_model_pricing(model: &str) -> Result<ModelPricing, CostError> {
    get_model_pricing(model, "anthropic")
}

/// Helper function compatible with old API (returns Option)
pub fn calculate_anthropic_cost(model: &str, input_tokens: u32, output_tokens: u32) -> Option<f64> {
    let usage = UsageTokens::new(input_tokens, output_tokens);
    generic_cost_per_token(model, &usage, "anthropic")
        .ok()
        .map(|b| b.total_cost)
}
