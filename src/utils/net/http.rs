//! Shared HTTP client for optimal connection pooling
//!
//! This module provides a high-performance shared HTTP client with connection reuse.

use reqwest::{Client, ClientBuilder};
use std::sync::OnceLock;
use std::time::Duration;
use tracing::{debug, warn};

/// Shared HTTP client instance with optimized settings  
#[allow(dead_code)]
static SHARED_HTTP_CLIENT: OnceLock<Client> = OnceLock::new();

/// Get the shared HTTP client instance
#[allow(dead_code)]
pub fn get_shared_client() -> &'static Client {
    SHARED_HTTP_CLIENT.get_or_init(|| {
        debug!("Initializing shared HTTP client with optimized settings");

        ClientBuilder::new()
            // Connection pool settings
            .pool_max_idle_per_host(20)
            .pool_idle_timeout(Duration::from_secs(90))
            // Request timeouts
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            // TCP optimizations
            .tcp_keepalive(Duration::from_secs(60))
            .tcp_nodelay(true)
            // HTTP/2 support
            .http2_prior_knowledge()
            // User agent
            .user_agent("LiteLLM-RS/0.1.0")
            // Build with error handling
            .build()
            .unwrap_or_else(|e| {
                warn!(
                    "Failed to create optimized HTTP client, falling back to default: {}",
                    e
                );
                Client::new()
            })
    })
}

/// Create a custom HTTP client with specific timeout
#[allow(dead_code)]
pub fn create_custom_client(timeout: Duration) -> Result<Client, reqwest::Error> {
    ClientBuilder::new()
        .pool_max_idle_per_host(20)
        .pool_idle_timeout(Duration::from_secs(90))
        .timeout(timeout)
        .connect_timeout(Duration::from_secs(10))
        .tcp_keepalive(Duration::from_secs(60))
        .tcp_nodelay(true)
        .http2_prior_knowledge()
        .user_agent("LiteLLM-RS/0.1.0")
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shared_client_creation() {
        let client = get_shared_client();
        // Just verify we can get the client without panicking
        assert!(std::ptr::addr_of!(*client) == std::ptr::addr_of!(*get_shared_client()));
    }

    #[test]
    fn test_custom_client_creation() {
        let client = create_custom_client(Duration::from_secs(15));
        assert!(client.is_ok());
    }
}
