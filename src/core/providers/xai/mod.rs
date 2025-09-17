//! xAI Provider
//!
//! xAI provides access to Grok models through an OpenAI-compatible API.
//! Grok models are designed for advanced reasoning and understanding.

// Core modules
mod config;
mod error;
mod provider;
mod model_info;

// Re-export main types for external use
pub use config::XAIConfig;
pub use error::{XAIError, XAIErrorMapper};
pub use provider::XAIProvider;
pub use model_info::{XAIModel, get_model_info};