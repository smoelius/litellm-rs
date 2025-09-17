//! cache系统 trait 定义
//!
//! 提供统一的cache接口，支持多种cache后端

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// cache核心 trait
///
/// 定义统一的cache操作接口
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

    /// 清空所有cache
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

/// cache键 trait
///
/// 定义cache键必须支持的操作
pub trait CacheKey: Send + Sync + Clone + std::fmt::Debug + std::hash::Hash + Eq {
    /// 将键序列化为字符串
    fn to_cache_key(&self) -> String;

    /// 从字符串反序列化键
    fn from_cache_key(s: &str) -> Result<Self, CacheError>
    where
        Self: Sized;
}

/// cache值 trait
///
/// 定义cache值必须支持的操作
pub trait CacheValue: Send + Sync + Clone + std::fmt::Debug {
    /// 序列化为字节
    fn to_bytes(&self) -> Result<Vec<u8>, CacheError>;

    /// 从字节反序列化
    fn from_bytes(bytes: &[u8]) -> Result<Self, CacheError>
    where
        Self: Sized;
}

/// 为 String implementation CacheKey
impl CacheKey for String {
    fn to_cache_key(&self) -> String {
        self.clone()
    }

    fn from_cache_key(s: &str) -> Result<Self, CacheError> {
        Ok(s.to_string())
    }
}

/// 为所有implementation了 Serialize + DeserializeOwned 的类型implementation CacheValue
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

/// cache统计信息
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// cache命中次数
    pub hits: u64,
    /// cache未命中次数
    pub misses: u64,
    /// current键的count
    pub key_count: usize,
    /// usage的内存量（字节）
    pub memory_usage: usize,
    /// 命中率
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

/// 带统计功能的cache trait
#[async_trait]
pub trait CacheWithStats<K, V>: Cache<K, V>
where
    K: Send + Sync,
    V: Send + Sync,
{
    /// Get
    async fn stats(&self) -> Result<CacheStats, Self::Error>;

    /// 重置统计信息
    async fn reset_stats(&self) -> Result<(), Self::Error>;
}

/// cache事件类型
#[derive(Debug, Clone)]
pub enum CacheEvent<K, V> {
    /// cache命中
    Hit { key: K },
    /// cache未命中
    Miss { key: K },
    /// Settings
    Set { key: K, value: V },
    /// Delete
    Delete { key: K },
    /// cache过期
    Expire { key: K },
    /// cache清空
    Clear,
}

/// cache事件监听器
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
