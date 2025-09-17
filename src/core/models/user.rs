//! User models for the Gateway
//!
//! This module defines user-related data structures.

use super::{Metadata, UsageStats};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// User account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// User metadata
    #[serde(flatten)]
    pub metadata: Metadata,
    /// Username (unique)
    pub username: String,
    /// Email address (unique)
    pub email: String,
    /// Display name
    pub display_name: Option<String>,
    /// Password hash
    #[serde(skip_serializing)]
    pub password_hash: String,
    /// User role
    pub role: UserRole,
    /// User status
    pub status: UserStatus,
    /// Associated team IDs
    pub team_ids: Vec<Uuid>,
    /// User preferences
    pub preferences: UserPreferences,
    /// Usage statistics
    pub usage_stats: UsageStats,
    /// Rate limits
    pub rate_limits: Option<UserRateLimits>,
    /// Last login timestamp
    pub last_login_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Email verification status
    pub email_verified: bool,
    /// Two-factor authentication enabled
    pub two_factor_enabled: bool,
    /// Profile information
    pub profile: UserProfile,
}

/// User role
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    /// Super administrator
    SuperAdmin,
    /// Administrator
    Admin,
    /// Team manager
    Manager,
    /// Regular user
    User,
    /// Read-only user
    Viewer,
    /// API-only user
    ApiUser,
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRole::SuperAdmin => write!(f, "super_admin"),
            UserRole::Admin => write!(f, "admin"),
            UserRole::Manager => write!(f, "manager"),
            UserRole::User => write!(f, "user"),
            UserRole::Viewer => write!(f, "viewer"),
            UserRole::ApiUser => write!(f, "api_user"),
        }
    }
}

impl std::str::FromStr for UserRole {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "super_admin" => Ok(UserRole::SuperAdmin),
            "admin" => Ok(UserRole::Admin),
            "manager" => Ok(UserRole::Manager),
            "user" => Ok(UserRole::User),
            "viewer" => Ok(UserRole::Viewer),
            "api_user" => Ok(UserRole::ApiUser),
            _ => Err(format!("Invalid user role: {}", s)),
        }
    }
}

/// User status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserStatus {
    /// Active user
    Active,
    /// Inactive user
    Inactive,
    /// Suspended user
    Suspended,
    /// Pending email verification
    Pending,
    /// Deleted user (soft delete)
    Deleted,
}

/// User preferences
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserPreferences {
    /// Preferred language
    pub language: Option<String>,
    /// Timezone
    pub timezone: Option<String>,
    /// Theme preference
    pub theme: Option<String>,
    /// Notification settings
    pub notifications: NotificationSettings,
    /// Dashboard settings
    pub dashboard: DashboardSettings,
    /// API preferences
    pub api: ApiPreferences,
}

/// Notification settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NotificationSettings {
    /// Email notifications enabled
    pub email_enabled: bool,
    /// Slack notifications enabled
    pub slack_enabled: bool,
    /// Webhook notifications enabled
    pub webhook_enabled: bool,
    /// Notification types
    pub types: Vec<NotificationType>,
}

/// Notification type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationType {
    /// Rate limit warnings
    RateLimitWarning,
    /// Quota warnings
    QuotaWarning,
    /// Service alerts
    ServiceAlert,
    /// Security alerts
    SecurityAlert,
    /// Usage reports
    UsageReport,
    /// System maintenance
    SystemMaintenance,
}

/// Dashboard settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DashboardSettings {
    /// Default time range
    pub default_time_range: Option<String>,
    /// Favorite charts
    pub favorite_charts: Vec<String>,
    /// Custom dashboard layout
    pub layout: Option<serde_json::Value>,
}

/// API preferences
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiPreferences {
    /// Default model
    pub default_model: Option<String>,
    /// Default temperature
    pub default_temperature: Option<f32>,
    /// Default max tokens
    pub default_max_tokens: Option<u32>,
    /// Preferred providers
    pub preferred_providers: Vec<String>,
}

/// User rate limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRateLimits {
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
    /// Monthly budget limit
    pub monthly_budget: Option<f64>,
}

