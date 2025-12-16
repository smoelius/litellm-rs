//! Utility functions and helper implementations
//!
//! This module contains utility methods for the rate limiter and RateLimitKey implementation.

use crate::utils::error::Result;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use uuid::Uuid;

use super::limiter::RateLimiter;
use super::types::{RateLimitKey, RateLimitResult};

impl RateLimiter {
    /// Record a request
    pub(super) async fn record_request(&self, key: &str, tokens: u32) -> Result<()> {
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
    pub(super) fn build_key_string(&self, key: &RateLimitKey) -> String {
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
