//! Cache configuration

use super::*;
use serde::{Deserialize, Serialize};

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Enable caching
    #[serde(default)]
    pub enabled: bool,
    /// Cache TTL in seconds
    #[serde(default = "default_cache_ttl")]
    pub ttl: u64,
    /// Maximum cache size
    #[serde(default = "default_cache_max_size")]
    pub max_size: usize,
    /// Enable semantic caching
    #[serde(default)]
    pub semantic_cache: bool,
    /// Similarity threshold for semantic cache
    #[serde(default = "default_similarity_threshold")]
    pub similarity_threshold: f64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            ttl: default_cache_ttl(),
            max_size: default_cache_max_size(),
            semantic_cache: false,
            similarity_threshold: default_similarity_threshold(),
        }
    }
}

#[allow(dead_code)]
impl CacheConfig {
    /// Merge cache configurations
    pub fn merge(mut self, other: Self) -> Self {
        if other.enabled {
            self.enabled = other.enabled;
        }
        if other.ttl != default_cache_ttl() {
            self.ttl = other.ttl;
        }
        if other.max_size != default_cache_max_size() {
            self.max_size = other.max_size;
        }
        if other.semantic_cache {
            self.semantic_cache = other.semantic_cache;
        }
        if other.similarity_threshold != default_similarity_threshold() {
            self.similarity_threshold = other.similarity_threshold;
        }
        self
    }
}
