//! Cache types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Cache key type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheKey {
    /// Cache type
    pub cache_type: String,
    /// Key value
    pub key: String,
    /// Extra identifiers
    pub identifiers: HashMap<String, String>,
}

impl std::hash::Hash for CacheKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.cache_type.hash(state);
        self.key.hash(state);
        // Sort the HashMap keys for consistent hashing
        let mut sorted_keys: Vec<_> = self.identifiers.keys().collect();
        sorted_keys.sort();
        for k in sorted_keys {
            k.hash(state);
            self.identifiers.get(k).hash(state);
        }
    }
}

impl CacheKey {
    pub fn new(cache_type: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            cache_type: cache_type.into(),
            key: key.into(),
            identifiers: HashMap::new(),
        }
    }

    pub fn with_identifier(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.identifiers.insert(key.into(), value.into());
        self
    }
}

impl std::fmt::Display for CacheKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.cache_type, self.key)?;
        for (k, v) in &self.identifiers {
            write!(f, ":{}={}", k, v)?;
        }
        Ok(())
    }
}
