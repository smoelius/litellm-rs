//! Storage configuration

use super::*;
use super::{default_connection_timeout, default_redis_max_connections};
use serde::{Deserialize, Serialize};

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StorageConfig {
    /// Database configuration
    pub database: DatabaseConfig,
    /// Redis configuration
    pub redis: RedisConfig,
}

#[allow(dead_code)]
impl StorageConfig {
    /// Merge storage configurations
    pub fn merge(mut self, other: Self) -> Self {
        self.database = self.database.merge(other.database);
        self.redis = self.redis.merge(other.redis);
        self
    }
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database URL
    pub url: String,
    /// Maximum connections
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
    /// Connection timeout in seconds
    #[serde(default = "default_connection_timeout")]
    pub connection_timeout: u64,
    /// Enable SSL
    #[serde(default)]
    pub ssl: bool,
    /// Enable database (if false, use in-memory storage)
    #[serde(default)]
    pub enabled: bool,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "postgresql://localhost/litellm".to_string(),
            max_connections: default_max_connections(),
            connection_timeout: default_connection_timeout(),
            ssl: false,
            enabled: false,
        }
    }
}

#[allow(dead_code)]
impl DatabaseConfig {
    /// Merge database configurations
    pub fn merge(mut self, other: Self) -> Self {
        if !other.url.is_empty() && other.url != "postgresql://localhost/litellm" {
            self.url = other.url;
        }
        if other.max_connections != default_max_connections() {
            self.max_connections = other.max_connections;
        }
        if other.connection_timeout != default_connection_timeout() {
            self.connection_timeout = other.connection_timeout;
        }
        if other.ssl {
            self.ssl = other.ssl;
        }
        self
    }
}

/// Redis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    /// Redis URL
    pub url: String,
    /// Enable Redis (if false, use in-memory cache)
    #[serde(default = "default_redis_enabled")]
    pub enabled: bool,
    /// Maximum connections
    #[serde(default = "default_redis_max_connections")]
    pub max_connections: u32,
    /// Connection timeout in seconds
    #[serde(default = "default_connection_timeout")]
    pub connection_timeout: u64,
    /// Enable cluster mode
    #[serde(default)]
    pub cluster: bool,
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            url: "redis://localhost:6379".to_string(),
            enabled: default_redis_enabled(),
            max_connections: default_redis_max_connections(),
            connection_timeout: default_connection_timeout(),
            cluster: false,
        }
    }
}

#[allow(dead_code)]
impl RedisConfig {
    /// Merge Redis configurations
    pub fn merge(mut self, other: Self) -> Self {
        if !other.url.is_empty() && other.url != "redis://localhost:6379" {
            self.url = other.url;
        }
        if other.max_connections != default_redis_max_connections() {
            self.max_connections = other.max_connections;
        }
        if other.connection_timeout != default_connection_timeout() {
            self.connection_timeout = other.connection_timeout;
        }
        if other.cluster {
            self.cluster = other.cluster;
        }
        self
    }
}

fn default_redis_enabled() -> bool {
    true
}
