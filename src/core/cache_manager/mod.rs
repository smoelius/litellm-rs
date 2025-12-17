//! Advanced cache management with multiple strategies
//!
//! This module provides a unified cache management system with support for
//! different caching strategies including LRU, TTL, and semantic caching.

pub mod manager;
pub mod types;

#[cfg(test)]
mod tests;
