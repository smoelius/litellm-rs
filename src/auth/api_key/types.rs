//! API key types and data structures
//!
//! This module contains request/response types for API key management.

use crate::core::models::{ApiKey, RateLimits};
use crate::core::models::user::types::User;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// API key creation request
#[derive(Debug, Clone)]
pub struct CreateApiKeyRequest {
    /// Key name/description
    pub name: String,
    /// Associated user ID
    pub user_id: Option<Uuid>,
    /// Associated team ID
    pub team_id: Option<Uuid>,
    /// Permissions for the key
    pub permissions: Vec<String>,
    /// Rate limits for the key
    pub rate_limits: Option<RateLimits>,
    /// Expiration date
    pub expires_at: Option<DateTime<Utc>>,
}

/// API key verification result
#[derive(Debug, Clone)]
pub struct ApiKeyVerification {
    /// The API key
    pub api_key: ApiKey,
    /// Associated user (if any)
    pub user: Option<User>,
    /// Whether the key is valid
    pub is_valid: bool,
    /// Reason for invalidity (if any)
    pub invalid_reason: Option<String>,
}
