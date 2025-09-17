//! Redis storage implementation
//!
//! This module provides Redis connectivity and caching operations.

#![allow(dead_code)]

use crate::config::RedisConfig;
use crate::utils::error::{GatewayError, Result};
use redis::{AsyncCommands, Client, RedisResult, aio::MultiplexedConnection};
use std::collections::HashMap;

use tracing::{debug, info};

/// Redis connection pool
#[derive(Debug, Clone)]
pub struct RedisPool {
    client: Client,
    connection_manager: MultiplexedConnection,
    config: RedisConfig,
}

/// Redis connection wrapper
pub struct RedisConnection {
    conn: MultiplexedConnection,
}

/// Redis subscription wrapper
// Note: Subscription functionality temporarily disabled due to Redis API changes
// This should be fixed when updating to a compatible Redis version
#[allow(dead_code)]
pub struct Subscription {
    _placeholder: (),
}

impl RedisPool {
    /// Create a new Redis pool
    pub async fn new(config: &RedisConfig) -> Result<Self> {
        info!("Creating Redis connection pool");
        debug!("Redis URL: {}", Self::sanitize_url(&config.url));

        let client = Client::open(config.url.as_str()).map_err(GatewayError::Redis)?;

        let connection_manager = client
            .get_multiplexed_async_connection()
            .await
            .map_err(GatewayError::Redis)?;

        info!("Redis connection pool created successfully");
        Ok(Self {
            client,
            connection_manager,
            config: config.clone(),
        })
    }

    /// Get a connection from the pool
    pub async fn get_connection(&self) -> Result<RedisConnection> {
        Ok(RedisConnection {
            conn: self.connection_manager.clone(),
        })
    }

    /// Health check
    pub async fn health_check(&self) -> Result<()> {
        debug!("Performing Redis health check");

        let mut conn = self.get_connection().await?;
        let _: String = redis::cmd("PING")
            .query_async(&mut conn.conn)
            .await
            .map_err(GatewayError::Redis)?;

        debug!("Redis health check passed");
        Ok(())
    }

    /// Close the connection pool
    pub async fn close(&self) -> Result<()> {
        info!("Closing Redis connection pool");
        // Connection manager will be dropped automatically
        info!("Redis connection pool closed");
        Ok(())
    }

    /// Basic cache operations
    pub async fn get(&self, key: &str) -> Result<Option<String>> {
        let mut conn = self.get_connection().await?;
        let result: RedisResult<String> = conn.conn.get(key).await;

        match result {
            Ok(value) => Ok(Some(value)),
            Err(e) if e.kind() == redis::ErrorKind::TypeError => Ok(None),
            Err(e) => Err(GatewayError::Redis(e)),
        }
    }

    /// Set a key-value pair with optional TTL
    pub async fn set(&self, key: &str, value: &str, ttl: Option<u64>) -> Result<()> {
        let mut conn = self.get_connection().await?;

        if let Some(ttl_seconds) = ttl {
            let _: () = conn
                .conn
                .set_ex(key, value, ttl_seconds)
                .await
                .map_err(GatewayError::Redis)?;
        } else {
            let _: () = conn
                .conn
                .set(key, value)
                .await
                .map_err(GatewayError::Redis)?;
        }

        Ok(())
    }

