//! Provider-specific cost calculation implementations
//! 
//! Each provider delegates to the unified cost calculation system

pub mod anthropic;
pub mod azure;
pub mod openai;
pub mod generic;

// Re-export provider calculators
pub use anthropic::AnthropicCostCalculator;
pub use azure::AzureCostCalculator;
pub use openai::OpenAICostCalculator;
pub use generic::{GenericCostCalculator, StubCostCalculator, create_generic_calculator};

use crate::core::cost::CostCalculator;
use crate::core::cost::types::CostError;
use std::sync::Arc;

/// Factory function to create appropriate cost calculator for a provider
pub fn create_cost_calculator(provider: &str) -> Result<Arc<dyn CostCalculator<Error = CostError> + Send + Sync>, CostError> {
    match provider.to_lowercase().as_str() {
        "openai" => Ok(Arc::new(OpenAICostCalculator::new())),
        "anthropic" => Ok(Arc::new(AnthropicCostCalculator::new())),
        "azure" | "azure_openai" => Ok(Arc::new(AzureCostCalculator::new())),
        "deepseek" | "moonshot" | "vertex_ai" | "gemini" => {
            Ok(Arc::new(create_generic_calculator(provider)))
        }
        _ => Ok(Arc::new(generic::StubCostCalculator::new(provider.to_string())))
    }
}