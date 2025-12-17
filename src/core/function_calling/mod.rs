//! Function calling support for AI providers
//!
//! This module provides OpenAI-compatible function calling capabilities.

pub mod builtin;
mod conversion;
pub mod executor;
mod processing;
#[cfg(test)]
mod tests;
pub mod types;
