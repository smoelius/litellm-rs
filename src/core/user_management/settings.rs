//! User, team, and organization settings

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// User preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    /// Preferred language
    pub language: Option<String>,
    /// Timezone
    pub timezone: Option<String>,
    /// Email notifications enabled
    pub email_notifications: bool,
    /// Slack notifications enabled
    pub slack_notifications: bool,
    /// Dashboard preferences
    pub dashboard_config: HashMap<String, serde_json::Value>,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            language: Some("en".to_string()),
            timezone: Some("UTC".to_string()),
            email_notifications: true,
            slack_notifications: false,
            dashboard_config: HashMap::new(),
        }
    }
}

/// Team settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamSettings {
    /// Default model for the team
    pub default_model: Option<String>,
    /// Auto-approve new members
    pub auto_approve_members: bool,
    /// Require approval for high-cost requests
    pub require_approval_for_high_cost: bool,
    /// High cost threshold
    pub high_cost_threshold: Option<f64>,
    /// Team-specific rate limits
    pub rate_limits: Option<crate::core::virtual_keys::RateLimits>,
}

impl Default for TeamSettings {
    fn default() -> Self {
        Self {
            default_model: None,
            auto_approve_members: true,
            require_approval_for_high_cost: false,
            high_cost_threshold: Some(10.0),
            rate_limits: None,
        }
    }
}

/// Organization settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationSettings {
    /// SSO configuration
    pub sso_config: Option<SSOConfig>,
    /// Default team for new users
    pub default_team: Option<String>,
    /// Require email verification
    pub require_email_verification: bool,
    /// Password policy
    pub password_policy: PasswordPolicy,
    /// Session timeout in minutes
    pub session_timeout_minutes: u32,
    /// Allowed email domains
    pub allowed_email_domains: Vec<String>,
}

impl Default for OrganizationSettings {
    fn default() -> Self {
        Self {
            sso_config: None,
            default_team: None,
            require_email_verification: true,
            password_policy: PasswordPolicy::default(),
            session_timeout_minutes: 480, // 8 hours
            allowed_email_domains: vec![],
        }
    }
}

/// SSO configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SSOConfig {
    /// SSO provider type
    pub provider: SSOProvider,
    /// Client ID
    pub client_id: String,
    /// Client secret (encrypted)
    pub client_secret: String,
    /// Authorization endpoint
    pub auth_endpoint: String,
    /// Token endpoint
    pub token_endpoint: String,
    /// User info endpoint
    pub userinfo_endpoint: String,
    /// Scopes
    pub scopes: Vec<String>,
    /// Attribute mappings
    pub attribute_mappings: HashMap<String, String>,
}

/// SSO providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SSOProvider {
    Google,
    Microsoft,
    Okta,
    Auth0,
    Generic,
}

/// Password policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordPolicy {
    /// Minimum length
    pub min_length: u32,
    /// Require uppercase
    pub require_uppercase: bool,
    /// Require lowercase
    pub require_lowercase: bool,
    /// Require numbers
    pub require_numbers: bool,
    /// Require special characters
    pub require_special_chars: bool,
    /// Password expiry in days
    pub expiry_days: Option<u32>,
}

impl Default for PasswordPolicy {
    fn default() -> Self {
        Self {
            min_length: 8,
            require_uppercase: true,
            require_lowercase: true,
            require_numbers: true,
            require_special_chars: false,
            expiry_days: None,
        }
    }
}
