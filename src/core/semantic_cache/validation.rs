//! Validation logic for cache entries and requests

use super::types::{SemanticCacheConfig, SemanticCacheEntry};
use crate::core::models::openai::ChatCompletionRequest;

/// Check if a request should be cached
pub fn should_cache_request(config: &SemanticCacheConfig, request: &ChatCompletionRequest) -> bool {
    // Don't cache streaming requests unless explicitly enabled
    if request.stream.unwrap_or(false) && !config.enable_streaming_cache {
        return false;
    }

    // Don't cache requests with function calls (they might have side effects)
    if request.tools.is_some() || request.tool_choice.is_some() {
        return false;
    }

    // Don't cache requests with high randomness
    if let Some(temperature) = request.temperature {
        if temperature > 0.7 {
            return false;
        }
    }

    true
}

/// Check if cache entry is still valid
pub fn is_entry_valid(entry: &SemanticCacheEntry) -> bool {
    if let Some(ttl_seconds) = entry.ttl_seconds {
        let expiry_time = entry.created_at + chrono::Duration::seconds(ttl_seconds as i64);
        chrono::Utc::now() < expiry_time
    } else {
        true // No TTL means never expires
    }
}
