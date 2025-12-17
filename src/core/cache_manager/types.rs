//! Cache manager type definitions
//!
//! This module contains all the type definitions for the cache manager,
//! including configuration, cache entries, keys, and statistics.

use crate::core::models::openai::ChatCompletionRequest;
use crate::utils::perf::strings::intern_string;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Type alias for semantic cache mapping
pub type SemanticCacheMap = HashMap<String, Vec<(CacheKey, f32)>>;

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Maximum number of entries in the cache
    pub max_entries: usize,
    /// Default TTL for cache entries
    pub default_ttl: Duration,
    /// Enable semantic caching
    pub enable_semantic: bool,
    /// Semantic similarity threshold (0.0 to 1.0)
    pub similarity_threshold: f32,
    /// Minimum prompt length for caching
    pub min_prompt_length: usize,
    /// Enable compression for large responses
    pub enable_compression: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 10000,
            default_ttl: Duration::from_secs(3600), // 1 hour
            enable_semantic: true,
            similarity_threshold: 0.95,
            min_prompt_length: 10,
            enable_compression: true,
        }
    }
}

/// Cache entry with metadata
#[derive(Debug, Clone)]
pub struct CacheEntry<T> {
    /// The cached value
    pub value: T,
    /// When the entry was created
    pub created_at: Instant,
    /// When the entry expires
    pub expires_at: Instant,
    /// Access count for popularity tracking
    pub access_count: u64,
    /// Last access time
    pub last_accessed: Instant,
    /// Size in bytes (estimated)
    pub size_bytes: usize,
}

impl<T> CacheEntry<T> {
    /// Create a new cache entry
    pub fn new(value: T, ttl: Duration, size_bytes: usize) -> Self {
        let now = Instant::now();
        Self {
            value,
            created_at: now,
            expires_at: now + ttl,
            access_count: 0,
            last_accessed: now,
            size_bytes,
        }
    }

    /// Check if the entry is expired
    pub fn is_expired(&self) -> bool {
        Instant::now() > self.expires_at
    }

    /// Mark the entry as accessed
    pub fn mark_accessed(&mut self) {
        self.access_count += 1;
        self.last_accessed = Instant::now();
    }

    /// Get the age of the entry
    pub fn age(&self) -> Duration {
        Instant::now().duration_since(self.created_at)
    }
}

/// Cache key for efficient lookups
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey {
    /// Model name (interned for efficiency)
    pub model: Arc<str>,
    /// Request hash
    pub request_hash: u64,
    /// Optional user ID for user-specific caching
    pub user_id: Option<Arc<str>>,
}

impl CacheKey {
    /// Create a new cache key from a request
    pub fn from_request(request: &ChatCompletionRequest, user_id: Option<&str>) -> Self {
        let model = intern_string(&request.model);
        let request_hash = Self::hash_request(request);
        let user_id = user_id.map(intern_string);

        Self {
            model,
            request_hash,
            user_id,
        }
    }

    /// Hash a request for cache key generation
    fn hash_request(request: &ChatCompletionRequest) -> u64 {
        use std::collections::hash_map::DefaultHasher;

        let mut hasher = DefaultHasher::new();

        // Hash the messages
        for message in &request.messages {
            message.role.hash(&mut hasher);
            if let Some(content) = &message.content {
                content.hash(&mut hasher);
            }
        }

        // Hash other relevant parameters
        request.temperature.map(|t| t.to_bits()).hash(&mut hasher);
        request.max_tokens.hash(&mut hasher);
        request.top_p.map(|p| p.to_bits()).hash(&mut hasher);
        request
            .frequency_penalty
            .map(|p| p.to_bits())
            .hash(&mut hasher);
        request
            .presence_penalty
            .map(|p| p.to_bits())
            .hash(&mut hasher);
        request.stop.hash(&mut hasher);

        hasher.finish()
    }
}

/// Atomic cache statistics for lock-free hot path updates
#[derive(Debug, Default)]
pub struct AtomicCacheStats {
    /// L1 cache hits
    pub l1_hits: AtomicU64,
    /// L1 cache misses
    pub l1_misses: AtomicU64,
    /// L2 cache hits
    pub l2_hits: AtomicU64,
    /// L2 cache misses
    pub l2_misses: AtomicU64,
    /// Semantic cache hits
    pub semantic_hits: AtomicU64,
    /// Semantic cache misses
    pub semantic_misses: AtomicU64,
    /// Cache evictions
    pub evictions: AtomicU64,
    /// Total cache size in bytes
    pub total_size_bytes: AtomicUsize,
}

/// Cache statistics snapshot (returned to callers)
#[derive(Debug, Default, Clone)]
pub struct CacheStats {
    /// L1 cache hits
    pub l1_hits: u64,
    /// L1 cache misses
    pub l1_misses: u64,
    /// L2 cache hits
    pub l2_hits: u64,
    /// L2 cache misses
    pub l2_misses: u64,
    /// Semantic cache hits
    pub semantic_hits: u64,
    /// Semantic cache misses
    pub semantic_misses: u64,
    /// Cache evictions
    pub evictions: u64,
    /// Total cache size in bytes
    pub total_size_bytes: usize,
}

impl CacheStats {
    /// Calculate hit rate
    pub fn hit_rate(&self) -> f64 {
        let total_hits = self.l1_hits + self.l2_hits + self.semantic_hits;
        let total_requests = total_hits + self.l1_misses + self.l2_misses + self.semantic_misses;

        if total_requests == 0 {
            0.0
        } else {
            total_hits as f64 / total_requests as f64
        }
    }
}

impl AtomicCacheStats {
    /// Create a snapshot of current stats
    pub fn snapshot(&self) -> CacheStats {
        CacheStats {
            l1_hits: self.l1_hits.load(Ordering::Relaxed),
            l1_misses: self.l1_misses.load(Ordering::Relaxed),
            l2_hits: self.l2_hits.load(Ordering::Relaxed),
            l2_misses: self.l2_misses.load(Ordering::Relaxed),
            semantic_hits: self.semantic_hits.load(Ordering::Relaxed),
            semantic_misses: self.semantic_misses.load(Ordering::Relaxed),
            evictions: self.evictions.load(Ordering::Relaxed),
            total_size_bytes: self.total_size_bytes.load(Ordering::Relaxed),
        }
    }

    /// Reset all stats to zero
    pub fn reset(&self) {
        self.l1_hits.store(0, Ordering::Relaxed);
        self.l1_misses.store(0, Ordering::Relaxed);
        self.l2_hits.store(0, Ordering::Relaxed);
        self.l2_misses.store(0, Ordering::Relaxed);
        self.semantic_hits.store(0, Ordering::Relaxed);
        self.semantic_misses.store(0, Ordering::Relaxed);
        self.evictions.store(0, Ordering::Relaxed);
        self.total_size_bytes.store(0, Ordering::Relaxed);
    }
}
