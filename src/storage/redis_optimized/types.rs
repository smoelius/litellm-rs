//! Type definitions for optimized Redis storage
//!
//! Contains configuration types, statistics, and connection metadata.

use std::time::{Duration, Instant};

/// Connection pool statistics for monitoring Redis performance
#[derive(Debug, Clone, Default)]
pub struct PoolStats {
    /// Total number of connections in the pool
    pub total_connections: usize,
    /// Number of actively used connections
    pub active_connections: usize,
    /// Number of idle connections waiting to be used
    pub idle_connections: usize,
    /// Total number of requests processed
    pub total_requests: u64,
    /// Number of failed requests
    pub failed_requests: u64,
    /// Average response time in milliseconds
    pub average_response_time_ms: f64,
}

/// Connection pool configuration for tuning Redis performance
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Maximum number of connections in the pool
    pub max_connections: usize,
    /// Minimum number of connections to maintain
    pub min_connections: usize,
    /// Connection timeout in seconds
    pub connection_timeout: Duration,
    /// Maximum idle time before connection is closed
    pub max_idle_time: Duration,
    /// Health check interval for background monitoring
    pub health_check_interval: Duration,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 20,
            min_connections: 5,
            connection_timeout: Duration::from_secs(5),
            max_idle_time: Duration::from_secs(300), // 5 minutes
            health_check_interval: Duration::from_secs(30),
        }
    }
}

/// Connection wrapper with metadata for pool management
#[derive(Debug)]
pub(super) struct PooledConnection {
    /// The actual Redis connection
    pub(super) connection: redis::aio::MultiplexedConnection,
    /// When this connection was created (for connection age tracking)
    #[allow(dead_code)]
    pub(super) created_at: Instant,
    /// When this connection was last used
    pub(super) last_used: Instant,
    /// Number of requests processed by this connection
    pub(super) request_count: u64,
    /// Whether the connection is healthy
    pub(super) is_healthy: bool,
}

impl PooledConnection {
    /// Create a new pooled connection wrapper
    pub(super) fn new(connection: redis::aio::MultiplexedConnection) -> Self {
        let now = Instant::now();
        Self {
            connection,
            created_at: now,
            last_used: now,
            request_count: 0,
            is_healthy: true,
        }
    }

    /// Mark the connection as recently used
    pub(super) fn mark_used(&mut self) {
        self.last_used = Instant::now();
        self.request_count += 1;
    }

    /// Check if connection has been idle longer than max_idle_time
    pub(super) fn is_idle(&self, max_idle_time: Duration) -> bool {
        self.last_used.elapsed() > max_idle_time
    }
}
