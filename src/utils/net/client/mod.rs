//! HTTP Client utilities
//!
//! This module provides HTTP client configuration, retry logic, and network utilities
//! for interacting with AI provider APIs.

mod types;
mod utils;

#[cfg(test)]
mod tests;

// Re-export all public types and utilities for backward compatibility
pub use types::{HttpClientConfig, RequestMetrics, RetryConfig};
pub use utils::ClientUtils;
