//! Groq Provider
//!
//! Groq provides ultra-fast AI inference using their Language Processing Units (LPUs).
//! This implementation provides access to various open-source models through Groq's
//! OpenAI-compatible API with optimizations for their specific capabilities.

// Core modules
mod config;
mod error;
mod provider;
mod model_info;

// Feature modules
pub mod streaming;
pub mod stt;

// Tests
#[cfg(test)]
mod tests;

// Re-export main types for external use
pub use config::GroqConfig;
pub use error::{GroqError, GroqErrorMapper};
pub use provider::GroqProvider;
pub use model_info::{GroqModel, get_model_info, is_reasoning_model};

// Re-export feature types
pub use stt::SpeechToTextRequest;