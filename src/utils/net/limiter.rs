//! Rate limiting utilities for the Gateway
//!
//! This module provides rate limiting functionality using token bucket and sliding window algorithms.

#![allow(dead_code)]

use crate::utils::error::{GatewayError, Result};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Rate limiter implementation
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RateLimiter {
    /// Rate limit configurations
    configs: Arc<RwLock<HashMap<String, RateLimitConfig>>>,
    /// Token buckets for rate limiting
    buckets: Arc<RwLock<HashMap<String, TokenBucket>>>,
    /// Sliding windows for request counting
    windows: Arc<RwLock<HashMap<String, SlidingWindow>>>,
}

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

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new() -> Self {
        Self {
            configs: Arc::new(RwLock::new(HashMap::new())),
            buckets: Arc::new(RwLock::new(HashMap::new())),
            windows: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add rate limit configuration
    pub async fn add_config(&self, key: String, config: RateLimitConfig) {
        let mut configs = self.configs.write().await;
        configs.insert(key, config);
    }

    /// Check rate limit for a request
    pub async fn check_rate_limit(
        &self,
        key: &RateLimitKey,
        tokens: u32,
    ) -> Result<RateLimitResult> {
        let key_str = self.build_key_string(key);

        // Get configuration
        let configs = self.configs.read().await;
        let config = configs
            .get(&key_str)
            .or_else(|| configs.get("default"))
            .ok_or_else(|| GatewayError::Config("No rate limit config found".to_string()))?;

        // Check different rate limits
        let mut result = RateLimitResult {
            allowed: true,
            remaining_requests: None,
            remaining_tokens: None,
            reset_time: None,
            retry_after: None,
            limit_type: None,
        };

        // Check RPM (requests per minute)
        if let Some(rpm) = config.rpm {
            let rpm_result = self
                .check_sliding_window_requests(
                    &format!("{}_rpm", key_str),
                    Duration::from_secs(60),
                    rpm,
                )
                .await?;

            if !rpm_result.allowed {
                result.allowed = false;
                result.limit_type = Some("rpm".to_string());
                result.retry_after = rpm_result.retry_after;
                return Ok(result);
            }
            result.remaining_requests = rpm_result.remaining_requests;
        }

        // Check TPM (tokens per minute)
        if let Some(tpm) = config.tpm {
            let tpm_result = self
                .check_sliding_window_tokens(
                    &format!("{}_tpm", key_str),
                    Duration::from_secs(60),
                    tpm,
                    tokens,
                )
                .await?;

            if !tpm_result.allowed {
                result.allowed = false;
                result.limit_type = Some("tpm".to_string());
                result.retry_after = tpm_result.retry_after;
                return Ok(result);
            }
            result.remaining_tokens = tpm_result.remaining_tokens;
        }

        // Check RPD (requests per day)
        if let Some(rpd) = config.rpd {
            let rpd_result = self
                .check_sliding_window_requests(
                    &format!("{}_rpd", key_str),
                    Duration::from_secs(86400), // 24 hours
                    rpd,
                )
                .await?;

            if !rpd_result.allowed {
                result.allowed = false;
                result.limit_type = Some("rpd".to_string());
                result.retry_after = rpd_result.retry_after;
                return Ok(result);
            }
        }

        // Check TPD (tokens per day)
        if let Some(tpd) = config.tpd {
            let tpd_result = self
                .check_sliding_window_tokens(
                    &format!("{}_tpd", key_str),
                    Duration::from_secs(86400), // 24 hours
                    tpd,
                    tokens,
                )
                .await?;

            if !tpd_result.allowed {
                result.allowed = false;
                result.limit_type = Some("tpd".to_string());
                result.retry_after = tpd_result.retry_after;
                return Ok(result);
            }
        }

        // If allowed, record the request
        if result.allowed {
            self.record_request(&key_str, tokens).await?;
        }

        Ok(result)
    }

    /// Check sliding window for requests
    async fn check_sliding_window_requests(
        &self,
        key: &str,
        window_size: Duration,
        limit: u32,
    ) -> Result<RateLimitResult> {
        let mut windows = self.windows.write().await;
        let window = windows
            .entry(key.to_string())
            .or_insert_with(|| SlidingWindow {
                window_size,
                requests: Vec::new(),
                tokens: Vec::new(),
            });

        let now = Instant::now();
        let window_start = now - window_size;

        // Remove old requests
        window
            .requests
            .retain(|&timestamp| timestamp > window_start);

        let current_count = window.requests.len() as u32;
        let allowed = current_count < limit;
        let remaining = limit.saturating_sub(current_count);

        let retry_after = if !allowed {
            window
                .requests
                .first()
                .map(|&first| window_size - (now - first))
        } else {
            None
        };

        Ok(RateLimitResult {
            allowed,
            remaining_requests: Some(remaining),
            remaining_tokens: None,
            reset_time: Some(window_size),
            retry_after,
            limit_type: None,
        })
    }

    /// Check sliding window for tokens
    async fn check_sliding_window_tokens(
        &self,
        key: &str,
        window_size: Duration,
        limit: u32,
        tokens: u32,
    ) -> Result<RateLimitResult> {
        let mut windows = self.windows.write().await;
        let window = windows
            .entry(key.to_string())
            .or_insert_with(|| SlidingWindow {
                window_size,
                requests: Vec::new(),
                tokens: Vec::new(),
            });

        let now = Instant::now();
        let window_start = now - window_size;

        // Remove old token records
        window
            .tokens
            .retain(|(timestamp, _)| *timestamp > window_start);

        let current_tokens: u32 = window.tokens.iter().map(|(_, count)| count).sum();
        let allowed = current_tokens + tokens <= limit;
        let remaining = limit.saturating_sub(current_tokens);

        let retry_after = if !allowed {
            window
                .tokens
                .first()
                .map(|(first, _)| window_size - (now - *first))
        } else {
            None
        };

        Ok(RateLimitResult {
            allowed,
            remaining_requests: None,
            remaining_tokens: Some(remaining),
            reset_time: Some(window_size),
            retry_after,
            limit_type: None,
        })
    }

    /// Record a request
    async fn record_request(&self, key: &str, tokens: u32) -> Result<()> {
        let now = Instant::now();

        // Record in all relevant windows
        let mut windows = self.windows.write().await;

        // RPM window
        let rpm_key = format!("{}_rpm", key);
        if let Some(window) = windows.get_mut(&rpm_key) {
            window.requests.push(now);
        }

        // TPM window
        let tpm_key = format!("{}_tpm", key);
        if let Some(window) = windows.get_mut(&tpm_key) {
            window.tokens.push((now, tokens));
        }

        // RPD window
        let rpd_key = format!("{}_rpd", key);
        if let Some(window) = windows.get_mut(&rpd_key) {
            window.requests.push(now);
        }

        // TPD window
        let tpd_key = format!("{}_tpd", key);
        if let Some(window) = windows.get_mut(&tpd_key) {
            window.tokens.push((now, tokens));
        }

        Ok(())
    }

    /// Build key string from components
    fn build_key_string(&self, key: &RateLimitKey) -> String {
        let mut parts = Vec::new();

        if let Some(user_id) = key.user_id {
            parts.push(format!("user:{}", user_id));
        }

        if let Some(team_id) = key.team_id {
            parts.push(format!("team:{}", team_id));
        }

        if let Some(api_key_id) = key.api_key_id {
            parts.push(format!("key:{}", api_key_id));
        }

        if let Some(ip) = &key.ip_address {
            parts.push(format!("ip:{}", ip));
        }

        parts.push(format!("type:{}", key.limit_type));

        parts.join(":")
    }

    /// Clean up old entries
    pub async fn cleanup(&self) {
        let now = Instant::now();
        let mut windows = self.windows.write().await;

        windows.retain(|_, window| {
            let window_start = now - window.window_size;
            window
                .requests
                .retain(|&timestamp| timestamp > window_start);
            window
                .tokens
                .retain(|(timestamp, _)| *timestamp > window_start);

            // Keep window if it has recent activity
            !window.requests.is_empty() || !window.tokens.is_empty()
        });
    }

    /// Get rate limit status
    pub async fn get_status(&self, key: &RateLimitKey) -> Result<HashMap<String, RateLimitResult>> {
        let key_str = self.build_key_string(key);
        let mut status = HashMap::new();

        let configs = self.configs.read().await;
        if let Some(config) = configs.get(&key_str).or_else(|| configs.get("default")) {
            if let Some(rpm) = config.rpm {
                let result = self
                    .check_sliding_window_requests(
                        &format!("{}_rpm", key_str),
                        Duration::from_secs(60),
                        rpm,
                    )
                    .await?;
                status.insert("rpm".to_string(), result);
            }

            if let Some(tpm) = config.tpm {
                let result = self
                    .check_sliding_window_tokens(
                        &format!("{}_tpm", key_str),
                        Duration::from_secs(60),
                        tpm,
                        0, // Don't consume tokens for status check
                    )
                    .await?;
                status.insert("tpm".to_string(), result);
            }
        }

        Ok(status)
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

impl RateLimitKey {
    /// Create a new rate limit key
    pub fn new(limit_type: String) -> Self {
        Self {
            user_id: None,
            team_id: None,
            api_key_id: None,
            ip_address: None,
            limit_type,
        }
    }

    /// Set user ID
    pub fn with_user(mut self, user_id: Uuid) -> Self {
        self.user_id = Some(user_id);
        self
    }

    /// Set team ID
    pub fn with_team(mut self, team_id: Uuid) -> Self {
        self.team_id = Some(team_id);
        self
    }

    /// Set API key ID
    pub fn with_api_key(mut self, api_key_id: Uuid) -> Self {
        self.api_key_id = Some(api_key_id);
        self
    }

    /// Set IP address
    pub fn with_ip(mut self, ip_address: String) -> Self {
        self.ip_address = Some(ip_address);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_creation() {
        let limiter = RateLimiter::new();

        let config = RateLimitConfig {
            rpm: Some(10),
            tpm: Some(1000),
            rpd: None,
            tpd: None,
            concurrent: None,
            burst: None,
        };

        limiter.add_config("test".to_string(), config).await;
    }

    #[tokio::test]
    async fn test_rate_limit_check() {
        let limiter = RateLimiter::new();

        let config = RateLimitConfig {
            rpm: Some(2), // Very low limit for testing
            tpm: Some(100),
            rpd: None,
            tpd: None,
            concurrent: None,
            burst: None,
        };

        let key = RateLimitKey::new("test".to_string()).with_user(Uuid::new_v4());

        limiter
            .add_config(limiter.build_key_string(&key), config)
            .await;

        // First request should be allowed
        let result = limiter.check_rate_limit(&key, 10).await.unwrap();
        assert!(result.allowed);

        // Second request should be allowed
        let result = limiter.check_rate_limit(&key, 10).await.unwrap();
        assert!(result.allowed);

        // Third request should be denied
        let result = limiter.check_rate_limit(&key, 10).await.unwrap();
        assert!(!result.allowed);
        assert_eq!(result.limit_type, Some("rpm".to_string()));
    }

    #[tokio::test]
    async fn test_token_rate_limit() {
        let limiter = RateLimiter::new();

        let config = RateLimitConfig {
            rpm: None,
            tpm: Some(50), // Low token limit
            rpd: None,
            tpd: None,
            concurrent: None,
            burst: None,
        };

        let key = RateLimitKey::new("test".to_string()).with_user(Uuid::new_v4());

        limiter
            .add_config(limiter.build_key_string(&key), config)
            .await;

        // Request with 30 tokens should be allowed
        let result = limiter.check_rate_limit(&key, 30).await.unwrap();
        assert!(result.allowed);

        // Request with 25 tokens should be denied (30 + 25 > 50)
        let result = limiter.check_rate_limit(&key, 25).await.unwrap();
        assert!(!result.allowed);
        assert_eq!(result.limit_type, Some("tpm".to_string()));
    }

    #[test]
    fn test_key_building() {
        let limiter = RateLimiter::new();
        let user_id = Uuid::new_v4();
        let team_id = Uuid::new_v4();

        let key = RateLimitKey::new("test".to_string())
            .with_user(user_id)
            .with_team(team_id)
            .with_ip("127.0.0.1".to_string());

        let key_str = limiter.build_key_string(&key);
        assert!(key_str.contains(&format!("user:{}", user_id)));
        assert!(key_str.contains(&format!("team:{}", team_id)));
        assert!(key_str.contains("ip:127.0.0.1"));
        assert!(key_str.contains("type:test"));
    }
}
