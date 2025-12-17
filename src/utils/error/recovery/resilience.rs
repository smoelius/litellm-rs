//! Resilience patterns for resource isolation and timeout protection

use crate::utils::error::{GatewayError, Result};
use std::sync::Arc;
use std::time::Duration;
use tracing::debug;

/// Timeout wrapper for async operations
#[allow(dead_code)]
pub struct TimeoutWrapper {
    timeout: Duration,
}

#[allow(dead_code)]
impl TimeoutWrapper {
    /// Create a new timeout wrapper
    pub fn new(timeout: Duration) -> Self {
        Self { timeout }
    }

    /// Execute a function with timeout protection
    pub async fn call<F, R>(&self, f: F) -> Result<R>
    where
        F: std::future::Future<Output = R>,
    {
        match tokio::time::timeout(self.timeout, f).await {
            Ok(result) => Ok(result),
            Err(_) => Err(GatewayError::Timeout(format!(
                "Operation timed out after {:?}",
                self.timeout
            ))),
        }
    }
}

/// Bulkhead pattern for resource isolation
#[allow(dead_code)]
pub struct Bulkhead {
    semaphore: Arc<tokio::sync::Semaphore>,
    name: String,
    max_concurrent: usize,
}

#[allow(dead_code)]
impl Bulkhead {
    /// Create a new bulkhead
    pub fn new(name: String, max_concurrent: usize) -> Self {
        Self {
            semaphore: Arc::new(tokio::sync::Semaphore::new(max_concurrent)),
            name,
            max_concurrent,
        }
    }

    /// Execute a function with bulkhead protection
    pub async fn call<F, R>(&self, f: F) -> Result<R>
    where
        F: std::future::Future<Output = Result<R>>,
    {
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|e| GatewayError::Internal(format!("Bulkhead acquire failed: {}", e)))?;

        debug!("Bulkhead '{}' acquired permit", self.name);

        let result = f.await;

        debug!("Bulkhead '{}' released permit", self.name);

        result
    }

    /// Get available permits
    pub fn available_permits(&self) -> usize {
        self.semaphore.available_permits()
    }

    /// Get maximum concurrent operations
    pub fn max_concurrent(&self) -> usize {
        self.max_concurrent
    }
}
