//! User preferences and settings

use serde::{Deserialize, Serialize};

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
