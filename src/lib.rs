//! # LiteLLM-RS
//!
//! A Rust implementation of Python LiteLLM - call 100+ LLM APIs using OpenAI format.
//! High-performance AI Gateway with unified interface for multiple providers.
//!
//! ## Features
//!
//! - **Python LiteLLM Compatible**: Drop-in replacement with same API design
//! - **OpenAI Compatible**: Full compatibility with OpenAI API format
//! - **Multi-Provider**: Support for 100+ AI providers (OpenAI, Anthropic, Azure, Google, etc.)
//! - **Unified Interface**: Call any LLM using the same function signature
//! - **High Performance**: Built with Rust and Tokio for maximum throughput
//! - **Intelligent Routing**: Smart load balancing and failover across providers
//! - **Cost Optimization**: Automatic cost tracking and provider selection
//! - **Streaming Support**: Real-time response streaming
//!
//! ## Quick Start - Python LiteLLM Style
//!
//! ```rust,no_run
//! use litellm_rs::{completion, user_message, system_message};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Call OpenAI (default provider for gpt-* models)
//!     let response = completion(
//!         "gpt-4",
//!         vec![
//!             system_message("You are a helpful assistant."),
//!             user_message("Hello, how are you?"),
//!         ],
//!         None,
//!     ).await?;
//!     
//!     println!("Response: {}", response.choices[0].message.content);
//!
//!     // Call Anthropic with explicit provider
//!     let response = completion(
//!         "anthropic/claude-3-sonnet-20240229",
//!         vec![user_message("What is the capital of France?")],
//!         None,
//!     ).await?;
//!     
//!     println!("Claude says: {}", response.choices[0].message.content);
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Gateway Mode
//!
//! ```rust,no_run
//! use litellm_rs::{Gateway, Config};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = Config::from_file("config/gateway.yaml").await?;
//!     let gateway = Gateway::new(config).await?;
//!     gateway.run().await?;
//!     Ok(())
//! }
//! ```

#![allow(missing_docs)]
#![allow(missing_doc_code_examples)]
#![warn(clippy::all)]
#![allow(clippy::module_inception)]

// Public module exports
mod auth;
// Core completion API moved to core::completion
pub mod config;
pub mod core;
mod monitoring;
pub mod sdk; // New SDK module
pub mod server;
pub mod services; // Add services module
pub mod storage;
pub mod utils;

// Re-export main types
pub use config::Config;
pub use utils::error::{GatewayError, Result};

// Export core completion functionality (Python LiteLLM compatible)
pub use core::completion::{
    Choice, CompletionOptions, CompletionResponse, ContentPart, LiteLLMError, Message, Router,
    Usage, acompletion, assistant_message, completion, completion_stream, system_message,
    user_message,
};

// Export streaming types
pub use core::streaming::{ChatCompletionChunk, ChatCompletionChunkChoice, ChatCompletionDelta};

// Export unified type system
pub use core::types::{MessageContent, MessageRole};

// Export core functionality
pub use core::models::{RequestContext, openai::*};
pub use core::providers::{
    Provider, ProviderError, ProviderRegistry, ProviderType, UnifiedProviderError,
};

use tracing::info;

/// A minimal LiteLLM Gateway implementation
pub struct Gateway {
    config: Config,
    server: server::HttpServer,
}

impl Gateway {
    /// Create a new gateway instance
    pub async fn new(config: Config) -> Result<Self> {
        info!("Creating new gateway instance");

        // Create HTTP server
        let server = server::HttpServer::new(&config).await?;

        Ok(Self { config, server })
    }

    /// Run the gateway server
    pub async fn run(self) -> Result<()> {
        info!("Starting LiteLLM Gateway");
        info!("Configuration: {:#?}", self.config);

        // Start HTTP server
        self.server.start().await?;

        Ok(())
    }
}

// Version information
/// Current version of the crate
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
/// Name of the crate
pub const NAME: &str = env!("CARGO_PKG_NAME");
/// Description of the crate
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

/// Gateway build information
#[derive(Debug, Clone)]
pub struct BuildInfo {
    /// Version number
    pub version: &'static str,
    /// Build
    pub build_time: &'static str,
    /// Git commit hash
    pub git_hash: &'static str,
    /// Rust version
    pub rust_version: &'static str,
}

impl Default for BuildInfo {
    fn default() -> Self {
        Self {
            version: VERSION,
            build_time: "unknown",
            git_hash: "unknown",
            rust_version: "unknown",
        }
    }
}

/// Build
pub fn build_info() -> BuildInfo {
    BuildInfo::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_info() {
        let info = build_info();
        assert!(!info.version.is_empty());
        assert_eq!(info.version, VERSION);
    }

    #[test]
    fn test_constants() {
        // Test that constants are defined and have expected values
        assert_eq!(VERSION, env!("CARGO_PKG_VERSION"));
        assert_eq!(NAME, env!("CARGO_PKG_NAME"));
        assert_eq!(DESCRIPTION, env!("CARGO_PKG_DESCRIPTION"));
    }
}