/// User profile information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserProfile {
    /// First name
    pub first_name: Option<String>,
    /// Last name
    pub last_name: Option<String>,
    /// Company/Organization
    pub company: Option<String>,
    /// Job title
    pub title: Option<String>,
    /// Phone number
    pub phone: Option<String>,
    /// Avatar URL
    pub avatar_url: Option<String>,
    /// Bio/Description
    pub bio: Option<String>,
    /// Location
    pub location: Option<String>,
    /// Website URL
    pub website: Option<String>,
    /// Social media links
    pub social_links: HashMap<String, String>,
}

/// User session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    /// Session metadata
    #[serde(flatten)]
    pub metadata: Metadata,
    /// User ID
    pub user_id: Uuid,
    /// Session token
    #[serde(skip_serializing)]
    pub token: String,
    /// Session type
    pub session_type: SessionType,
    /// IP address
    pub ip_address: Option<String>,
    /// User agent
    pub user_agent: Option<String>,
    /// Expires at
    pub expires_at: chrono::DateTime<chrono::Utc>,
    /// Last activity
    pub last_activity: chrono::DateTime<chrono::Utc>,
    /// Session data
    pub data: HashMap<String, serde_json::Value>,
}

/// Session type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionType {
    /// Web session
    Web,
    /// API session
    Api,
    /// Mobile session
    Mobile,
    /// CLI session
    Cli,
}

/// User activity log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserActivity {
    /// Activity metadata
    #[serde(flatten)]
    pub metadata: Metadata,
    /// User ID
    pub user_id: Uuid,
    /// Activity type
    pub activity_type: ActivityType,
    /// Activity description
    pub description: String,
    /// IP address
    pub ip_address: Option<String>,
    /// User agent
    pub user_agent: Option<String>,
    /// Additional data
    pub data: HashMap<String, serde_json::Value>,
}

/// Activity type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivityType {
    /// User login
    Login,
    /// User logout
    Logout,
    /// Password change
    PasswordChange,
    /// Profile update
    ProfileUpdate,
    /// API key created
    ApiKeyCreated,
    /// API key deleted
    ApiKeyDeleted,
    /// Team joined
    TeamJoined,
    /// Team left
    TeamLeft,
    /// Settings changed
    SettingsChanged,
    /// Security event
    SecurityEvent,
}

impl User {
    /// Create a new user
    pub fn new(username: String, email: String, password_hash: String) -> Self {
        Self {
            metadata: Metadata::new(),
            username,
            email,
            display_name: None,
            password_hash,
            role: UserRole::User,
            status: UserStatus::Pending,
            team_ids: vec![],
            preferences: UserPreferences::default(),
            usage_stats: UsageStats::default(),
            rate_limits: None,
            last_login_at: None,
            email_verified: false,
            two_factor_enabled: false,
            profile: UserProfile::default(),
        }
    }

    /// Get user ID
    pub fn id(&self) -> Uuid {
        self.metadata.id
    }

    /// Check if user is active
    pub fn is_active(&self) -> bool {
        matches!(self.status, UserStatus::Active)
    }

    /// Check if user has role
    pub fn has_role(&self, role: &UserRole) -> bool {
        match (&self.role, role) {
            (UserRole::SuperAdmin, _) => true,
            (
                UserRole::Admin,
                UserRole::Admin
                | UserRole::Manager
                | UserRole::User
                | UserRole::Viewer
                | UserRole::ApiUser,
            ) => true,
            (
                UserRole::Manager,
                UserRole::Manager | UserRole::User | UserRole::Viewer | UserRole::ApiUser,
            ) => true,
            (current, target) => current == target,
        }
    }

    /// Check if user is in team
    pub fn is_in_team(&self, team_id: Uuid) -> bool {
        self.team_ids.contains(&team_id)
    }

    /// Add user to team
    pub fn add_to_team(&mut self, team_id: Uuid) {
        if !self.team_ids.contains(&team_id) {
            self.team_ids.push(team_id);
            self.metadata.touch();
        }
    }

    /// Remove user from team
    pub fn remove_from_team(&mut self, team_id: Uuid) {
        if let Some(pos) = self.team_ids.iter().position(|&id| id == team_id) {
            self.team_ids.remove(pos);
            self.metadata.touch();
        }
    }