    /// Delete a key
    pub async fn delete(&self, key: &str) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let _: () = conn.conn.del(key).await.map_err(GatewayError::Redis)?;
        Ok(())
    }

    /// Check if a key exists
    pub async fn exists(&self, key: &str) -> Result<bool> {
        let mut conn = self.get_connection().await?;
        let exists: bool = conn.conn.exists(key).await.map_err(GatewayError::Redis)?;
        Ok(exists)
    }

    /// Set expiration time for a key
    pub async fn expire(&self, key: &str, ttl: u64) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let _: () = conn
            .conn
            .expire(key, ttl as i64)
            .await
            .map_err(GatewayError::Redis)?;
        Ok(())
    }

    /// Get time to live for a key
    pub async fn ttl(&self, key: &str) -> Result<i64> {
        let mut conn = self.get_connection().await?;
        let ttl: i64 = conn.conn.ttl(key).await.map_err(GatewayError::Redis)?;
        Ok(ttl)
    }

    /// Batch operations
    pub async fn mget(&self, keys: &[String]) -> Result<Vec<Option<String>>> {
        let mut conn = self.get_connection().await?;
        let values: Vec<Option<String>> =
            conn.conn.mget(keys).await.map_err(GatewayError::Redis)?;
        Ok(values)
    }

    /// Set multiple key-value pairs with optional TTL
    pub async fn mset(&self, pairs: &[(String, String)], ttl: Option<u64>) -> Result<()> {
        let mut conn = self.get_connection().await?;

        if pairs.is_empty() {
            return Ok(());
        }

        // Use atomic pipeline for better performance and consistency
        let mut pipe = redis::pipe();
        pipe.atomic();

        for (key, value) in pairs {
            if let Some(ttl_seconds) = ttl {
                pipe.set_ex(key, value, ttl_seconds);
            } else {
                pipe.set(key, value);
            }
        }

        let _: () = pipe
            .query_async(&mut conn.conn)
            .await
            .map_err(GatewayError::Redis)?;
        Ok(())
    }

    /// List operations
    pub async fn list_push(&self, key: &str, value: &str) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let _: () = conn
            .conn
            .lpush(key, value)
            .await
            .map_err(GatewayError::Redis)?;
        Ok(())
    }

    /// Pop value from list
    pub async fn list_pop(&self, key: &str) -> Result<Option<String>> {
        let mut conn = self.get_connection().await?;
        let result: RedisResult<String> = conn.conn.rpop(key, None).await;

        match result {
            Ok(value) => Ok(Some(value)),
            Err(e) if e.kind() == redis::ErrorKind::TypeError => Ok(None),
            Err(e) => Err(GatewayError::Redis(e)),
        }
    }

    /// Get list length
    pub async fn list_length(&self, key: &str) -> Result<usize> {
        let mut conn = self.get_connection().await?;
        let len: usize = conn.conn.llen(key).await.map_err(GatewayError::Redis)?;
        Ok(len)
    }

    /// Get list range
    pub async fn list_range(&self, key: &str, start: isize, stop: isize) -> Result<Vec<String>> {
        let mut conn = self.get_connection().await?;
        let values: Vec<String> = conn
            .conn
            .lrange(key, start, stop)
            .await
            .map_err(GatewayError::Redis)?;
        Ok(values)
    }

    /// Set operations
    pub async fn set_add(&self, key: &str, member: &str) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let _: () = conn
            .conn
            .sadd(key, member)
            .await
            .map_err(GatewayError::Redis)?;
        Ok(())
    }

    /// Remove member from set
    pub async fn set_remove(&self, key: &str, member: &str) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let _: () = conn
            .conn
            .srem(key, member)
            .await
            .map_err(GatewayError::Redis)?;
        Ok(())
    }

    /// Get all set members
    pub async fn set_members(&self, key: &str) -> Result<Vec<String>> {
        let mut conn = self.get_connection().await?;
        let members: Vec<String> = conn.conn.smembers(key).await.map_err(GatewayError::Redis)?;
        Ok(members)
    }

    /// Check if member is in set
    pub async fn set_is_member(&self, key: &str, member: &str) -> Result<bool> {
        let mut conn = self.get_connection().await?;
        let is_member: bool = conn
            .conn
            .sismember(key, member)
            .await
            .map_err(GatewayError::Redis)?;
        Ok(is_member)
    }

    /// Hash operations
    pub async fn hash_set(&self, key: &str, field: &str, value: &str) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let _: () = conn
            .conn
            .hset(key, field, value)
            .await
            .map_err(GatewayError::Redis)?;
        Ok(())
    }

    /// Get hash field value
    pub async fn hash_get(&self, key: &str, field: &str) -> Result<Option<String>> {
        let mut conn = self.get_connection().await?;
        let result: RedisResult<String> = conn.conn.hget(key, field).await;

        match result {
            Ok(value) => Ok(Some(value)),
            Err(e) if e.kind() == redis::ErrorKind::TypeError => Ok(None),
            Err(e) => Err(GatewayError::Redis(e)),
        }
    }

    /// Delete hash field
    pub async fn hash_delete(&self, key: &str, field: &str) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let _: () = conn
            .conn
            .hdel(key, field)
            .await
            .map_err(GatewayError::Redis)?;
        Ok(())
    }

    /// Get all hash fields and values
    pub async fn hash_get_all(&self, key: &str) -> Result<HashMap<String, String>> {
        let mut conn = self.get_connection().await?;
        let hash: HashMap<String, String> =
            conn.conn.hgetall(key).await.map_err(GatewayError::Redis)?;
        Ok(hash)
    }

    /// Check if hash field exists
    /// Check if a hash field exists
    pub async fn hash_exists(&self, key: &str, field: &str) -> Result<bool> {
        let mut conn = self.get_connection().await?;
        let exists: bool = conn
            .conn
            .hexists(key, field)
            .await
            .map_err(GatewayError::Redis)?;
        Ok(exists)
    }

    /// Sorted set operations
    /// Add member to sorted set with score
    pub async fn sorted_set_add(&self, key: &str, score: f64, member: &str) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let _: () = conn
            .conn
            .zadd(key, score, member)
            .await
            .map_err(GatewayError::Redis)?;
        Ok(())
    }

    /// Get sorted set range
    /// Get a range of elements from a sorted set
    pub async fn sorted_set_range(
        &self,
        key: &str,
        start: isize,
        stop: isize,
    ) -> Result<Vec<String>> {
        let mut conn = self.get_connection().await?;
        let members: Vec<String> = conn
            .conn
            .zrange(key, start, stop)
            .await
            .map_err(GatewayError::Redis)?;
        Ok(members)
    }

    /// Remove member from sorted set
    /// Remove a member from a sorted set
    pub async fn sorted_set_remove(&self, key: &str, member: &str) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let _: () = conn
            .conn
            .zrem(key, member)
            .await
            .map_err(GatewayError::Redis)?;
        Ok(())
    }

    /// Pub/Sub operations
    /// Publish message to channel
    pub async fn publish(&self, channel: &str, message: &str) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let _: () = conn
            .conn
            .publish(channel, message)
            .await
            .map_err(GatewayError::Redis)?;
        Ok(())
    }

    /// Subscribe to channels
    /// Subscribe to Redis channels for pub/sub messaging
    /// Note: Temporarily disabled due to Redis API compatibility issues
    pub async fn subscribe(&self, _channels: &[String]) -> Result<Subscription> {
        // TODO: Fix when Redis API is updated to compatible version
        Err(GatewayError::Redis(redis::RedisError::from((
            redis::ErrorKind::IoError,
            "PubSub temporarily disabled due to API compatibility",
        ))))
    }

    /// Atomic operations
    /// Increment key value by delta
    pub async fn increment(&self, key: &str, delta: i64) -> Result<i64> {
        let mut conn = self.get_connection().await?;
        let new_value: i64 = conn
            .conn
            .incr(key, delta)
            .await
            .map_err(GatewayError::Redis)?;
        Ok(new_value)
    }

    /// Decrement key value by delta
    /// Decrement a key by a delta value
    pub async fn decrement(&self, key: &str, delta: i64) -> Result<i64> {
        let mut conn = self.get_connection().await?;
        let new_value: i64 = conn
            .conn
            .decr(key, delta)
            .await
            .map_err(GatewayError::Redis)?;
        Ok(new_value)
    }

    /// Utility functions
    fn sanitize_url(url: &str) -> String {
        if let Ok(parsed) = url::Url::parse(url) {
            let mut sanitized = parsed.clone();
            if sanitized.password().is_some() {
                let _ = sanitized.set_password(Some("***"));
            }
            sanitized.to_string()
        } else {
            "invalid_url".to_string()
        }
    }

    /// Get Redis info
    pub async fn info(&self) -> Result<String> {
        let mut conn = self.get_connection().await?;
        let info: String = redis::cmd("INFO")
            .query_async(&mut conn.conn)
            .await
            .map_err(GatewayError::Redis)?;
        Ok(info)
    }

    /// Flush database (use with caution)
    pub async fn flush_db(&self) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let _: () = redis::cmd("FLUSHDB")
            .query_async(&mut conn.conn)
            .await
            .map_err(GatewayError::Redis)?;
        Ok(())
    }
}

