//! Unified Cost Calculation Module
//!
//! This module provides a centralized cost calculation system for all providers,
//! eliminating code duplication and ensuring consistency across the codebase.
//!
//! ## Design Philosophy
//! Based on Python LiteLLM's successful pattern:
//! - Single source of truth for cost calculation logic
//! - Providers delegate to generic functions
//! - Centralized model pricing data
//! - Consistent cost structures across all providers

pub mod calculator;
pub mod types;
pub mod utils;

// Re-export main types and functions
pub use calculator::{
    CostCalculator, compare_model_costs, estimate_cost, generic_cost_per_token, get_model_pricing,
};
pub use types::{
    CostBreakdown, CostError, CostEstimate, CostResult, CostSummary, CostTracker,
    ModelCostComparison, ModelPricing, ProviderPricing, UsageTokens,
};
pub use utils::{
    calculate_cost_component, format_cost, get_cost_per_unit, select_tiered_pricing, tokens_to_cost,
};

pub mod providers {
    //! Provider-specific cost calculation modules
    //! Each provider only needs to implement simple delegation functions

    pub mod anthropic;
    pub mod azure;
    pub mod generic;
    pub mod openai;
}
