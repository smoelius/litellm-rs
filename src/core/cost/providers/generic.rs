//! Generic Provider Cost Calculation
//!
//! Default implementation for providers without specific cost calculation needs

use crate::core::cost::{
    CostCalculator,
    calculator::{estimate_cost, generic_cost_per_token, get_model_pricing},
    types::{CostBreakdown, CostError, CostEstimate, ModelPricing, UsageTokens},
};
use async_trait::async_trait;

/// Generic Cost Calculator that can be used by any provider
#[derive(Debug, Clone)]
pub struct GenericCostCalculator {
    provider_name: String,
}

impl GenericCostCalculator {
    pub fn new(provider_name: String) -> Self {
        Self { provider_name }
    }
}

#[async_trait]
impl CostCalculator for GenericCostCalculator {
    type Error = CostError;

    async fn calculate_cost(
        &self,
        model: &str,
        usage: &UsageTokens,
    ) -> Result<CostBreakdown, Self::Error> {
        generic_cost_per_token(model, usage, &self.provider_name)
    }

    async fn estimate_cost(
        &self,
        model: &str,
        input_tokens: u32,
        max_output_tokens: Option<u32>,
    ) -> Result<CostEstimate, Self::Error> {
        estimate_cost(model, &self.provider_name, input_tokens, max_output_tokens)
    }

    fn get_model_pricing(&self, model: &str) -> Result<ModelPricing, Self::Error> {
        get_model_pricing(model, &self.provider_name)
    }

    fn provider_name(&self) -> &str {
        &self.provider_name
    }
}

/// Creates a generic cost calculator for any provider
pub fn create_generic_calculator(provider: &str) -> GenericCostCalculator {
    GenericCostCalculator::new(provider.to_string())
}

/// Simple stub cost calculator that returns zero costs
/// Useful for providers without pricing information
#[derive(Debug, Clone)]
pub struct StubCostCalculator {
    provider_name: String,
}

impl StubCostCalculator {
    pub fn new(provider_name: String) -> Self {
        Self { provider_name }
    }
}

#[async_trait]
impl CostCalculator for StubCostCalculator {
    type Error = CostError;

    async fn calculate_cost(
        &self,
        model: &str,
        usage: &UsageTokens,
    ) -> Result<CostBreakdown, Self::Error> {
        let mut breakdown =
            CostBreakdown::new(model.to_string(), self.provider_name.clone(), usage.clone());
        // Return zero costs for stub
        breakdown.calculate_total();
        Ok(breakdown)
    }

    async fn estimate_cost(
        &self,
        _model: &str,
        _input_tokens: u32,
        _max_output_tokens: Option<u32>,
    ) -> Result<CostEstimate, Self::Error> {
        Ok(CostEstimate {
            min_cost: 0.0,
            max_cost: 0.0,
            input_cost: 0.0,
            estimated_output_cost: 0.0,
            currency: "USD".to_string(),
        })
    }

    fn get_model_pricing(&self, model: &str) -> Result<ModelPricing, Self::Error> {
        Err(CostError::ModelNotSupported {
            model: model.to_string(),
            provider: self.provider_name.clone(),
        })
    }

    fn provider_name(&self) -> &str {
        &self.provider_name
    }
}
