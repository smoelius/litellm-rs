//! Cache system trait definitions
//!
//! Provides unified cache interface supporting multiple cache backends

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Core cache trait
///
/// Defines unified cache operation interface
#[async_trait]
pub trait Cache<K, V>: Send + Sync
where
    K: Send + Sync,
    V: Send + Sync,
{
    /// Error
    type Error: std::error::Error + Send + Sync + 'static;

    /// Get
    async fn get(&self, key: &K) -> Result<Option<V>, Self::Error>;

    /// Settings
    async fn set(&self, key: &K, value: V, ttl: Duration) -> Result<(), Self::Error>;

    /// Delete
    async fn delete(&self, key: &K) -> Result<bool, Self::Error>;

    /// Check
    async fn exists(&self, key: &K) -> Result<bool, Self::Error>;

    /// Settings
    async fn expire(&self, key: &K, ttl: Duration) -> Result<bool, Self::Error>;

    /// Get
    async fn ttl(&self, key: &K) -> Result<Option<Duration>, Self::Error>;

    /// Clear all cache
    async fn clear(&self) -> Result<(), Self::Error>;

    /// Get
    async fn size(&self) -> Result<usize, Self::Error>;

    /// Get
    async fn get_many(&self, keys: &[K]) -> Result<Vec<Option<V>>, Self::Error> {
        let mut results = Vec::with_capacity(keys.len());
        for key in keys {
            results.push(self.get(key).await?);
        }
        Ok(results)
    }

    /// Settings
    async fn set_many(&self, items: &[(K, V, Duration)]) -> Result<(), Self::Error>
    where
        K: Clone,
        V: Clone,
    {
        for (key, value, ttl) in items {
            self.set(key, value.clone(), *ttl).await?;
        }
        Ok(())
    }
}

/// Cache key trait
///
/// Defines operations that cache keys must support
pub trait CacheKey: Send + Sync + Clone + std::fmt::Debug + std::hash::Hash + Eq {
    /// Serialize key to string
    fn to_cache_key(&self) -> String;

    /// Deserialize key from string
    fn from_cache_key(s: &str) -> Result<Self, CacheError>
    where
        Self: Sized;
}

/// Cache value trait
///
/// Defines operations that cache values must support
pub trait CacheValue: Send + Sync + Clone + std::fmt::Debug {
    /// Serialize to bytes
    fn to_bytes(&self) -> Result<Vec<u8>, CacheError>;

    /// Deserialize from bytes
    fn from_bytes(bytes: &[u8]) -> Result<Self, CacheError>
    where
        Self: Sized;
}

/// Implementation of CacheKey for String
impl CacheKey for String {
    fn to_cache_key(&self) -> String {
        self.clone()
    }

    fn from_cache_key(s: &str) -> Result<Self, CacheError> {
        Ok(s.to_string())
    }
}

/// Implementation of CacheValue for all types that implement Serialize + DeserializeOwned
impl<T> CacheValue for T
where
    T: Serialize + for<'de> Deserialize<'de> + Send + Sync + Clone + std::fmt::Debug,
{
    fn to_bytes(&self) -> Result<Vec<u8>, CacheError> {
        bincode::serialize(self).map_err(CacheError::Serialization)
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, CacheError> {
        bincode::deserialize(bytes).map_err(CacheError::Deserialization)
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Cache hit count
    pub hits: u64,
    /// Cache miss count
    pub misses: u64,
    /// Current key count
    pub key_count: usize,
    /// Used memory amount (bytes)
    pub memory_usage: usize,
    /// Hit rate
    pub hit_rate: f64,
}

impl CacheStats {
    pub fn new() -> Self {
        Self {
            hits: 0,
            misses: 0,
            key_count: 0,
            memory_usage: 0,
            hit_rate: 0.0,
        }
    }

    pub fn calculate_hit_rate(&mut self) {
        let total = self.hits + self.misses;
        if total > 0 {
            self.hit_rate = self.hits as f64 / total as f64;
        }
    }
}

impl Default for CacheStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache trait with statistics functionality
#[async_trait]
pub trait CacheWithStats<K, V>: Cache<K, V>
where
    K: Send + Sync,
    V: Send + Sync,
{
    /// Get
    async fn stats(&self) -> Result<CacheStats, Self::Error>;

    /// Reset statistics
    async fn reset_stats(&self) -> Result<(), Self::Error>;
}

/// Cache event types
#[derive(Debug, Clone)]
pub enum CacheEvent<K, V> {
    /// Cache hit
    Hit { key: K },
    /// Cache miss
    Miss { key: K },
    /// Settings
    Set { key: K, value: V },
    /// Delete
    Delete { key: K },
    /// Cache expiration
    Expire { key: K },
    /// Cache clear
    Clear,
}

/// Cache event listener
#[async_trait]
pub trait CacheEventListener<K, V>: Send + Sync
where
    K: Send + Sync,
    V: Send + Sync,
{
    /// Handle
    async fn on_event(&self, event: CacheEvent<K, V>);
}

/// Error
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Connection failed: {0}")]
    Connection(String),

    #[error("Serialization failed: {0}")]
    Serialization(#[from] Box<bincode::ErrorKind>),

    #[error("Deserialization failed: {0}")]
    Deserialization(Box<bincode::ErrorKind>),

    #[error("Key not found: {key}")]
    KeyNotFound { key: String },

    #[error("Cache is full")]
    CacheFull,

    #[error("Invalid TTL: {ttl_ms}ms")]
    InvalidTTL { ttl_ms: u64 },

    #[error("Cache operation timeout")]
    Timeout,

    #[error("Cache backend error: {0}")]
    Backend(String),

    #[error("Other cache error: {0}")]
    Other(String),
}

impl CacheError {
    pub fn connection(msg: impl Into<String>) -> Self {
        Self::Connection(msg.into())
    }

    pub fn key_not_found(key: impl Into<String>) -> Self {
        Self::KeyNotFound { key: key.into() }
    }

    pub fn backend(msg: impl Into<String>) -> Self {
        Self::Backend(msg.into())
    }

    pub fn other(msg: impl Into<String>) -> Self {
        Self::Other(msg.into())
    }
}
