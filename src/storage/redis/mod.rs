//! Redis storage implementation
//!
//! This module provides Redis connectivity and caching operations.
//!
//! ## Module Structure
//!
//! - `pool` - Connection pool and core connection management
//! - `cache` - Basic cache operations (get, set, delete, exists, expire, ttl)
//! - `batch` - Batch operations (mget, mset)
//! - `collections` - List and Set operations
//! - `hash` - Hash and Sorted Set operations
//! - `pubsub` - Pub/Sub operations (temporarily disabled)
//! - `atomic` - Atomic operations and utilities
//! - `tests` - Module tests

#![allow(dead_code)]

// Module declarations
mod pool;
mod cache;
mod batch;
mod collections;
mod hash;
mod pubsub;
mod atomic;
#[cfg(test)]
mod tests;

// Re-export public types
pub use pool::{RedisPool, RedisConnection};
pub use pubsub::Subscription;
