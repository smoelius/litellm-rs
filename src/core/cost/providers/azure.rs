//! Azure Provider Cost Calculation
//!
//! Simple delegation to the unified cost calculation system

use crate::core::cost::{
    CostCalculator,
    calculator::{estimate_cost, generic_cost_per_token, get_model_pricing},
    types::{CostBreakdown, CostError, CostEstimate, ModelPricing, UsageTokens},
};
use async_trait::async_trait;

/// Azure Cost Calculator - delegates to generic implementation
#[derive(Debug, Clone)]
pub struct AzureCostCalculator;

impl AzureCostCalculator {
    pub fn new() -> Self {
        Self
    }

    /// Calculate fine-tuning cost
    pub fn calculate_fine_tuning_cost(
        &self,
        base_model: &str,
        training_tokens: u32,
        hosting_hours: f64,
    ) -> Result<f64, CostError> {
        let pricing = get_model_pricing(base_model, "azure")?;

        // Fine-tuning cost calculation (simplified)
        let training_cost =
            (training_tokens as f64 / 1000.0) * pricing.input_cost_per_1k_tokens * 10.0; // Rough multiplier
        let hosting_cost = hosting_hours * 1.02; // $1.02/hour for hosting

        Ok(training_cost + hosting_cost)
    }

    /// Calculate DALL-E cost for Azure
    pub fn calculate_dalle_cost(
        &self,
        model: &str,
        size: &str,
        quality: Option<&str>,
        n: u32,
    ) -> Result<f64, CostError> {
        let pricing = get_model_pricing(model, "azure")?;

        if let Some(ref cost_per_image) = pricing.cost_per_image {
            let cost_multiplier = if model.contains("dall-e-3") {
                match (size, quality.unwrap_or("standard")) {
                    ("1024x1024", "standard") => 1.0,
                    ("1024x1024", "hd") => 2.0,
                    ("1024x1792", "standard") | ("1792x1024", "standard") => 2.0,
                    ("1024x1792", "hd") | ("1792x1024", "hd") => 4.0,
                    _ => 1.0,
                }
            } else {
                match size {
                    "256x256" => 0.5,
                    "512x512" => 1.0,
                    "1024x1024" => 1.5,
                    _ => 1.0,
                }
            };

            // Use base cost from pricing
            let base_cost = cost_per_image.get("base").copied().unwrap_or(0.04);
            return Ok(base_cost * cost_multiplier * n as f64);
        }

        Err(CostError::MissingPricing {
            model: model.to_string(),
        })
    }
}

impl Default for AzureCostCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CostCalculator for AzureCostCalculator {
    type Error = CostError;

    async fn calculate_cost(
        &self,
        model: &str,
        usage: &UsageTokens,
    ) -> Result<CostBreakdown, Self::Error> {
        generic_cost_per_token(model, usage, "azure")
    }

    async fn estimate_cost(
        &self,
        model: &str,
        input_tokens: u32,
        max_output_tokens: Option<u32>,
    ) -> Result<CostEstimate, Self::Error> {
        estimate_cost(model, "azure", input_tokens, max_output_tokens)
    }

    fn get_model_pricing(&self, model: &str) -> Result<ModelPricing, Self::Error> {
        get_model_pricing(model, "azure")
    }

    fn provider_name(&self) -> &str {
        "azure"
    }
}

/// Helper function for easy cost calculation (maintains compatibility)
pub fn cost_per_token(model: &str, usage: &UsageTokens) -> Result<(f64, f64), CostError> {
    let breakdown = generic_cost_per_token(model, usage, "azure")?;
    Ok((breakdown.input_cost, breakdown.output_cost))
}

/// Get Azure model pricing (convenience function)
pub fn get_azure_model_pricing(model: &str) -> Result<ModelPricing, CostError> {
    get_model_pricing(model, "azure")
}
