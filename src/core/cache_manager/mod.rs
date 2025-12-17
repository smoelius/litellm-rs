//! Advanced cache management with multiple strategies
//!
//! This module provides a unified cache management system with support for
//! different caching strategies including LRU, TTL, and semantic caching.

mod manager;
mod types;

#[cfg(test)]
mod tests;

// Re-export all public types for backward compatibility
pub use manager::CacheManager;
pub use types::{
    AtomicCacheStats, CacheConfig, CacheEntry, CacheKey, CacheStats, SemanticCacheMap,
};
