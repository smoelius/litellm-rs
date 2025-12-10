//! Advanced cache management with multiple strategies
//!
//! This module provides a unified cache management system with support for
//! different caching strategies including LRU, TTL, and semantic caching.

use crate::core::models::openai::{ChatCompletionRequest, ChatCompletionResponse};
use crate::utils::error::Result;
use crate::utils::perf::strings::intern_string;
use dashmap::DashMap;
use lru::LruCache;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use tracing::{debug, info};

/// Type alias for semantic cache mapping
type SemanticCacheMap = HashMap<String, Vec<(CacheKey, f32)>>;

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

/// Multi-tier cache manager
pub struct CacheManager {
    /// L1 cache: In-memory LRU cache for hot data
    l1_cache: Arc<RwLock<LruCache<CacheKey, CacheEntry<ChatCompletionResponse>>>>,
    /// L2 cache: Larger capacity with TTL
    l2_cache: Arc<DashMap<CacheKey, CacheEntry<ChatCompletionResponse>>>,
    /// Semantic cache for similar queries
    semantic_cache: Arc<RwLock<SemanticCacheMap>>,
    /// Cache configuration
    config: CacheConfig,
    /// Cache statistics (lock-free atomics for hot path)
    stats: Arc<AtomicCacheStats>,
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

impl CacheManager {
    /// Create a new cache manager
    pub fn new(config: CacheConfig) -> Result<Self> {
        // Ensure we have a reasonable minimum capacity
        let l1_capacity = NonZeroUsize::new(config.max_entries / 10)
            .or_else(|| NonZeroUsize::new(100))
            .ok_or_else(|| {
                crate::utils::error::GatewayError::Config(
                    "Invalid cache configuration: max_entries must be greater than 0".to_string(),
                )
            })?;

        Ok(Self {
            l1_cache: Arc::new(RwLock::new(LruCache::new(l1_capacity))),
            l2_cache: Arc::new(DashMap::new()),
            semantic_cache: Arc::new(RwLock::new(HashMap::new())),
            config,
            stats: Arc::new(AtomicCacheStats::default()),
        })
    }

    /// Get a cached response
    pub async fn get(&self, key: &CacheKey) -> Result<Option<ChatCompletionResponse>> {
        // Try L1 cache first
        {
            let mut l1 = self.l1_cache.write();
            if let Some(entry) = l1.get_mut(key) {
                if !entry.is_expired() {
                    entry.mark_accessed();
                    self.stats.l1_hits.fetch_add(1, Ordering::Relaxed);
                    debug!("L1 cache hit for key: {:?}", key);
                    return Ok(Some(entry.value.clone()));
                } else {
                    l1.pop(key);
                }
            }
        }

        self.stats.l1_misses.fetch_add(1, Ordering::Relaxed);

        // Try L2 cache
        if let Some(mut entry) = self.l2_cache.get_mut(key) {
            if !entry.is_expired() {
                entry.mark_accessed();

                // Promote to L1 cache
                let mut l1 = self.l1_cache.write();
                l1.put(key.clone(), entry.clone());

                self.stats.l2_hits.fetch_add(1, Ordering::Relaxed);
                debug!("L2 cache hit for key: {:?}", key);
                return Ok(Some(entry.value.clone()));
            } else {
                self.l2_cache.remove(key);
            }
        }

        self.stats.l2_misses.fetch_add(1, Ordering::Relaxed);

        // Try semantic cache if enabled
        if self.config.enable_semantic {
            if let Some(response) = self.semantic_lookup(key).await? {
                self.stats.semantic_hits.fetch_add(1, Ordering::Relaxed);
                debug!("Semantic cache hit for key: {:?}", key);
                return Ok(Some(response));
            }
        }

        self.stats.semantic_misses.fetch_add(1, Ordering::Relaxed);
        Ok(None)
    }

    /// Store a response in the cache
    pub async fn put(&self, key: CacheKey, response: ChatCompletionResponse) -> Result<()> {
        let size_bytes = self.estimate_size(&response);
        let entry = CacheEntry::new(response, self.config.default_ttl, size_bytes);

        // Store in L2 cache
        self.l2_cache.insert(key.clone(), entry.clone());

        // Update semantic cache if enabled
        if self.config.enable_semantic {
            self.update_semantic_cache(&key).await?;
        }

        // Update statistics (lock-free)
        self.stats
            .total_size_bytes
            .fetch_add(size_bytes, Ordering::Relaxed);

        // Cleanup expired entries periodically
        if self.l2_cache.len() % 1000 == 0 {
            self.cleanup_expired().await;
        }

        debug!("Cached response for key: {:?}", key);
        Ok(())
    }

