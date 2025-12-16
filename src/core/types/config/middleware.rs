//! Middleware configuration types

use super::defaults::*;
use super::rate_limit::RateLimitConfig;
use super::retry::RetryConfig;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Middleware configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiddlewareConfig {
    /// Cache configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache: Option<CacheConfig>,
    /// Retry configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry: Option<RetryConfig>,
    /// Rate limit configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limit: Option<RateLimitConfig>,
    /// Auth configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<AuthConfig>,
    /// CORS configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cors: Option<CorsConfig>,
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CacheConfig {
    /// Memory cache
    #[serde(rename = "memory")]
    Memory {
        max_size: usize,
        #[serde(with = "super::duration_serde")]
        ttl: Duration,
    },
    /// Redis cache
    #[serde(rename = "redis")]
    Redis {
        url: String,
        #[serde(with = "super::duration_serde")]
        ttl: Duration,
        #[serde(default = "default_pool_size")]
        pool_size: u32,
    },
    /// Tiered cache
    #[serde(rename = "tiered")]
    Tiered {
        l1: Box<CacheConfig>,
        l2: Box<CacheConfig>,
        l3: Option<Box<CacheConfig>>,
    },
}

/// Auth configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Enabled authentication methods
    pub methods: Vec<AuthMethod>,
    /// JWT configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jwt: Option<JwtConfig>,
    /// API key configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<ApiKeyConfig>,
}

/// Authentication method
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthMethod {
    Jwt,
    ApiKey,
    Basic,
    Custom { handler: String },
}

/// JWT configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    /// Signing key
    pub secret: String,
    /// Algorithm
    #[serde(default = "default_jwt_algorithm")]
    pub algorithm: String,
    /// Expiration time (seconds)
    #[serde(default = "default_jwt_expiration")]
    pub expiration_seconds: u64,
    /// Issuer
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issuer: Option<String>,
    /// Audience
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audience: Option<String>,
}

/// API key configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyConfig {
    /// Header name
    #[serde(default = "default_api_key_header")]
    pub header_name: String,
    /// Prefix
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
    /// Valid API keys
    #[serde(default)]
    pub valid_keys: Vec<String>,
}

/// CORS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    /// Allowed origins
    pub allowed_origins: Vec<String>,
    /// Allowed methods
    #[serde(default = "default_cors_methods")]
    pub allowed_methods: Vec<String>,
    /// Allowed headers
    #[serde(default = "default_cors_headers")]
    pub allowed_headers: Vec<String>,
    /// Allow credentials
    #[serde(default)]
    pub allow_credentials: bool,
    /// Maximum age (seconds)
    #[serde(default = "default_cors_max_age")]
    pub max_age_seconds: u64,
}
