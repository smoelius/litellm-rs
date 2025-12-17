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
