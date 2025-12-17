//! HTTP server implementation
//!
//! This module provides the HTTP server and routing functionality.

// Submodules
pub mod middleware;
pub mod routes;

// New modular server components
mod builder;
mod handlers;
mod server;
mod state;
mod types;
mod utils;

#[cfg(test)]
mod tests;

// Re-export public types
pub use builder::{run_server, ServerBuilder};
pub use server::HttpServer;
pub use state::AppState;
pub use types::{RequestMetrics, ServerHealth};
