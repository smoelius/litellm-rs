//! Authentication rate limiter for brute force protection

use dashmap::DashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Brute force protection for authentication endpoints
pub struct AuthRateLimiter {
    /// Map of client identifier -> tracker
    attempts: DashMap<String, AuthAttemptTracker>,
    /// Maximum failed attempts before lockout
    max_attempts: u32,
    /// Time window for counting failures (seconds)
    window_secs: u64,
    /// Lockout duration (seconds) - uses exponential backoff
    base_lockout_secs: u64,
    /// Total blocked attempts counter for monitoring
    blocked_count: AtomicU64,
}

/// Tracks authentication attempts for a single client
struct AuthAttemptTracker {
    failure_count: u32,
    window_start: Instant,
    lockout_until: Option<Instant>,
    lockout_count: u32,
}

impl Default for AuthRateLimiter {
    fn default() -> Self {
        Self::new(5, 300, 60)
    }
}

impl AuthRateLimiter {
    pub fn new(max_attempts: u32, window_secs: u64, base_lockout_secs: u64) -> Self {
        Self {
            attempts: DashMap::new(),
            max_attempts,
            window_secs,
            base_lockout_secs,
            blocked_count: AtomicU64::new(0),
        }
    }

    pub fn check_allowed(&self, client_id: &str) -> Result<(), u64> {
        let now = Instant::now();

        let mut entry = self
            .attempts
            .entry(client_id.to_string())
            .or_insert_with(|| AuthAttemptTracker {
                failure_count: 0,
                window_start: now,
                lockout_until: None,
                lockout_count: 0,
            });

        let tracker = entry.value_mut();

        if let Some(lockout_until) = tracker.lockout_until {
            if now < lockout_until {
                let remaining = lockout_until.duration_since(now).as_secs();
                self.blocked_count.fetch_add(1, Ordering::Relaxed);
                return Err(remaining);
            }
            tracker.lockout_until = None;
        }

        let window_duration = Duration::from_secs(self.window_secs);
        if now.duration_since(tracker.window_start) > window_duration {
            tracker.failure_count = 0;
            tracker.window_start = now;
        }

        Ok(())
    }

    pub fn record_failure(&self, client_id: &str) -> Option<u64> {
        let now = Instant::now();

        let mut entry = self
            .attempts
            .entry(client_id.to_string())
            .or_insert_with(|| AuthAttemptTracker {
                failure_count: 0,
                window_start: now,
                lockout_until: None,
                lockout_count: 0,
            });

        let tracker = entry.value_mut();
        tracker.failure_count += 1;

        if tracker.failure_count >= self.max_attempts {
            let lockout_multiplier = 2u64.pow(tracker.lockout_count);
            let lockout_secs = self.base_lockout_secs.saturating_mul(lockout_multiplier);
            let lockout_duration = Duration::from_secs(lockout_secs);

            tracker.lockout_until = Some(now + lockout_duration);
            tracker.lockout_count += 1;
            tracker.failure_count = 0;

            tracing::warn!(
                "Client {} locked out for {} seconds (lockout #{})",
                client_id,
                lockout_secs,
                tracker.lockout_count
            );

            return Some(lockout_secs);
        }

        None
    }

    pub fn record_success(&self, client_id: &str) {
        if let Some(mut entry) = self.attempts.get_mut(client_id) {
            entry.failure_count = 0;
        }
    }

    pub fn blocked_attempts(&self) -> u64 {
        self.blocked_count.load(Ordering::Relaxed)
    }

    pub fn cleanup_old_entries(&self) {
        let now = Instant::now();
        let max_age = Duration::from_secs(self.window_secs * 2);

        self.attempts.retain(|_, tracker| {
            now.duration_since(tracker.window_start) < max_age
                || tracker.lockout_until.is_some_and(|until| until > now)
        });
    }
}

/// Global auth rate limiter
static AUTH_RATE_LIMITER: std::sync::OnceLock<Arc<AuthRateLimiter>> = std::sync::OnceLock::new();

/// Get or initialize the global auth rate limiter
pub fn get_auth_rate_limiter() -> Arc<AuthRateLimiter> {
    AUTH_RATE_LIMITER
        .get_or_init(|| Arc::new(AuthRateLimiter::default()))
        .clone()
}
