//! Cache metrics models

use serde::{Deserialize, Serialize};

/// Cache metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CacheMetrics {
    /// Cache hit
    pub hit: bool,
    /// Cache type
    pub cache_type: Option<String>,
    /// Cache key
    pub cache_key: Option<String>,
    /// Similarity score (for semantic cache)
    pub similarity_score: Option<f32>,
    /// Cache latency in milliseconds
    pub cache_latency_ms: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== CacheMetrics Default Tests ====================

    #[test]
    fn test_cache_metrics_default() {
        let metrics = CacheMetrics::default();
        assert!(!metrics.hit);
        assert!(metrics.cache_type.is_none());
        assert!(metrics.cache_key.is_none());
        assert!(metrics.similarity_score.is_none());
        assert!(metrics.cache_latency_ms.is_none());
    }

    // ==================== CacheMetrics Structure Tests ====================

    #[test]
    fn test_cache_metrics_hit() {
        let metrics = CacheMetrics {
            hit: true,
            cache_type: Some("exact".to_string()),
            cache_key: Some("abc123".to_string()),
            similarity_score: None,
            cache_latency_ms: Some(5),
        };
        assert!(metrics.hit);
        assert_eq!(metrics.cache_type, Some("exact".to_string()));
        assert_eq!(metrics.cache_key, Some("abc123".to_string()));
        assert_eq!(metrics.cache_latency_ms, Some(5));
    }

    #[test]
    fn test_cache_metrics_miss() {
        let metrics = CacheMetrics {
            hit: false,
            cache_type: Some("semantic".to_string()),
            cache_key: None,
            similarity_score: None,
            cache_latency_ms: Some(2),
        };
        assert!(!metrics.hit);
        assert!(metrics.cache_key.is_none());
    }

    #[test]
    fn test_cache_metrics_semantic_cache() {
        let metrics = CacheMetrics {
            hit: true,
            cache_type: Some("semantic".to_string()),
            cache_key: Some("semantic-key-xyz".to_string()),
            similarity_score: Some(0.95),
            cache_latency_ms: Some(15),
        };
        assert!(metrics.hit);
        assert_eq!(metrics.cache_type, Some("semantic".to_string()));
        assert!((metrics.similarity_score.unwrap() - 0.95).abs() < f32::EPSILON);
    }

    #[test]
    fn test_cache_metrics_high_similarity() {
        let metrics = CacheMetrics {
            hit: true,
            cache_type: Some("semantic".to_string()),
            cache_key: Some("key".to_string()),
            similarity_score: Some(0.99),
            cache_latency_ms: Some(10),
        };
        assert!(metrics.similarity_score.unwrap() > 0.9);
    }

    #[test]
    fn test_cache_metrics_low_similarity() {
        let metrics = CacheMetrics {
            hit: false,
            cache_type: Some("semantic".to_string()),
            cache_key: None,
            similarity_score: Some(0.75),
            cache_latency_ms: Some(20),
        };
        assert!(!metrics.hit);
        assert!(metrics.similarity_score.unwrap() < 0.9);
    }

    // ==================== CacheMetrics Serialization Tests ====================

    #[test]
    fn test_cache_metrics_serialization() {
        let metrics = CacheMetrics {
            hit: true,
            cache_type: Some("redis".to_string()),
            cache_key: Some("cache:key:123".to_string()),
            similarity_score: Some(0.85),
            cache_latency_ms: Some(8),
        };
        let json = serde_json::to_value(&metrics).unwrap();
        assert_eq!(json["hit"], true);
        assert_eq!(json["cache_type"], "redis");
        assert_eq!(json["cache_key"], "cache:key:123");
        assert!((json["similarity_score"].as_f64().unwrap() - 0.85).abs() < 0.001);
        assert_eq!(json["cache_latency_ms"], 8);
    }

    #[test]
    fn test_cache_metrics_serialization_minimal() {
        let metrics = CacheMetrics {
            hit: false,
            cache_type: None,
            cache_key: None,
            similarity_score: None,
            cache_latency_ms: None,
        };
        let json = serde_json::to_value(&metrics).unwrap();
        assert_eq!(json["hit"], false);
        assert!(json["cache_type"].is_null());
        assert!(json["cache_key"].is_null());
    }

    #[test]
    fn test_cache_metrics_deserialization() {
        let json = r#"{
            "hit": true,
            "cache_type": "memory",
            "cache_key": "mem-key",
            "similarity_score": 0.92,
            "cache_latency_ms": 3
        }"#;
        let metrics: CacheMetrics = serde_json::from_str(json).unwrap();
        assert!(metrics.hit);
        assert_eq!(metrics.cache_type, Some("memory".to_string()));
        assert_eq!(metrics.cache_key, Some("mem-key".to_string()));
        assert!((metrics.similarity_score.unwrap() - 0.92).abs() < f32::EPSILON);
        assert_eq!(metrics.cache_latency_ms, Some(3));
    }

    #[test]
    fn test_cache_metrics_deserialization_minimal() {
        let json = r#"{"hit": false}"#;
        let metrics: CacheMetrics = serde_json::from_str(json).unwrap();
        assert!(!metrics.hit);
        assert!(metrics.cache_type.is_none());
    }

    // ==================== CacheMetrics Clone Tests ====================

    #[test]
    fn test_cache_metrics_clone() {
        let metrics = CacheMetrics {
            hit: true,
            cache_type: Some("clone-test".to_string()),
            cache_key: Some("key".to_string()),
            similarity_score: Some(0.88),
            cache_latency_ms: Some(7),
        };
        let cloned = metrics.clone();
        assert_eq!(metrics.hit, cloned.hit);
        assert_eq!(metrics.cache_type, cloned.cache_type);
        assert_eq!(metrics.similarity_score, cloned.similarity_score);
    }
}
