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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    // ==================== CacheKey Construction Tests ====================

    #[test]
    fn test_cache_key_new() {
        let key = CacheKey::new("completion", "abc123");

        assert_eq!(key.cache_type, "completion");
        assert_eq!(key.key, "abc123");
        assert!(key.identifiers.is_empty());
    }

    #[test]
    fn test_cache_key_new_with_string() {
        let key = CacheKey::new(String::from("embedding"), String::from("xyz789"));

        assert_eq!(key.cache_type, "embedding");
        assert_eq!(key.key, "xyz789");
    }

    #[test]
    fn test_cache_key_with_identifier() {
        let key = CacheKey::new("completion", "key1").with_identifier("model", "gpt-4");

        assert_eq!(key.identifiers.get("model"), Some(&"gpt-4".to_string()));
    }

    #[test]
    fn test_cache_key_with_multiple_identifiers() {
        let key = CacheKey::new("completion", "key1")
            .with_identifier("model", "gpt-4")
            .with_identifier("temperature", "0.7")
            .with_identifier("max_tokens", "100");

        assert_eq!(key.identifiers.len(), 3);
        assert_eq!(key.identifiers.get("model"), Some(&"gpt-4".to_string()));
        assert_eq!(
            key.identifiers.get("temperature"),
            Some(&"0.7".to_string())
        );
        assert_eq!(
            key.identifiers.get("max_tokens"),
            Some(&"100".to_string())
        );
    }

    #[test]
    fn test_cache_key_identifier_override() {
        let key = CacheKey::new("test", "key")
            .with_identifier("field", "value1")
            .with_identifier("field", "value2");

        assert_eq!(key.identifiers.get("field"), Some(&"value2".to_string()));
    }

    // ==================== Display Tests ====================

    #[test]
    fn test_cache_key_display_simple() {
        let key = CacheKey::new("type", "key");
        let display = format!("{}", key);

        assert_eq!(display, "type:key");
    }

    #[test]
    fn test_cache_key_display_with_identifiers() {
        let key = CacheKey::new("cache", "id").with_identifier("version", "1");

        let display = format!("{}", key);
        assert!(display.starts_with("cache:id"));
        assert!(display.contains("version=1"));
    }

    // ==================== Hash Tests ====================

    fn calculate_hash<T: Hash>(t: &T) -> u64 {
        let mut hasher = DefaultHasher::new();
        t.hash(&mut hasher);
        hasher.finish()
    }

    #[test]
    fn test_cache_key_hash_same_keys() {
        let key1 = CacheKey::new("type", "key");
        let key2 = CacheKey::new("type", "key");

        assert_eq!(calculate_hash(&key1), calculate_hash(&key2));
    }

    #[test]
    fn test_cache_key_hash_different_keys() {
        let key1 = CacheKey::new("type1", "key");
        let key2 = CacheKey::new("type2", "key");

        assert_ne!(calculate_hash(&key1), calculate_hash(&key2));
    }

    #[test]
    fn test_cache_key_hash_with_identifiers_order_independent() {
        // Identifiers should be sorted for consistent hashing
        let key1 = CacheKey::new("type", "key")
            .with_identifier("a", "1")
            .with_identifier("b", "2");

        let key2 = CacheKey::new("type", "key")
            .with_identifier("b", "2")
            .with_identifier("a", "1");

        assert_eq!(calculate_hash(&key1), calculate_hash(&key2));
    }

    #[test]
    fn test_cache_key_hash_different_identifiers() {
        let key1 = CacheKey::new("type", "key").with_identifier("a", "1");

        let key2 = CacheKey::new("type", "key").with_identifier("a", "2");

        assert_ne!(calculate_hash(&key1), calculate_hash(&key2));
    }

    // ==================== Equality Tests ====================

    #[test]
    fn test_cache_key_equality_same() {
        let key1 = CacheKey::new("type", "key");
        let key2 = CacheKey::new("type", "key");

        assert_eq!(key1, key2);
    }

    #[test]
    fn test_cache_key_equality_different_type() {
        let key1 = CacheKey::new("type1", "key");
        let key2 = CacheKey::new("type2", "key");

        assert_ne!(key1, key2);
    }

    #[test]
    fn test_cache_key_equality_different_key() {
        let key1 = CacheKey::new("type", "key1");
        let key2 = CacheKey::new("type", "key2");

        assert_ne!(key1, key2);
    }

    #[test]
    fn test_cache_key_equality_with_identifiers() {
        let key1 = CacheKey::new("type", "key").with_identifier("a", "1");

        let key2 = CacheKey::new("type", "key").with_identifier("a", "1");

        assert_eq!(key1, key2);
    }

    #[test]
    fn test_cache_key_equality_different_identifiers() {
        let key1 = CacheKey::new("type", "key").with_identifier("a", "1");

        let key2 = CacheKey::new("type", "key").with_identifier("a", "2");

        assert_ne!(key1, key2);
    }

    // ==================== Serialization Tests ====================

    #[test]
    fn test_cache_key_serialization() {
        let key = CacheKey::new("completion", "abc123").with_identifier("model", "gpt-4");

        let json = serde_json::to_value(&key).unwrap();
        assert_eq!(json["cache_type"], "completion");
        assert_eq!(json["key"], "abc123");
        assert_eq!(json["identifiers"]["model"], "gpt-4");
    }

    #[test]
    fn test_cache_key_deserialization() {
        let json = r#"{
            "cache_type": "embedding",
            "key": "xyz789",
            "identifiers": {"dim": "1536"}
        }"#;

        let key: CacheKey = serde_json::from_str(json).unwrap();
        assert_eq!(key.cache_type, "embedding");
        assert_eq!(key.key, "xyz789");
        assert_eq!(key.identifiers.get("dim"), Some(&"1536".to_string()));
    }

    #[test]
    fn test_cache_key_serialization_roundtrip() {
        let original = CacheKey::new("test", "key")
            .with_identifier("a", "1")
            .with_identifier("b", "2");

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: CacheKey = serde_json::from_str(&json).unwrap();

        assert_eq!(original, deserialized);
    }

    // ==================== Clone Tests ====================

    #[test]
    fn test_cache_key_clone() {
        let original = CacheKey::new("type", "key").with_identifier("field", "value");

        let cloned = original.clone();

        assert_eq!(original.cache_type, cloned.cache_type);
        assert_eq!(original.key, cloned.key);
        assert_eq!(original.identifiers, cloned.identifiers);
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_cache_key_empty_strings() {
        let key = CacheKey::new("", "");

        assert_eq!(key.cache_type, "");
        assert_eq!(key.key, "");
    }

    #[test]
    fn test_cache_key_special_characters() {
        let key = CacheKey::new("cache:type", "key/with/slashes")
            .with_identifier("field=value", "data:with:colons");

        assert_eq!(key.cache_type, "cache:type");
        assert_eq!(key.key, "key/with/slashes");
    }

    #[test]
    fn test_cache_key_unicode() {
        let key = CacheKey::new("缓存", "键值").with_identifier("标识符", "值");

        assert_eq!(key.cache_type, "缓存");
        assert_eq!(key.key, "键值");
    }

    #[test]
    fn test_cache_key_long_values() {
        let long_key = "a".repeat(1000);
        let key = CacheKey::new("type", &long_key);

        assert_eq!(key.key.len(), 1000);
    }
}
