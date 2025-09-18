//! Cloudflare Workers AI Provider
//!
//! Cloudflare Workers AI provides access to various open-source models
//! running on Cloudflare's global network infrastructure.

// Core modules
mod config;
mod error;
mod model_info;
mod provider;

// Re-export main types for external use
pub use config::CloudflareConfig;
pub use error::{CloudflareError, CloudflareErrorMapper};
pub use model_info::{CloudflareModel, get_model_info};
pub use provider::CloudflareProvider;
