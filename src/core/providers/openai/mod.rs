//! OpenAI Provider - New Architecture Implementation
//!
//! Complete OpenAI API integration following the unified provider architecture.
//! Supports all OpenAI services: Chat, Images, Audio, Embeddings, Fine-tuning, etc.

pub mod client;
pub mod config;
pub mod error;
pub mod models;
pub mod provider;
pub mod streaming;
pub mod transformer;

// Feature-specific modules
pub mod capabilities;

// New functionality modules
pub mod advanced_chat;
pub mod completions;
pub mod fine_tuning;
pub mod image_edit;
pub mod image_variations;
pub mod realtime;
pub mod vector_stores;

// Re-exports for easy access
pub use capabilities::*;
pub use client::OpenAIProvider;
pub use config::OpenAIConfig;
pub use error::OpenAIError;
pub use models::{OpenAIModelRegistry, get_openai_registry};
pub use transformer::{OpenAIRequestTransformer, OpenAIResponseTransformer, OpenAITransformer};
