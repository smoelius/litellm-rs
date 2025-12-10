//! Rate Limiting Implementation
//!
//! Provides sliding window rate limiting with support for multiple strategies

use crate::config::models::rate_limit::{RateLimitConfig, RateLimitStrategy};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::debug;

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
struct RateLimitEntry {
    /// Request timestamps for sliding window
    timestamps: Vec<Instant>,
    /// Token count for token bucket
    tokens: f64,
    /// Last token refill time
    last_refill: Instant,
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

/// Rate limiter implementation
pub struct RateLimiter {
    /// Rate limit configuration
    config: RateLimitConfig,
    /// Rate limit entries by key (IP or API key)
    entries: Arc<RwLock<HashMap<String, RateLimitEntry>>>,
    /// Window duration
    window: Duration,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            entries: Arc::new(RwLock::new(HashMap::new())),
            window: Duration::from_secs(60), // 1 minute window
        }
    }

    /// Create a rate limiter with custom window
    pub fn with_window(config: RateLimitConfig, window: Duration) -> Self {
        Self {
            config,
            entries: Arc::new(RwLock::new(HashMap::new())),
            window,
        }
    }

    /// Check if a request should be allowed (read-only, does not record)
    ///
    /// WARNING: Using check() followed by record() has a race condition.
    /// Use check_and_record() for atomic check-then-record operations.
    pub async fn check(&self, key: &str) -> RateLimitResult {
        if !self.config.enabled {
            return RateLimitResult {
                allowed: true,
                current_count: 0,
                limit: self.config.default_rpm,
                remaining: self.config.default_rpm,
                reset_after_secs: 0,
                retry_after_secs: None,
            };
        }

        match self.config.strategy {
            RateLimitStrategy::SlidingWindow => self.check_sliding_window_impl(key, false).await,
            RateLimitStrategy::TokenBucket => self.check_token_bucket_impl(key, false).await,
            RateLimitStrategy::FixedWindow => self.check_fixed_window_impl(key, false).await,
        }
    }

    /// Atomically check and record a request (prevents TOCTOU race condition)
    ///
    /// This is the preferred method for rate limiting as it performs both
    /// the check and record operations in a single lock acquisition.
    pub async fn check_and_record(&self, key: &str) -> RateLimitResult {
        if !self.config.enabled {
            return RateLimitResult {
                allowed: true,
                current_count: 0,
                limit: self.config.default_rpm,
                remaining: self.config.default_rpm,
                reset_after_secs: 0,
                retry_after_secs: None,
            };
        }

        match self.config.strategy {
            RateLimitStrategy::SlidingWindow => self.check_sliding_window_impl(key, true).await,
            RateLimitStrategy::TokenBucket => self.check_token_bucket_impl(key, true).await,
            RateLimitStrategy::FixedWindow => self.check_fixed_window_impl(key, true).await,
        }
    }

    /// Record a request (increments counter)
    ///
    /// WARNING: This is a separate operation from check() and has a race condition.
    /// Use check_and_record() for atomic operations.
    #[deprecated(note = "Use check_and_record() instead to avoid race conditions")]
    pub async fn record(&self, key: &str) {
        if !self.config.enabled {
            return;
        }

        let mut entries = self.entries.write().await;
        // Avoid String allocation if key already exists
        let entry = if let Some(e) = entries.get_mut(key) {
            e
        } else {
            entries.entry(key.to_string()).or_default()
        };

        match self.config.strategy {
            RateLimitStrategy::SlidingWindow | RateLimitStrategy::FixedWindow => {
                entry.timestamps.push(Instant::now());
            }
            RateLimitStrategy::TokenBucket => {
                // Token is consumed in check_token_bucket
            }
        }
    }

    /// Sliding window rate limiting implementation
    /// If `record` is true, atomically records the request if allowed
    async fn check_sliding_window_impl(&self, key: &str, record: bool) -> RateLimitResult {
        let now = Instant::now();
        let window_start = now - self.window;
        let limit = self.config.default_rpm;

        let mut entries = self.entries.write().await;
        // Avoid String allocation if key already exists
        let entry = if let Some(e) = entries.get_mut(key) {
            e
        } else {
            entries.entry(key.to_string()).or_default()
        };

        // Remove expired timestamps
        entry.timestamps.retain(|&t| t > window_start);

        let current_count = entry.timestamps.len() as u32;
        let allowed = current_count < limit;
        let remaining = limit.saturating_sub(current_count);

        // Calculate reset time (time until oldest request expires)
        let reset_after_secs = if let Some(&oldest) = entry.timestamps.first() {
            let elapsed = now.duration_since(oldest);
            self.window.saturating_sub(elapsed).as_secs()
        } else {
            self.window.as_secs()
        };

        let retry_after_secs = if !allowed {
            Some(reset_after_secs.max(1))
        } else {
            // Atomically record if allowed and record flag is set
            if record {
                entry.timestamps.push(now);
            }
            None
        };

        if !allowed {
            debug!(
                "Rate limit exceeded for {}: {}/{} requests",
                key, current_count, limit
            );
        }

        RateLimitResult {
            allowed,
            current_count,
            limit,
            // Adjust remaining if we just recorded
            remaining: if record && allowed { remaining.saturating_sub(1) } else { remaining },
            reset_after_secs,
            retry_after_secs,
        }
    }

    /// Token bucket rate limiting implementation
    /// If `record` is true, atomically consumes a token if allowed
    async fn check_token_bucket_impl(&self, key: &str, record: bool) -> RateLimitResult {
        let now = Instant::now();
        let limit = self.config.default_rpm;
        let tokens_per_second = limit as f64 / 60.0;

        let mut entries = self.entries.write().await;
        // Avoid String allocation if key already exists
        let entry = if let Some(e) = entries.get_mut(key) {
            e
        } else {
            entries.entry(key.to_string()).or_insert_with(|| {
                RateLimitEntry {
                    tokens: limit as f64,
                    last_refill: now,
                    timestamps: Vec::new(),
                }
            })
        };

        // Refill tokens based on elapsed time
        let elapsed = now.duration_since(entry.last_refill);
        let new_tokens = elapsed.as_secs_f64() * tokens_per_second;
        entry.tokens = (entry.tokens + new_tokens).min(limit as f64);
        entry.last_refill = now;

        let allowed = entry.tokens >= 1.0;
        let current_count = (limit as f64 - entry.tokens) as u32;
        let remaining = entry.tokens as u32;

        // Calculate time until next token
        let reset_after_secs = if entry.tokens < 1.0 {
            ((1.0 - entry.tokens) / tokens_per_second).ceil() as u64
        } else {
            0
        };

        let retry_after_secs = if !allowed {
            Some(reset_after_secs.max(1))
        } else {
            // Atomically consume token if allowed and record flag is set
            if record {
                entry.tokens -= 1.0;
            }
            None
        };

        RateLimitResult {
            allowed,
            current_count,
            limit,
            // Adjust remaining if we just consumed a token
            remaining: if record && allowed { remaining.saturating_sub(1) } else { remaining },
            reset_after_secs,
            retry_after_secs,
        }
    }

    /// Fixed window rate limiting implementation
    /// If `record` is true, atomically records the request if allowed
    async fn check_fixed_window_impl(&self, key: &str, record: bool) -> RateLimitResult {
        let now = Instant::now();
        let limit = self.config.default_rpm;

        let mut entries = self.entries.write().await;
        // Avoid String allocation if key already exists
        let entry = if let Some(e) = entries.get_mut(key) {
            e
        } else {
            entries.entry(key.to_string()).or_default()
        };

        // Check if we need to reset the window
        let window_start = if let Some(&first) = entry.timestamps.first() {
            let elapsed = now.duration_since(first);
            if elapsed >= self.window {
                entry.timestamps.clear();
                now
            } else {
                first
            }
        } else {
            now
        };

        let current_count = entry.timestamps.len() as u32;
        let allowed = current_count < limit;
        let remaining = limit.saturating_sub(current_count);

        // Calculate reset time
        let elapsed = now.duration_since(window_start);
        let reset_after_secs = self.window.saturating_sub(elapsed).as_secs();

        let retry_after_secs = if !allowed {
            Some(reset_after_secs.max(1))
        } else {
            // Atomically record if allowed and record flag is set
            if record {
                entry.timestamps.push(now);
            }
            None
        };

        RateLimitResult {
            allowed,
            current_count,
            limit,
            // Adjust remaining if we just recorded
            remaining: if record && allowed { remaining.saturating_sub(1) } else { remaining },
            reset_after_secs,
            retry_after_secs,
        }
    }

    /// Cleanup expired entries
    pub async fn cleanup(&self) {
        let now = Instant::now();
        let window_start = now - self.window;

        let mut entries = self.entries.write().await;
        entries.retain(|_, entry| {
            entry.timestamps.retain(|&t| t > window_start);
            !entry.timestamps.is_empty() || entry.tokens > 0.0
        });
    }

    /// Start background cleanup task
    pub fn start_cleanup_task(self: Arc<Self>) {
        let limiter = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                limiter.cleanup().await;
            }
        });
    }

    /// Get current status for a key
    pub async fn status(&self, key: &str) -> Option<RateLimitResult> {
        if !self.config.enabled {
            return None;
        }

        Some(self.check(key).await)
    }

    /// Check if rate limiting is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Get the configured limit
    pub fn limit(&self) -> u32 {
        self.config.default_rpm
    }
}

