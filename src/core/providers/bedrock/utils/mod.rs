//! Utility modules for Bedrock provider
//!
//! Contains shared utilities for AWS authentication, region management,
//! cost calculation, and other common functionality.

pub mod auth;
pub mod cost;
pub mod region;

// Re-export main types and functions
pub use auth::{AwsAuth, AwsCredentials};
pub use cost::{CostCalculator, ModelPricing};
pub use region::{
    validate_region,
    is_model_available_in_region,
    AWS_REGIONS
};