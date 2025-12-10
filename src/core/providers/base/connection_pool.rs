use std::sync::Arc;
use std::time::Duration;

use reqwest::Client;
use serde_json;

use crate::core::providers::unified_provider::ProviderError;

#[derive(Debug, Clone)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
}

/// Unified connection pool configuration
pub struct PoolConfig;
impl PoolConfig {
    pub const TIMEOUT_SECS: u64 = 600;
    pub const POOL_SIZE: usize = 80;
    pub const KEEPALIVE_SECS: u64 = 90;
}

/// Simplified connection pool without generic complexity
#[derive(Debug, Clone)]
pub struct ConnectionPool {
    client: Arc<Client>,
}

impl ConnectionPool {
    /// Create a new connection pool with optimized settings
    pub fn new() -> Result<Self, ProviderError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(PoolConfig::TIMEOUT_SECS))
            .pool_idle_timeout(Duration::from_secs(PoolConfig::KEEPALIVE_SECS))
            .pool_max_idle_per_host(PoolConfig::POOL_SIZE)
            .build()
            .map_err(|e| {
                ProviderError::configuration("Failed to create HTTP client", e.to_string())
            })?;

        Ok(Self {
            client: Arc::new(client),
        })
    }

    /// Get the underlying reqwest client
    pub fn client(&self) -> &Client {
        &self.client
    }
}

/// Global pool manager - single instance for all providers
#[derive(Debug, Clone)]
pub struct GlobalPoolManager {
    pool: Arc<ConnectionPool>,
}

impl GlobalPoolManager {
    /// Create a new global pool manager
    pub fn new() -> Result<Self, ProviderError> {
        Ok(Self {
            pool: Arc::new(ConnectionPool::new()?),
        })
    }

    /// Execute an HTTP request
    pub async fn execute_request(
        &self,
        url: &str,
        method: HttpMethod,
        headers: Vec<(String, String)>,
        body: Option<serde_json::Value>,
    ) -> Result<reqwest::Response, ProviderError> {
        let client = self.pool.client();

        let mut request_builder = match method {
            HttpMethod::GET => client.get(url),
            HttpMethod::POST => client.post(url),
            HttpMethod::PUT => client.put(url),
            HttpMethod::DELETE => client.delete(url),
        };

        // Add headers
        for (key, value) in headers {
            request_builder = request_builder.header(&key, &value);
        }

        // Add body if present
        if let Some(body_data) = body {
            request_builder = request_builder
                .header("Content-Type", "application/json")
                .json(&body_data);
        }

        request_builder
            .send()
            .await
            .map_err(|e| ProviderError::network("common", e.to_string()))
    }

    /// Get the underlying client for direct use
    pub fn client(&self) -> &Client {
        self.pool.client()
    }
}

impl Default for GlobalPoolManager {
    /// Create a default GlobalPoolManager
    ///
    /// Note: This will panic if the HTTP client cannot be created,
    /// which should be extremely rare. For fallible construction,
    /// use `GlobalPoolManager::new()` directly.
    fn default() -> Self {
        Self::new().unwrap_or_else(|e| {
            tracing::error!(
                "Failed to create GlobalPoolManager: {}, using minimal client",
                e
            );
            // Fallback to a basic client without custom settings
            Self {
                pool: std::sync::Arc::new(ConnectionPool {
                    client: std::sync::Arc::new(Client::new()),
                }),
            }
        })
    }
}

// Re-export for backwards compatibility
pub use ConnectionPool as LockFreePool;
pub use GlobalPoolManager as ConnectionGuard;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pool_creation() {
        let pool = ConnectionPool::new();
        assert!(pool.is_ok());
    }

    #[tokio::test]
    async fn test_global_manager() {
        let manager = GlobalPoolManager::new();
        assert!(manager.is_ok());
    }
}
