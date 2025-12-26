//! Response metadata types (Provider info, Metrics, Cache info)

/// Provider information
#[derive(Debug, Clone, Default)]
pub struct ProviderInfo {
    /// Provider name
    pub name: String,
    /// Provider type
    pub provider_type: String,
    /// Model used
    pub model: String,
    /// API version
    pub api_version: Option<String>,
    /// Region
    pub region: Option<String>,
    /// Deployment ID
    pub deployment_id: Option<String>,
}

/// Response performance metrics
#[derive(Debug, Clone, Default)]
pub struct ResponseMetrics {
    /// Total response time in milliseconds
    pub total_time_ms: u64,
    /// Provider response time in milliseconds
    pub provider_time_ms: u64,
    /// Queue time in milliseconds
    pub queue_time_ms: u64,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
    /// Number of retries
    pub retry_count: u32,
    /// Whether response was cached
    pub from_cache: bool,
    /// Cache hit type
    pub cache_type: Option<String>,
}

/// Cache information
#[derive(Debug, Clone, Default)]
pub struct CacheInfo {
    /// Whether response was cached
    pub cached: bool,
    /// Cache key
    pub cache_key: Option<String>,
    /// Cache TTL
    pub ttl_seconds: Option<u64>,
    /// Cache hit/miss
    pub hit: bool,
    /// Cache type (memory, redis, semantic)
    pub cache_type: Option<String>,
    /// Semantic similarity score (for semantic cache)
    pub similarity_score: Option<f32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== ProviderInfo Tests ====================

    #[test]
    fn test_provider_info_default() {
        let info = ProviderInfo::default();
        assert!(info.name.is_empty());
        assert!(info.provider_type.is_empty());
        assert!(info.model.is_empty());
        assert!(info.api_version.is_none());
        assert!(info.region.is_none());
        assert!(info.deployment_id.is_none());
    }

    #[test]
    fn test_provider_info_structure() {
        let info = ProviderInfo {
            name: "openai-primary".to_string(),
            provider_type: "openai".to_string(),
            model: "gpt-4".to_string(),
            api_version: Some("2024-01".to_string()),
            region: Some("us-east-1".to_string()),
            deployment_id: Some("dep-123".to_string()),
        };
        assert_eq!(info.name, "openai-primary");
        assert_eq!(info.provider_type, "openai");
        assert_eq!(info.model, "gpt-4");
    }

    #[test]
    fn test_provider_info_partial() {
        let info = ProviderInfo {
            name: "anthropic".to_string(),
            provider_type: "anthropic".to_string(),
            model: "claude-3".to_string(),
            api_version: None,
            region: None,
            deployment_id: None,
        };
        assert!(info.api_version.is_none());
        assert!(info.region.is_none());
    }

    #[test]
    fn test_provider_info_clone() {
        let info = ProviderInfo {
            name: "clone-test".to_string(),
            provider_type: "test".to_string(),
            model: "model".to_string(),
            api_version: Some("v1".to_string()),
            region: None,
            deployment_id: None,
        };
        let cloned = info.clone();
        assert_eq!(info.name, cloned.name);
        assert_eq!(info.provider_type, cloned.provider_type);
        assert_eq!(info.api_version, cloned.api_version);
    }

    // ==================== ResponseMetrics Tests ====================

    #[test]
    fn test_response_metrics_default() {
        let metrics = ResponseMetrics::default();
        assert_eq!(metrics.total_time_ms, 0);
        assert_eq!(metrics.provider_time_ms, 0);
        assert_eq!(metrics.queue_time_ms, 0);
        assert_eq!(metrics.processing_time_ms, 0);
        assert_eq!(metrics.retry_count, 0);
        assert!(!metrics.from_cache);
        assert!(metrics.cache_type.is_none());
    }

    #[test]
    fn test_response_metrics_structure() {
        let metrics = ResponseMetrics {
            total_time_ms: 150,
            provider_time_ms: 100,
            queue_time_ms: 20,
            processing_time_ms: 30,
            retry_count: 1,
            from_cache: false,
            cache_type: None,
        };
        assert_eq!(metrics.total_time_ms, 150);
        assert_eq!(metrics.provider_time_ms, 100);
        assert_eq!(metrics.retry_count, 1);
    }

