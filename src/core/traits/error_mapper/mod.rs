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

pub mod trait_def;
pub mod types;
pub mod implementations;

#[cfg(test)]
mod tests;
