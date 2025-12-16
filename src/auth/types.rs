//! Authentication and authorization types

use crate::core::models::{ApiKey, User, UserSession};
use crate::core::models::RequestContext;

/// Authentication result
#[derive(Debug, Clone)]
pub struct AuthResult {
    /// Whether authentication was successful
    pub success: bool,
    /// Authenticated user (if any)
    pub user: Option<User>,
    /// API key used (if any)
    pub api_key: Option<ApiKey>,
    /// Session information (if any)
    pub session: Option<UserSession>,
    /// Error message (if authentication failed)
    pub error: Option<String>,
    /// Request context
    pub context: RequestContext,
}

/// Authorization result
#[derive(Debug, Clone)]
pub struct AuthzResult {
    /// Whether authorization was successful
    pub allowed: bool,
    /// Required permissions that were checked
    pub required_permissions: Vec<String>,
    /// User's actual permissions
    pub user_permissions: Vec<String>,
    /// Reason for denial (if not allowed)
    pub reason: Option<String>,
}

/// Authentication method
#[derive(Debug, Clone)]
pub enum AuthMethod {
    /// JWT token authentication
    Jwt(String),
    /// API key authentication
    ApiKey(String),
    /// Session-based authentication
    Session(String),
    /// No authentication
    None,
}