    #[test]
    fn test_response_metrics_from_cache() {
        let metrics = ResponseMetrics {
            total_time_ms: 5,
            provider_time_ms: 0,
            queue_time_ms: 1,
            processing_time_ms: 4,
            retry_count: 0,
            from_cache: true,
            cache_type: Some("redis".to_string()),
        };
        assert!(metrics.from_cache);
        assert_eq!(metrics.cache_type, Some("redis".to_string()));
    }

    #[test]
    fn test_response_metrics_with_retries() {
        let metrics = ResponseMetrics {
            total_time_ms: 500,
            provider_time_ms: 450,
            queue_time_ms: 10,
            processing_time_ms: 40,
            retry_count: 3,
            from_cache: false,
            cache_type: None,
        };
        assert_eq!(metrics.retry_count, 3);
    }

    #[test]
    fn test_response_metrics_clone() {
        let metrics = ResponseMetrics {
            total_time_ms: 200,
            provider_time_ms: 180,
            queue_time_ms: 5,
            processing_time_ms: 15,
            retry_count: 0,
            from_cache: true,
            cache_type: Some("memory".to_string()),
        };
        let cloned = metrics.clone();
        assert_eq!(metrics.total_time_ms, cloned.total_time_ms);
        assert_eq!(metrics.from_cache, cloned.from_cache);
        assert_eq!(metrics.cache_type, cloned.cache_type);
    }

    // ==================== CacheInfo Tests ====================

    #[test]
    fn test_cache_info_default() {
        let info = CacheInfo::default();
        assert!(!info.cached);
        assert!(info.cache_key.is_none());
        assert!(info.ttl_seconds.is_none());
        assert!(!info.hit);
        assert!(info.cache_type.is_none());
        assert!(info.similarity_score.is_none());
    }

    #[test]
    fn test_cache_info_cache_miss() {
        let info = CacheInfo {
            cached: false,
            cache_key: Some("key-123".to_string()),
            ttl_seconds: Some(3600),
            hit: false,
            cache_type: Some("redis".to_string()),
            similarity_score: None,
        };
        assert!(!info.cached);
        assert!(!info.hit);
        assert_eq!(info.cache_key, Some("key-123".to_string()));
    }

    #[test]
    fn test_cache_info_cache_hit() {
        let info = CacheInfo {
            cached: true,
            cache_key: Some("key-456".to_string()),
            ttl_seconds: Some(1800),
            hit: true,
            cache_type: Some("memory".to_string()),
            similarity_score: None,
        };
        assert!(info.cached);
        assert!(info.hit);
        assert_eq!(info.cache_type, Some("memory".to_string()));
    }

    #[test]
    fn test_cache_info_semantic_cache() {
        let info = CacheInfo {
            cached: true,
            cache_key: Some("semantic-key".to_string()),
            ttl_seconds: Some(7200),
            hit: true,
            cache_type: Some("semantic".to_string()),
            similarity_score: Some(0.95),
        };
        assert_eq!(info.cache_type, Some("semantic".to_string()));
        assert!((info.similarity_score.unwrap() - 0.95).abs() < f32::EPSILON);
    }

    #[test]
    fn test_cache_info_clone() {
        let info = CacheInfo {
            cached: true,
            cache_key: Some("clone-key".to_string()),
            ttl_seconds: Some(600),
            hit: true,
            cache_type: Some("redis".to_string()),
            similarity_score: Some(0.85),
        };
        let cloned = info.clone();
        assert_eq!(info.cached, cloned.cached);
        assert_eq!(info.cache_key, cloned.cache_key);
        assert_eq!(info.similarity_score, cloned.similarity_score);
    }

    #[test]
    fn test_cache_info_ttl_variations() {
        // Short TTL
        let short = CacheInfo {
            cached: true,
            cache_key: Some("short".to_string()),
            ttl_seconds: Some(60),
            hit: true,
            cache_type: Some("memory".to_string()),
            similarity_score: None,
        };
        assert_eq!(short.ttl_seconds, Some(60));

        // Long TTL
        let long = CacheInfo {
            cached: true,
            cache_key: Some("long".to_string()),
            ttl_seconds: Some(86400), // 1 day
            hit: true,
            cache_type: Some("redis".to_string()),
            similarity_score: None,
        };
        assert_eq!(long.ttl_seconds, Some(86400));
    }
}
