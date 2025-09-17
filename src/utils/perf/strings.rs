//! String pool for efficient string management
//!
//! This module provides utilities to reduce string allocations and cloning
//! by using string interning and reference counting.

use dashmap::DashMap;
use std::borrow::Cow;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

/// A thread-safe string pool for interning commonly used strings
///
/// This implementation uses a hash-based approach to reduce memory usage
/// and improve lookup performance for frequently used strings.
pub struct StringPool {
    pool: DashMap<u64, Arc<str>>,
    stats: DashMap<u64, usize>, // Track usage frequency
}

impl StringPool {
    /// Create a new string pool
    pub fn new() -> Self {
        Self {
            pool: DashMap::new(),
            stats: DashMap::new(),
        }
    }

    /// Hash a string for use as a key
    fn hash_string(s: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        hasher.finish()
    }

    /// Intern a string, returning an Arc<str> for efficient sharing
    pub fn intern(&self, s: &str) -> Arc<str> {
        let hash = Self::hash_string(s);

        if let Some(interned) = self.pool.get(&hash) {
            // Update usage statistics
            self.stats
                .entry(hash)
                .and_modify(|count| *count += 1)
                .or_insert(1);
            interned.clone()
        } else {
            let arc_str: Arc<str> = Arc::from(s);
            self.pool.insert(hash, arc_str.clone());
            self.stats.insert(hash, 1);
            arc_str
        }
    }

    /// Get an interned string if it exists, otherwise return None
    #[allow(dead_code)] // Reserved for future string pool operations
    pub fn get(&self, s: &str) -> Option<Arc<str>> {
        let hash = Self::hash_string(s);
        self.pool.get(&hash).map(|v| v.clone())
    }

    /// Get usage statistics for a string
    #[allow(dead_code)] // May be used for debugging/monitoring
    pub fn get_usage_count(&self, s: &str) -> usize {
        let hash = Self::hash_string(s);
        self.stats.get(&hash).map(|count| *count).unwrap_or(0)
    }

    /// Get the most frequently used strings
    #[allow(dead_code)] // May be used for debugging/monitoring
    pub fn get_top_strings(&self, limit: usize) -> Vec<(Arc<str>, usize)> {
        let mut entries: Vec<_> = self
            .pool
            .iter()
            .filter_map(|entry| {
                let hash = *entry.key();
                let arc_str = entry.value().clone();
                let count = self.stats.get(&hash).map(|c| *c).unwrap_or(0);
                if count > 0 {
                    Some((arc_str, count))
                } else {
                    None
                }
            })
            .collect();

        entries.sort_by(|a, b| b.1.cmp(&a.1));
        entries.truncate(limit);
        entries
    }

    /// Clear the pool
    #[allow(dead_code)] // Reserved for future string pool operations
    pub fn clear(&self) {
        self.pool.clear();
    }

    /// Get the number of interned strings
    #[allow(dead_code)] // Reserved for future string pool operations
    pub fn len(&self) -> usize {
        self.pool.len()
    }

    /// Check if the pool is empty
    #[allow(dead_code)] // Reserved for future string pool operations
    pub fn is_empty(&self) -> bool {
        self.pool.is_empty()
    }
}

impl Default for StringPool {
    fn default() -> Self {
        Self::new()
    }
}

/// A smart string type that can be either borrowed or owned
#[derive(Debug, Clone)]
pub enum SmartString {
    /// Borrowed string reference
    Borrowed(&'static str),
    /// Owned string
    Owned(String),
    /// Reference-counted string
    Shared(Arc<str>),
}

impl SmartString {
    /// Create a new SmartString from a static string
    #[allow(dead_code)] // Reserved for future smart string operations
    pub fn from_static(s: &'static str) -> Self {
        Self::Borrowed(s)
    }

    /// Create a new SmartString from an owned string
    #[allow(dead_code)] // Reserved for future smart string operations
    pub fn from_owned(s: String) -> Self {
        Self::Owned(s)
    }

    /// Create a new SmartString from an Arc<str>
    #[allow(dead_code)] // Reserved for future smart string operations
    pub fn from_shared(s: Arc<str>) -> Self {
        Self::Shared(s)
    }

