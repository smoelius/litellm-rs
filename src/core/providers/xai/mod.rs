//! xAI Provider
//!
//! xAI provides access to Grok models through an OpenAI-compatible API.
//! Grok models are designed for advanced reasoning and understanding.

// Core modules
mod config;
mod error;
mod model_info;
mod provider;

// Re-export main types for external use
pub use config::XAIConfig;
pub use error::{XAIError, XAIErrorMapper};
pub use model_info::{XAIModel, get_model_info};
pub use provider::XAIProvider;
