//! Optimized Redis storage implementation with connection pooling and batch operations
//!
//! This module provides enhanced Redis connectivity with improved performance
//! through connection pooling, batch operations, and intelligent caching.

use crate::config::RedisConfig;
use crate::utils::error::{GatewayError, Result};
use dashmap::DashMap;
use redis::{AsyncCommands, Client, Pipeline, RedisResult, aio::MultiplexedConnection};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};
use tracing::{debug, info, warn};

/// Connection pool statistics
#[derive(Debug, Clone, Default)]
pub struct PoolStats {
    /// Total number of connections
    pub total_connections: usize,
    /// Number of active connections
    pub active_connections: usize,
    /// Number of idle connections
    #[allow(dead_code)] // Reserved for future pool monitoring
    pub idle_connections: usize,
    /// Total number of requests
    pub total_requests: u64,
    /// Number of failed requests
    #[allow(dead_code)] // Reserved for future pool monitoring
    pub failed_requests: u64,
    /// Average response time in milliseconds
    #[allow(dead_code)] // Reserved for future pool monitoring
    pub average_response_time_ms: f64,
}

/// Connection pool configuration
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Maximum number of connections in the pool
    #[allow(dead_code)] // Reserved for future pool configuration
    pub max_connections: usize,
    /// Minimum number of connections to maintain
    #[allow(dead_code)] // Reserved for future pool configuration
    pub min_connections: usize,
    /// Connection timeout in seconds
    #[allow(dead_code)] // Reserved for future pool configuration
    pub connection_timeout: Duration,
    /// Maximum idle time before connection is closed
    pub max_idle_time: Duration,
    /// Health check interval
    #[allow(dead_code)] // Reserved for future pool configuration
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

/// Connection wrapper with metadata
#[derive(Debug)]
#[allow(dead_code)] // Reserved for future connection pooling
struct PooledConnection {
    #[allow(dead_code)] // Reserved for future connection operations
    connection: MultiplexedConnection,
    #[allow(dead_code)] // Reserved for future connection lifecycle management
    created_at: Instant,
    #[allow(dead_code)] // Reserved for future connection tracking
    last_used: Instant,
    #[allow(dead_code)] // Reserved for future connection statistics
    request_count: u64,
    #[allow(dead_code)] // Reserved for future connection health monitoring
    is_healthy: bool,
}

#[allow(dead_code)] // Reserved for future connection pooling
impl PooledConnection {
    #[allow(dead_code)] // Reserved for future connection creation
    fn new(connection: MultiplexedConnection) -> Self {
        let now = Instant::now();
        Self {
            connection,
            created_at: now,
            last_used: now,
            request_count: 0,
            is_healthy: true,
        }
    }

    #[allow(dead_code)] // Reserved for future connection tracking
    fn mark_used(&mut self) {
        self.last_used = Instant::now();
        self.request_count += 1;
    }

    #[allow(dead_code)] // Reserved for future connection management
    fn is_idle(&self, max_idle_time: Duration) -> bool {
        self.last_used.elapsed() > max_idle_time
    }
}

/// Optimized Redis connection pool
#[allow(dead_code)] // Reserved for future optimized Redis operations
pub struct OptimizedRedisPool {
    #[allow(dead_code)] // Reserved for future Redis client operations
    client: Client,
    #[allow(dead_code)] // Reserved for future connection pooling
    connections: Arc<RwLock<Vec<PooledConnection>>>,
    #[allow(dead_code)] // Reserved for future connection limiting
    semaphore: Arc<Semaphore>,
    #[allow(dead_code)] // Reserved for future configuration access
    config: RedisConfig,
    #[allow(dead_code)] // Reserved for future pool configuration
    pool_config: PoolConfig,
    #[allow(dead_code)] // Reserved for future pool statistics
    stats: Arc<RwLock<PoolStats>>,
    #[allow(dead_code)] // Reserved for future performance monitoring
    response_times: Arc<DashMap<String, Vec<Duration>>>,
}