    /// Get the string as a &str
    pub fn as_str(&self) -> &str {
        match self {
            Self::Borrowed(s) => s,
            Self::Owned(s) => s.as_str(),
            Self::Shared(s) => s.as_ref(),
        }
    }

    /// Convert to a Cow<str> for efficient string operations
    #[allow(dead_code)] // Reserved for future smart string operations
    pub fn as_cow(&self) -> Cow<'_, str> {
        match self {
            Self::Borrowed(s) => Cow::Borrowed(s),
            Self::Owned(s) => Cow::Borrowed(s.as_str()),
            Self::Shared(s) => Cow::Borrowed(s.as_ref()),
        }
    }

    /// Convert to an owned String
    #[allow(dead_code)] // Reserved for future smart string operations
    pub fn into_string(self) -> String {
        match self {
            Self::Borrowed(s) => s.to_string(),
            Self::Owned(s) => s,
            Self::Shared(s) => s.to_string(),
        }
    }

    /// Get the length of the string
    #[allow(dead_code)] // Reserved for future smart string operations
    pub fn len(&self) -> usize {
        self.as_str().len()
    }

    /// Check if the string is empty
    #[allow(dead_code)] // Reserved for future smart string operations
    pub fn is_empty(&self) -> bool {
        self.as_str().is_empty()
    }
}

impl From<&'static str> for SmartString {
    fn from(s: &'static str) -> Self {
        Self::Borrowed(s)
    }
}

impl From<String> for SmartString {
    fn from(s: String) -> Self {
        Self::Owned(s)
    }
}

impl From<Arc<str>> for SmartString {
    fn from(s: Arc<str>) -> Self {
        Self::Shared(s)
    }
}

impl AsRef<str> for SmartString {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::fmt::Display for SmartString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl PartialEq for SmartString {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl PartialEq<str> for SmartString {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<&str> for SmartString {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<String> for SmartString {
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other.as_str()
    }
}

/// Global string pool instance
static STRING_POOL: once_cell::sync::Lazy<StringPool> = once_cell::sync::Lazy::new(StringPool::new);

/// Intern a string using the global string pool
pub fn intern_string(s: &str) -> Arc<str> {
    STRING_POOL.intern(s)
}

/// Get an interned string from the global pool
#[allow(dead_code)] // Reserved for future global string pool operations
pub fn get_interned_string(s: &str) -> Option<Arc<str>> {
    STRING_POOL.get(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_pool() {
        let pool = StringPool::new();

        let s1 = pool.intern("hello");
        let s2 = pool.intern("hello");
        let s3 = pool.intern("world");

        // Same string should return the same Arc
        assert!(Arc::ptr_eq(&s1, &s2));
        assert!(!Arc::ptr_eq(&s1, &s3));

        assert_eq!(pool.len(), 2);
    }

    #[test]
    fn test_smart_string() {
        let static_str = SmartString::from_static("static");
        let owned_str = SmartString::from_owned("owned".to_string());
        let shared_str = SmartString::from_shared(Arc::from("shared"));

        assert_eq!(static_str.as_str(), "static");
        assert_eq!(owned_str.as_str(), "owned");
        assert_eq!(shared_str.as_str(), "shared");

        assert_eq!(static_str.len(), 6);
        assert!(!owned_str.is_empty());
    }

    #[test]
    fn test_smart_string_equality() {
        let s1 = SmartString::from_static("test");
        let s2 = SmartString::from_owned("test".to_string());
        let s3 = SmartString::from_shared(Arc::from("test"));

        assert_eq!(s1, s2);
        assert_eq!(s2, s3);
        assert_eq!(s1, "test");
        assert_eq!(s2, "test".to_string());
    }

    #[test]
    fn test_global_string_pool() {
        let s1 = intern_string("global_test");
        let s2 = intern_string("global_test");

        assert!(Arc::ptr_eq(&s1, &s2));

        let s3 = get_interned_string("global_test");
        assert!(s3.is_some());
        assert!(Arc::ptr_eq(&s1, &s3.unwrap()));

        let s4 = get_interned_string("not_found");
        assert!(s4.is_none());
    }
}
