//! Cloudflare Workers AI Provider
//!
//! Cloudflare Workers AI provides access to various open-source models
//! running on Cloudflare's global network infrastructure.

// Core modules
mod config;
mod error;
mod provider;
mod model_info;

// Re-export main types for external use
pub use config::CloudflareConfig;
pub use error::{CloudflareError, CloudflareErrorMapper};
pub use provider::CloudflareProvider;
pub use model_info::{CloudflareModel, get_model_info};