#[allow(dead_code)] // Reserved for future optimized Redis operations
impl OptimizedRedisPool {
    /// Create a new optimized Redis pool
    #[allow(dead_code)] // Reserved for future pool creation
    pub async fn new(config: &RedisConfig, pool_config: PoolConfig) -> Result<Self> {
        info!("Creating optimized Redis connection pool");
        debug!("Redis URL: {}", Self::sanitize_url(&config.url));

        let client = Client::open(config.url.as_str()).map_err(GatewayError::Redis)?;

        let semaphore = Arc::new(Semaphore::new(pool_config.max_connections));
        let connections = Arc::new(RwLock::new(Vec::new()));
        let stats = Arc::new(RwLock::new(PoolStats::default()));
        let response_times = Arc::new(DashMap::new());

        let pool = Self {
            client,
            connections: connections.clone(),
            semaphore,
            config: config.clone(),
            pool_config,
            stats,
            response_times,
        };

        // Initialize minimum connections
        pool.initialize_connections().await?;

        // Start background tasks
        pool.start_health_checker().await;
        pool.start_connection_manager().await;

        info!("Optimized Redis connection pool created successfully");
        Ok(pool)
    }

    /// Initialize minimum connections
    #[allow(dead_code)] // Reserved for future connection initialization
    async fn initialize_connections(&self) -> Result<()> {
        let mut connections = self.connections.write().await;

        for _ in 0..self.pool_config.min_connections {
            match self.create_connection().await {
                Ok(conn) => connections.push(PooledConnection::new(conn)),
                Err(e) => {
                    warn!("Failed to create initial connection: {}", e);
                    break;
                }
            }
        }

        info!("Initialized {} connections", connections.len());
        Ok(())
    }

    /// Create a new connection
    #[allow(dead_code)] // Reserved for future connection creation
    async fn create_connection(&self) -> Result<MultiplexedConnection> {
        let connection = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(GatewayError::Redis)?;

        Ok(connection)
    }

    /// Get a connection from the pool
    #[allow(dead_code)] // Reserved for future connection retrieval
    async fn get_connection(&self) -> Result<MultiplexedConnection> {
        // Acquire semaphore permit
        let _permit = self.semaphore.acquire().await.unwrap();

        // Try to get an existing connection
        {
            let mut connections = self.connections.write().await;
            if let Some(mut pooled_conn) = connections.pop() {
                if pooled_conn.is_healthy && !pooled_conn.is_idle(self.pool_config.max_idle_time) {
                    pooled_conn.mark_used();
                    return Ok(pooled_conn.connection);
                }
            }
        }

        // Create a new connection if none available
        self.create_connection().await
    }

    /// Return a connection to the pool
    #[allow(dead_code)] // Reserved for future connection pooling
    async fn return_connection(&self, connection: MultiplexedConnection) {
        let mut connections = self.connections.write().await;

        if connections.len() < self.pool_config.max_connections {
            connections.push(PooledConnection::new(connection));
        }
        // If pool is full, connection will be dropped
    }

    /// Execute a Redis command with performance tracking
    #[allow(dead_code)] // Reserved for future Redis operations
    pub async fn execute_command<T, F, Fut>(&self, operation: F) -> Result<T>
    where
        F: FnOnce(MultiplexedConnection) -> Fut,
        Fut: std::future::Future<Output = RedisResult<T>>,
    {
        let start_time = Instant::now();
        let connection = self.get_connection().await?;

        let result = operation(connection.clone()).await;
        let duration = start_time.elapsed();

        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.total_requests += 1;

            match &result {
                Ok(_) => {
                    // Update average response time
                    let total_time =
                        stats.average_response_time_ms * (stats.total_requests - 1) as f64;
                    stats.average_response_time_ms =
                        (total_time + duration.as_millis() as f64) / stats.total_requests as f64;
                }
                Err(_) => {
                    stats.failed_requests += 1;
                }
            }
        }

        // Return connection to pool
        self.return_connection(connection).await;

