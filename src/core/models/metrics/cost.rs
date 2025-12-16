//! Cost information models

use serde::{Deserialize, Serialize};

/// Cost information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CostInfo {
    /// Input cost
    pub input_cost: f64,
    /// Output cost
    pub output_cost: f64,
    /// Total cost
    pub total_cost: f64,
    /// Currency
    pub currency: String,
    /// Cost per token rates
    pub rates: CostRates,
}

/// Cost rates
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CostRates {
    /// Input cost per token
    pub input_cost_per_token: f64,
    /// Output cost per token
    pub output_cost_per_token: f64,
    /// Cost per request
    pub cost_per_request: Option<f64>,
}

impl CostInfo {
    /// Create new cost info
    pub fn new(input_cost: f64, output_cost: f64, currency: String) -> Self {
        Self {
            input_cost,
            output_cost,
            total_cost: input_cost + output_cost,
            currency,
            rates: CostRates::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_calculation() {
        let cost = CostInfo::new(0.01, 0.02, "USD".to_string());
        assert_eq!(cost.input_cost, 0.01);
        assert_eq!(cost.output_cost, 0.02);
        assert_eq!(cost.total_cost, 0.03);
        assert_eq!(cost.currency, "USD");
    }
}