    /// Semantic cache lookup
    async fn semantic_lookup(&self, _key: &CacheKey) -> Result<Option<ChatCompletionResponse>> {
        // TODO: Implement semantic similarity search
        // This would involve:
        // 1. Extract embeddings from the request
        // 2. Compare with cached embeddings
        // 3. Return similar cached responses if similarity > threshold
        Ok(None)
    }

    /// Update semantic cache
    async fn update_semantic_cache(&self, _key: &CacheKey) -> Result<()> {
        // TODO: Implement semantic cache updates
        // This would involve:
        // 1. Generate embeddings for the request
        // 2. Store in semantic index
        Ok(())
    }

    /// Estimate the size of a response in bytes
    fn estimate_size(&self, response: &ChatCompletionResponse) -> usize {
        // Rough estimation based on JSON serialization
        serde_json::to_string(response)
            .map(|s| s.len())
            .unwrap_or(1024) // Default estimate
    }

    /// Clean up expired entries
    async fn cleanup_expired(&self) {
        let mut removed_count = 0u64;
        let mut removed_size = 0usize;

        // Clean L2 cache
        self.l2_cache.retain(|_, entry| {
            if entry.is_expired() {
                removed_count += 1;
                removed_size += entry.size_bytes;
                false
            } else {
                true
            }
        });

        // Update statistics (lock-free)
        if removed_count > 0 {
            self.stats
                .evictions
                .fetch_add(removed_count, Ordering::Relaxed);
            // Use saturating subtraction for size
            let current_size = self.stats.total_size_bytes.load(Ordering::Relaxed);
            self.stats
                .total_size_bytes
                .store(current_size.saturating_sub(removed_size), Ordering::Relaxed);

            info!(
                "Cleaned up {} expired cache entries, freed {} bytes",
                removed_count, removed_size
            );
        }
    }

    /// Get cache statistics (lock-free snapshot)
    pub fn stats(&self) -> CacheStats {
        self.stats.snapshot()
    }

    /// Clear all caches
    pub async fn clear(&self) {
        self.l1_cache.write().clear();
        self.l2_cache.clear();
        self.semantic_cache.write().clear();

        // Reset atomic stats
        self.stats.reset();

        info!("All caches cleared");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::openai::*;

    #[tokio::test]
    async fn test_cache_manager() -> Result<()> {
        let config = CacheConfig::default();
        let cache = CacheManager::new(config)?;

        let request = ChatCompletionRequest {
            model: "gpt-4".to_string(),
            messages: vec![ChatMessage {
                role: MessageRole::User,
                content: Some(MessageContent::Text("Hello".to_string())),
                name: None,
                function_call: None,
                tool_calls: None,
                tool_call_id: None,
                audio: None,
            }],
            ..Default::default()
        };

        let key = CacheKey::from_request(&request, None);

        // Should be empty initially
        let initial_result = cache.get(&key).await?;
        assert!(initial_result.is_none());

        // Store a response
        let response = ChatCompletionResponse {
            id: "test".to_string(),
            object: "chat.completion".to_string(),
            created: 1234567890,
            model: "gpt-4".to_string(),
            choices: vec![],
            usage: None,
            system_fingerprint: None,
        };

        cache.put(key.clone(), response.clone()).await?;

        // Should find the cached response
        let cached = cache.get(&key).await?;
        assert!(cached.is_some());
        if let Some(cached_response) = cached {
            assert_eq!(cached_response.id, response.id);
        }

        Ok(())
    }

    #[test]
    fn test_cache_key_generation() {
        let request1 = ChatCompletionRequest {
            model: "gpt-4".to_string(),
            messages: vec![ChatMessage {
                role: MessageRole::User,
                content: Some(MessageContent::Text("Hello".to_string())),
                name: None,
                function_call: None,
                tool_calls: None,
                tool_call_id: None,
                audio: None,
            }],
            ..Default::default()
        };

        let request2 = ChatCompletionRequest {
            model: "gpt-4".to_string(),
            messages: vec![ChatMessage {
                role: MessageRole::User,
                content: Some(MessageContent::Text("Hello".to_string())),
                name: None,
                function_call: None,
                tool_calls: None,
                tool_call_id: None,
                audio: None,
            }],
            ..Default::default()
        };

        let key1 = CacheKey::from_request(&request1, None);
        let key2 = CacheKey::from_request(&request2, None);

        assert_eq!(key1, key2);
    }
}
