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
