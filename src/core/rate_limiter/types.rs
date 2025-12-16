//! Rate limiter types and data structures

use std::time::Instant;

/// Rate limit result
#[derive(Debug, Clone)]
pub struct RateLimitResult {
    /// Whether the request is allowed
    pub allowed: bool,
    /// Current request count in the window
    pub current_count: u32,
    /// Maximum requests allowed
    pub limit: u32,
    /// Remaining requests in the window
    pub remaining: u32,
    /// Time until the window resets (in seconds)
    pub reset_after_secs: u64,
    /// Retry after (in seconds, only set when not allowed)
    pub retry_after_secs: Option<u64>,
}

/// Rate limit entry for tracking request counts
#[derive(Debug, Clone)]
pub(super) struct RateLimitEntry {
    /// Request timestamps for sliding window
    pub(super) timestamps: Vec<Instant>,
    /// Token count for token bucket
    pub(super) tokens: f64,
    /// Last token refill time
    pub(super) last_refill: Instant,
}

impl Default for RateLimitEntry {
    fn default() -> Self {
        Self {
            timestamps: Vec::new(),
            tokens: 0.0,
            last_refill: Instant::now(),
        }
    }
}