impl Clone for RateLimiter {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            entries: self.entries.clone(),
            window: self.window,
        }
    }
}

/// Global rate limiter singleton
static GLOBAL_RATE_LIMITER: std::sync::OnceLock<Arc<RateLimiter>> = std::sync::OnceLock::new();

/// Initialize the global rate limiter
pub fn init_global_rate_limiter(config: RateLimitConfig) {
    let limiter = Arc::new(RateLimiter::new(config));
    let _ = GLOBAL_RATE_LIMITER.set(limiter.clone());

    // Start cleanup task
    limiter.start_cleanup_task();
}

/// Get the global rate limiter
pub fn get_global_rate_limiter() -> Option<Arc<RateLimiter>> {
    GLOBAL_RATE_LIMITER.get().cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config(enabled: bool, rpm: u32) -> RateLimitConfig {
        RateLimitConfig {
            enabled,
            default_rpm: rpm,
            default_tpm: 100000,
            strategy: RateLimitStrategy::SlidingWindow,
        }
    }

    #[tokio::test]
    async fn test_rate_limiter_disabled() {
        let limiter = RateLimiter::new(test_config(false, 10));

        for _ in 0..100 {
            let result = limiter.check_and_record("test-key").await;
            assert!(result.allowed);
        }
    }

    #[tokio::test]
    async fn test_sliding_window_allows_within_limit() {
        let limiter = RateLimiter::new(test_config(true, 10));

        for i in 0..10 {
            let result = limiter.check_and_record("test-key").await;
            assert!(result.allowed, "Request {} should be allowed", i);
        }
    }

    #[tokio::test]
    async fn test_sliding_window_blocks_over_limit() {
        let limiter = RateLimiter::new(test_config(true, 5));

        // Fill up the limit using atomic check_and_record
        for _ in 0..5 {
            let result = limiter.check_and_record("test-key").await;
            assert!(result.allowed);
        }

        // This should be blocked
        let result = limiter.check_and_record("test-key").await;
        assert!(!result.allowed);
        assert!(result.retry_after_secs.is_some());
    }

    #[tokio::test]
    async fn test_different_keys_independent() {
        let limiter = RateLimiter::new(test_config(true, 2));

        // Fill up limit for key1 using atomic method
        limiter.check_and_record("key1").await;
        limiter.check_and_record("key1").await;

        // key1 should be blocked
        let result = limiter.check_and_record("key1").await;
        assert!(!result.allowed);

        // key2 should still work
        let result = limiter.check_and_record("key2").await;
        assert!(result.allowed);
    }

    #[tokio::test]
    async fn test_token_bucket() {
        let config = RateLimitConfig {
            enabled: true,
            default_rpm: 60, // 1 per second
            default_tpm: 100000,
            strategy: RateLimitStrategy::TokenBucket,
        };
        let limiter = RateLimiter::new(config);

        // Should allow initial requests (bucket starts full)
        let result = limiter.check_and_record("test-key").await;
        assert!(result.allowed);
    }

    #[tokio::test]
    async fn test_fixed_window() {
        let config = RateLimitConfig {
            enabled: true,
            default_rpm: 5,
            default_tpm: 100000,
            strategy: RateLimitStrategy::FixedWindow,
        };
        let limiter = RateLimiter::new(config);

        for _ in 0..5 {
            let result = limiter.check_and_record("test-key").await;
            assert!(result.allowed);
        }

        // Should be blocked
        let result = limiter.check_and_record("test-key").await;
        assert!(!result.allowed);
    }

    #[tokio::test]
    async fn test_remaining_count() {
        let limiter = RateLimiter::new(test_config(true, 5));

        // First check (no record) should show 5 remaining
        let result = limiter.check("test-key").await;
        assert_eq!(result.remaining, 5);

        // After check_and_record, remaining should be 4
        let result = limiter.check_and_record("test-key").await;
        assert_eq!(result.remaining, 4);

        // Do two more
        limiter.check_and_record("test-key").await;
        limiter.check_and_record("test-key").await;

        // Should have 2 remaining
        let result = limiter.check("test-key").await;
        assert_eq!(result.remaining, 2);
    }

    #[tokio::test]
    async fn test_atomic_check_and_record() {
        let limiter = RateLimiter::new(test_config(true, 3));

        // Use atomic method - should record and decrement in one operation
        let r1 = limiter.check_and_record("atomic-key").await;
        assert!(r1.allowed);
        assert_eq!(r1.remaining, 2); // 3-1=2 after recording

        let r2 = limiter.check_and_record("atomic-key").await;
        assert!(r2.allowed);
        assert_eq!(r2.remaining, 1);

        let r3 = limiter.check_and_record("atomic-key").await;
        assert!(r3.allowed);
        assert_eq!(r3.remaining, 0);

        // 4th request should be blocked
        let r4 = limiter.check_and_record("atomic-key").await;
        assert!(!r4.allowed);
    }

    #[tokio::test]
    async fn test_cleanup() {
        let limiter = RateLimiter::with_window(
            test_config(true, 100),
            Duration::from_millis(50),
        );

        // Use atomic method
        limiter.check_and_record("key1").await;
        limiter.check_and_record("key2").await;

        // Wait for window to expire
        tokio::time::sleep(Duration::from_millis(100)).await;

        limiter.cleanup().await;

        // After cleanup, should have full limit again
        let result = limiter.check("key1").await;
        assert!(result.allowed);
        assert_eq!(result.remaining, 100);
    }
}