    /// Update last login
    pub fn update_last_login(&mut self) {
        self.last_login_at = Some(chrono::Utc::now());
        self.metadata.touch();
    }

    /// Verify email
    pub fn verify_email(&mut self) {
        self.email_verified = true;
        if matches!(self.status, UserStatus::Pending) {
            self.status = UserStatus::Active;
        }
        self.metadata.touch();
    }

    /// Enable two-factor authentication
    pub fn enable_two_factor(&mut self) {
        self.two_factor_enabled = true;
        self.metadata.touch();
    }

    /// Disable two-factor authentication
    pub fn disable_two_factor(&mut self) {
        self.two_factor_enabled = false;
        self.metadata.touch();
    }

    /// Update usage statistics
    pub fn update_usage(&mut self, requests: u64, tokens: u64, cost: f64) {
        self.usage_stats.total_requests += requests;
        self.usage_stats.total_tokens += tokens;
        self.usage_stats.total_cost += cost;

        // Update daily stats
        let today = chrono::Utc::now().date_naive();
        let last_reset = self.usage_stats.last_reset.date_naive();

        if today != last_reset {
            self.usage_stats.requests_today = 0;
            self.usage_stats.tokens_today = 0;
            self.usage_stats.cost_today = 0.0;
            self.usage_stats.last_reset = chrono::Utc::now();
        }

        self.usage_stats.requests_today += requests as u32;
        self.usage_stats.tokens_today += tokens as u32;
        self.usage_stats.cost_today += cost;

        self.metadata.touch();
    }
}

impl UserSession {
    /// Create a new session
    pub fn new(
        user_id: Uuid,
        token: String,
        session_type: SessionType,
        expires_at: chrono::DateTime<chrono::Utc>,
    ) -> Self {
        Self {
            metadata: Metadata::new(),
            user_id,
            token,
            session_type,
            ip_address: None,
            user_agent: None,
            expires_at,
            last_activity: chrono::Utc::now(),
            data: HashMap::new(),
        }
    }

    /// Check if session is expired
    pub fn is_expired(&self) -> bool {
        chrono::Utc::now() > self.expires_at
    }

    /// Update last activity
    pub fn update_activity(&mut self) {
        self.last_activity = chrono::Utc::now();
    }

    /// Set session data
    pub fn set_data<K: Into<String>, V: Into<serde_json::Value>>(&mut self, key: K, value: V) {
        self.data.insert(key.into(), value.into());
    }

    /// Get session data
    pub fn get_data(&self, key: &str) -> Option<&serde_json::Value> {
        self.data.get(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_creation() {
        let user = User::new(
            "testuser".to_string(),
            "test@example.com".to_string(),
            "hashed_password".to_string(),
        );

        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "test@example.com");
        assert!(matches!(user.role, UserRole::User));
        assert!(matches!(user.status, UserStatus::Pending));
        assert!(!user.is_active());
    }

    #[test]
    fn test_user_role_hierarchy() {
        let mut user = User::new(
            "admin".to_string(),
            "admin@example.com".to_string(),
            "hashed_password".to_string(),
        );
        user.role = UserRole::Admin;

        assert!(user.has_role(&UserRole::Admin));
        assert!(user.has_role(&UserRole::User));
        assert!(user.has_role(&UserRole::Viewer));
        assert!(!user.has_role(&UserRole::SuperAdmin));
    }

    #[test]
    fn test_team_management() {
        let mut user = User::new(
            "testuser".to_string(),
            "test@example.com".to_string(),
            "hashed_password".to_string(),
        );

        let team_id = Uuid::new_v4();
        assert!(!user.is_in_team(team_id));

        user.add_to_team(team_id);
        assert!(user.is_in_team(team_id));

        user.remove_from_team(team_id);
        assert!(!user.is_in_team(team_id));
    }

    #[test]
    fn test_session_expiry() {
        let user_id = Uuid::new_v4();
        let token = "test_token".to_string();
        let expires_at = chrono::Utc::now() - chrono::Duration::hours(1); // Expired

        let session = UserSession::new(user_id, token, SessionType::Web, expires_at);
        assert!(session.is_expired());
    }
}
