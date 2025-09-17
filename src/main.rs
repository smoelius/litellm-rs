//! LiteLLM-RS - High-performance async AI gateway
//!
//! Async gateway service supporting multiple AI providers

#![allow(missing_docs)]

use litellm_rs::server;
use tracing::Level;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize logging system
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .with_thread_ids(false)
        .init();

    // Start server (auto-loads config/gateway.yaml)
    server::run_server().await.map_err(|e| e.into())
}
