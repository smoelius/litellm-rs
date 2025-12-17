//! Function calling support for AI providers
//!
//! This module provides OpenAI-compatible function calling capabilities.

mod builtin;
mod conversion;
mod executor;
mod processing;
#[cfg(test)]
mod tests;
mod types;

// Re-export public API
pub use builtin::{CalculatorFunction, WeatherFunction};
pub use executor::{FunctionCallingHandler, FunctionExecutor};
pub use types::{
    FunctionCall, FunctionChoice, FunctionDefinition, ToolCall, ToolChoice, ToolDefinition,
};
