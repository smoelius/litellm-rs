//! Token counting utilities for the Gateway
//!
//! This module provides token counting functionality for different AI models.

mod token_counter;
mod types;

#[cfg(test)]
mod tests;

// Re-export public types for backward compatibility
pub use token_counter::TokenCounter;
pub use types::{ModelTokenConfig, TokenEstimate};
