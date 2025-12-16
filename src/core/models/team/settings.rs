//! Team settings and configuration

use super::team::TeamVisibility;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Team settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TeamSettings {
    /// Default user role for new members
    pub default_member_role: Option<String>,
    /// Require approval for new members
    pub require_approval: bool,
    /// Allow members to invite others
    pub allow_member_invites: bool,
    /// Team visibility
    pub visibility: TeamVisibility,
    /// API access settings
    pub api_access: ApiAccessSettings,
    /// Notification settings
    pub notifications: TeamNotificationSettings,
    /// Security settings
    pub security: TeamSecuritySettings,
}

/// API access settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiAccessSettings {
    /// Enable API access
    pub enabled: bool,
    /// Allowed IP addresses
    pub allowed_ips: Vec<String>,
    /// Allowed domains
    pub allowed_domains: Vec<String>,
    /// Require API key authentication
    pub require_api_key: bool,
    /// Default API settings
    pub default_settings: HashMap<String, serde_json::Value>,
}

/// Team notification settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TeamNotificationSettings {
    /// Slack webhook URL
    pub slack_webhook: Option<String>,
    /// Email notifications
    pub email_notifications: bool,
    /// Webhook notifications
    pub webhook_notifications: bool,
    /// Notification channels
    pub channels: Vec<NotificationChannel>,
}

/// Notification channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationChannel {
    /// Channel name
    pub name: String,
    /// Channel type
    pub channel_type: ChannelType,
    /// Channel configuration
    pub config: HashMap<String, serde_json::Value>,
    /// Enabled
    pub enabled: bool,
}

/// Channel type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChannelType {
    /// Email channel
    Email,
    /// Slack channel
    Slack,
    /// Webhook channel
    Webhook,
    /// Microsoft Teams channel
    Teams,
    /// Discord channel
    Discord,
}

/// Team security settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TeamSecuritySettings {
    /// Require two-factor authentication
    pub require_2fa: bool,
    /// Password policy
    pub password_policy: PasswordPolicy,
    /// Session timeout in minutes
    pub session_timeout: Option<u32>,
    /// IP whitelist
    pub ip_whitelist: Vec<String>,
    /// Audit logging enabled
    pub audit_logging: bool,
}

/// Password policy
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
    pub require_special: bool,
    /// Password expiry in days
    pub expiry_days: Option<u32>,
}
