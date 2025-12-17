//! Configuration builder for type-safe configuration construction
//!
//! This module provides a builder pattern for creating configurations
//! with compile-time validation and better ergonomics.

#![allow(dead_code)] // Builder module - functions may be used in the future

mod config_builder;
mod presets;
mod provider_builder;
mod server_builder;
#[cfg(test)]
mod tests;
mod types;

// Re-export public types and implementations
pub use config_builder::*;
pub use presets::*;
pub use provider_builder::*;
pub use server_builder::*;
pub use types::{ConfigBuilder, ProviderConfigBuilder, ServerConfigBuilder};