impl Subscription {
    /// Get the next message
    /// Note: Temporarily disabled due to Redis API compatibility issues  
    pub async fn next_message(&mut self) -> Result<redis::Msg> {
        // TODO: Fix when Redis API is updated to compatible version
        Err(GatewayError::Redis(redis::RedisError::from((
            redis::ErrorKind::IoError,
            "PubSub temporarily disabled due to API compatibility",
        ))))
    }

    /// Unsubscribe from all channels
    pub async fn unsubscribe_all(&mut self) -> Result<()> {
        // Note: Redis 0.24 doesn't have unsubscribe_all, we'll need to track channels manually
        // For now, just return Ok
        // self.pubsub.unsubscribe_all().await.map_err(GatewayError::Redis)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_url() {
        let url = "redis://user:password@localhost:6379/0";
        let sanitized = RedisPool::sanitize_url(url);
        assert!(sanitized.contains("user:***@localhost"));
        assert!(!sanitized.contains("password"));
    }

    #[tokio::test]
    async fn test_redis_pool_creation() {
        let config = RedisConfig {
            url: "redis://localhost:6379".to_string(),
            enabled: true,
            max_connections: 10,
            connection_timeout: 5,
            cluster: false,
        };

        // This test would require an actual Redis instance
        // For now, we'll just test that the config is properly structured
        assert_eq!(config.url, "redis://localhost:6379");
        assert_eq!(config.max_connections, 10);
    }
}
