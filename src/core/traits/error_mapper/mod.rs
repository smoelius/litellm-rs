//! Error mapping traits and implementations
//!
//! This module provides error mapping functionality to convert HTTP responses
//! and other errors into provider-specific error types.
//!
//! # Module Structure
//!
//! - `trait_def` - Core ErrorMapper trait definition
//! - `types` - Generic error mapper implementation
//! - `implementations` - Provider-specific error mappers (OpenAI, Anthropic)
//! - `tests` - Comprehensive test suite

mod trait_def;
mod types;
mod implementations;

#[cfg(test)]
mod tests;

// Re-export all public items for backward compatibility
pub use trait_def::ErrorMapper;
pub use types::GenericErrorMapper;
pub use implementations::{OpenAIErrorMapper, AnthropicErrorMapper};
