//! Rate limiting types and data structures
//!
//! This module defines the core types used for rate limiting functionality.

use std::time::{Duration, Instant};
use uuid::Uuid;

/// Rate limit configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Requests per minute
    pub rpm: Option<u32>,
    /// Tokens per minute
    pub tpm: Option<u32>,
    /// Requests per day
    pub rpd: Option<u32>,
    /// Tokens per day
    pub tpd: Option<u32>,
    /// Concurrent requests
    pub concurrent: Option<u32>,
    /// Burst allowance
    pub burst: Option<u32>,
}

/// Token bucket for rate limiting
#[derive(Debug, Clone)]
pub struct TokenBucket {
    /// Current token count
    pub tokens: f64,
    /// Maximum tokens (capacity)
    pub capacity: f64,
    /// Refill rate (tokens per second)
    pub refill_rate: f64,
    /// Last refill time
    pub last_refill: Instant,
}

/// Sliding window for request counting
#[derive(Debug, Clone)]
pub struct SlidingWindow {
    /// Window size in seconds
    pub window_size: Duration,
    /// Request timestamps
    pub requests: Vec<Instant>,
    /// Token counts with timestamps
    pub tokens: Vec<(Instant, u32)>,
}

/// Rate limit check result
#[derive(Debug, Clone)]
pub struct RateLimitResult {
    /// Whether the request is allowed
    pub allowed: bool,
    /// Remaining requests in current window
    pub remaining_requests: Option<u32>,
    /// Remaining tokens in current window
    pub remaining_tokens: Option<u32>,
    /// Time until reset
    pub reset_time: Option<Duration>,
    /// Retry after duration
    pub retry_after: Option<Duration>,
    /// Rate limit type that was hit
    pub limit_type: Option<String>,
}

/// Rate limit key components
#[derive(Debug, Clone)]
pub struct RateLimitKey {
    /// User ID
    pub user_id: Option<Uuid>,
    /// Team ID
    pub team_id: Option<Uuid>,
    /// API Key ID
    pub api_key_id: Option<Uuid>,
    /// IP address
    pub ip_address: Option<String>,
    /// Rate limit type
    pub limit_type: String,
}
