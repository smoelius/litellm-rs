//! Tests for rate limiting functionality
//!
//! This module contains comprehensive tests for the rate limiter.

#![cfg(test)]

use super::limiter::RateLimiter;
use super::types::{RateLimitConfig, RateLimitKey};
use uuid::Uuid;

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