        result.map_err(GatewayError::Redis)
    }

    /// Batch set operations with pipeline
    #[allow(dead_code)] // Reserved for future batch operations
    pub async fn batch_set(&self, pairs: &[(String, String)], ttl: Option<u64>) -> Result<()> {
        if pairs.is_empty() {
            return Ok(());
        }

        self.execute_command(|mut conn| async move {
            let mut pipe = Pipeline::new();
            pipe.atomic();

            for (key, value) in pairs {
                if let Some(ttl_seconds) = ttl {
                    pipe.set_ex(key, value, ttl_seconds);
                } else {
                    pipe.set(key, value);
                }
            }

            pipe.query_async(&mut conn).await
        })
        .await
    }

    /// Batch get operations
    #[allow(dead_code)] // Reserved for future batch operations
    pub async fn batch_get(&self, keys: &[String]) -> Result<Vec<Option<String>>> {
        if keys.is_empty() {
            return Ok(Vec::new());
        }

        self.execute_command(|mut conn| async move { conn.mget(keys).await })
            .await
    }

    /// Batch delete operations
    #[allow(dead_code)] // Reserved for future batch operations
    pub async fn batch_delete(&self, keys: &[String]) -> Result<u64> {
        if keys.is_empty() {
            return Ok(0);
        }

        self.execute_command(|mut conn| async move { conn.del(keys).await })
            .await
    }

    /// Get pool statistics
    #[allow(dead_code)] // Reserved for future pool monitoring
    pub async fn get_stats(&self) -> PoolStats {
        let connections = self.connections.read().await;
        let mut stats = self.stats.read().await.clone();

        stats.total_connections = connections.len();
        stats.active_connections =
            self.pool_config.max_connections - self.semaphore.available_permits();
        stats.idle_connections = connections
            .iter()
            .filter(|c| c.is_idle(self.pool_config.max_idle_time))
            .count();

        stats
    }

    /// Start health checker background task
    #[allow(dead_code)] // Reserved for future health monitoring
    async fn start_health_checker(&self) {
        let connections = self.connections.clone();
        let interval = self.pool_config.health_check_interval;

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);

            loop {
                interval_timer.tick().await;

                let mut conns = connections.write().await;
                conns.retain(|conn| {
                    // Remove unhealthy or idle connections
                    conn.is_healthy && !conn.is_idle(Duration::from_secs(600)) // 10 minutes max idle
                });
            }
        });
    }

    /// Start connection manager background task
    #[allow(dead_code)] // Reserved for future connection management
    async fn start_connection_manager(&self) {
        let connections = self.connections.clone();
        let client = self.client.clone();
        let min_connections = self.pool_config.min_connections;

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(Duration::from_secs(60));

            loop {
                interval_timer.tick().await;

                let mut conns = connections.write().await;

                // Ensure minimum connections
                while conns.len() < min_connections {
                    match client.get_multiplexed_async_connection().await {
                        Ok(conn) => {
                            conns.push(PooledConnection::new(conn));
                            debug!("Added connection to maintain minimum pool size");
                        }
                        Err(e) => {
                            warn!("Failed to create connection for pool maintenance: {}", e);
                            break;
                        }
                    }
                }
            }
        });
    }

    /// Sanitize URL for logging
    #[allow(dead_code)] // Reserved for future URL sanitization
    fn sanitize_url(url: &str) -> String {
        if let Some(at_pos) = url.find('@') {
            if let Some(scheme_end) = url.find("://") {
                format!("{}://[REDACTED]@{}", &url[..scheme_end], &url[at_pos + 1..])
            } else {
                "[REDACTED]".to_string()
            }
        } else {
            url.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_config_default() {
        let config = PoolConfig::default();
        assert_eq!(config.max_connections, 20);
        assert_eq!(config.min_connections, 5);
        assert_eq!(config.connection_timeout, Duration::from_secs(5));
    }

    #[test]
    fn test_pool_stats_default() {
        let stats = PoolStats::default();
        assert_eq!(stats.total_connections, 0);
        assert_eq!(stats.active_connections, 0);
        assert_eq!(stats.total_requests, 0);
    }

    #[test]
    fn test_sanitize_url() {
        let url_with_auth = "redis://user:pass@localhost:6379";
        let sanitized = OptimizedRedisPool::sanitize_url(url_with_auth);
        assert_eq!(sanitized, "redis://[REDACTED]@localhost:6379");

        let url_without_auth = "redis://localhost:6379";
        let sanitized = OptimizedRedisPool::sanitize_url(url_without_auth);
        assert_eq!(sanitized, "redis://localhost:6379");
    }
}
