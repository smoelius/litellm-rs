//! Configuration builder for type-safe configuration construction
//!
//! This module provides a builder pattern for creating configurations
//! with compile-time validation and better ergonomics.

#![allow(dead_code)] // Builder module - functions may be used in the future

pub mod config_builder;
pub mod presets;
pub mod provider_builder;
pub mod server_builder;
#[cfg(test)]
mod tests;
pub mod types;
