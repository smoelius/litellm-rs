//! LLM Provider module
//!
//! This module provides the unified interface for all AI providers.
//! The original `llm_provider.rs` has been split into smaller modules for better maintainability.

mod types;
mod trait_definition;

// Re-export everything for backward compatibility
pub use trait_definition::LLMProvider;
