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

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== UserPreferences Tests ====================

    #[test]
    fn test_user_preferences_default() {
        let prefs = UserPreferences::default();
        assert!(prefs.language.is_none());
        assert!(prefs.timezone.is_none());
        assert!(prefs.theme.is_none());
    }

    #[test]
    fn test_user_preferences_with_values() {
        let prefs = UserPreferences {
            language: Some("en-US".to_string()),
            timezone: Some("America/New_York".to_string()),
            theme: Some("dark".to_string()),
            notifications: NotificationSettings::default(),
            dashboard: DashboardSettings::default(),
            api: ApiPreferences::default(),
        };

        assert_eq!(prefs.language, Some("en-US".to_string()));
        assert_eq!(prefs.timezone, Some("America/New_York".to_string()));
        assert_eq!(prefs.theme, Some("dark".to_string()));
    }

    #[test]
    fn test_user_preferences_serialize() {
        let prefs = UserPreferences {
            language: Some("zh-CN".to_string()),
            timezone: Some("Asia/Shanghai".to_string()),
            theme: Some("light".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_string(&prefs).unwrap();
        assert!(json.contains("zh-CN"));
        assert!(json.contains("Asia/Shanghai"));
        assert!(json.contains("light"));
    }

    #[test]
    fn test_user_preferences_deserialize() {
        let json = r#"{
            "language": "ja-JP",
            "timezone": "Asia/Tokyo",
            "theme": "auto",
            "notifications": {"email_enabled": true, "slack_enabled": false, "webhook_enabled": false, "types": []},
            "dashboard": {"default_time_range": null, "favorite_charts": [], "layout": null},
            "api": {"default_model": null, "default_temperature": null, "default_max_tokens": null, "preferred_providers": []}
        }"#;

        let prefs: UserPreferences = serde_json::from_str(json).unwrap();
        assert_eq!(prefs.language, Some("ja-JP".to_string()));
        assert_eq!(prefs.timezone, Some("Asia/Tokyo".to_string()));
    }

    #[test]
    fn test_user_preferences_clone() {
        let original = UserPreferences {
            language: Some("fr-FR".to_string()),
            ..Default::default()
        };

        let cloned = original.clone();
        assert_eq!(original.language, cloned.language);
    }

    #[test]
    fn test_user_preferences_debug() {
        let prefs = UserPreferences::default();
        let debug_str = format!("{:?}", prefs);
        assert!(debug_str.contains("UserPreferences"));
    }

    // ==================== NotificationSettings Tests ====================

    #[test]
    fn test_notification_settings_default() {
        let settings = NotificationSettings::default();
        assert!(!settings.email_enabled);
        assert!(!settings.slack_enabled);
        assert!(!settings.webhook_enabled);
        assert!(settings.types.is_empty());
    }

    #[test]
    fn test_notification_settings_all_enabled() {
        let settings = NotificationSettings {
            email_enabled: true,
            slack_enabled: true,
            webhook_enabled: true,
            types: vec![
                NotificationType::RateLimitWarning,
                NotificationType::SecurityAlert,
            ],
        };

        assert!(settings.email_enabled);
        assert!(settings.slack_enabled);
        assert!(settings.webhook_enabled);
        assert_eq!(settings.types.len(), 2);
    }

    #[test]
    fn test_notification_settings_serialize() {
        let settings = NotificationSettings {
            email_enabled: true,
            slack_enabled: false,
            webhook_enabled: true,
            types: vec![NotificationType::UsageReport],
        };

        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("\"email_enabled\":true"));
        assert!(json.contains("\"slack_enabled\":false"));
        assert!(json.contains("usage_report"));
    }

    #[test]
    fn test_notification_settings_deserialize() {
        let json = r#"{
            "email_enabled": true,
            "slack_enabled": true,
            "webhook_enabled": false,
            "types": ["rate_limit_warning", "quota_warning"]
        }"#;

        let settings: NotificationSettings = serde_json::from_str(json).unwrap();
        assert!(settings.email_enabled);
        assert!(settings.slack_enabled);
        assert!(!settings.webhook_enabled);
        assert_eq!(settings.types.len(), 2);
    }

    #[test]
    fn test_notification_settings_clone() {
        let original = NotificationSettings {
            email_enabled: true,
            slack_enabled: false,
            webhook_enabled: true,
            types: vec![NotificationType::ServiceAlert],
        };

        let cloned = original.clone();
        assert_eq!(original.email_enabled, cloned.email_enabled);
        assert_eq!(original.types.len(), cloned.types.len());
    }

    // ==================== NotificationType Tests ====================

    #[test]
    fn test_notification_type_rate_limit_warning() {
        let t = NotificationType::RateLimitWarning;
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, "\"rate_limit_warning\"");
    }

    #[test]
    fn test_notification_type_quota_warning() {
        let t = NotificationType::QuotaWarning;
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, "\"quota_warning\"");
    }

    #[test]
    fn test_notification_type_service_alert() {
        let t = NotificationType::ServiceAlert;
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, "\"service_alert\"");
    }

    #[test]
    fn test_notification_type_security_alert() {
        let t = NotificationType::SecurityAlert;
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, "\"security_alert\"");
    }

    #[test]
    fn test_notification_type_usage_report() {
        let t = NotificationType::UsageReport;
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, "\"usage_report\"");
    }

    #[test]
    fn test_notification_type_system_maintenance() {
        let t = NotificationType::SystemMaintenance;
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, "\"system_maintenance\"");
    }

    #[test]
    fn test_notification_type_deserialize() {
        let t: NotificationType = serde_json::from_str("\"security_alert\"").unwrap();
        assert!(matches!(t, NotificationType::SecurityAlert));
    }

    #[test]
    fn test_notification_type_clone() {
        let original = NotificationType::RateLimitWarning;
        let cloned = original.clone();
        let json1 = serde_json::to_string(&original).unwrap();
        let json2 = serde_json::to_string(&cloned).unwrap();
        assert_eq!(json1, json2);
    }

    #[test]
    fn test_notification_type_debug() {
        let t = NotificationType::QuotaWarning;
        let debug_str = format!("{:?}", t);
        assert!(debug_str.contains("QuotaWarning"));
    }

    // ==================== DashboardSettings Tests ====================

    #[test]
    fn test_dashboard_settings_default() {
        let settings = DashboardSettings::default();
        assert!(settings.default_time_range.is_none());
        assert!(settings.favorite_charts.is_empty());
        assert!(settings.layout.is_none());
    }

    #[test]
    fn test_dashboard_settings_with_values() {
        let settings = DashboardSettings {
            default_time_range: Some("24h".to_string()),
            favorite_charts: vec!["requests".to_string(), "latency".to_string()],
            layout: Some(serde_json::json!({"columns": 2})),
        };

        assert_eq!(settings.default_time_range, Some("24h".to_string()));
        assert_eq!(settings.favorite_charts.len(), 2);
        assert!(settings.layout.is_some());
    }

    #[test]
    fn test_dashboard_settings_serialize() {
        let settings = DashboardSettings {
            default_time_range: Some("7d".to_string()),
            favorite_charts: vec!["errors".to_string()],
            layout: None,
        };

        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("7d"));
        assert!(json.contains("errors"));
    }

    #[test]
    fn test_dashboard_settings_deserialize() {
        let json = r#"{
            "default_time_range": "30d",
            "favorite_charts": ["cpu", "memory"],
            "layout": null
        }"#;

        let settings: DashboardSettings = serde_json::from_str(json).unwrap();
        assert_eq!(settings.default_time_range, Some("30d".to_string()));
        assert_eq!(settings.favorite_charts.len(), 2);
    }

    #[test]
    fn test_dashboard_settings_clone() {
        let original = DashboardSettings {
            default_time_range: Some("1h".to_string()),
            favorite_charts: vec!["chart1".to_string()],
            layout: Some(serde_json::json!({"test": true})),
        };

        let cloned = original.clone();
        assert_eq!(original.default_time_range, cloned.default_time_range);
        assert_eq!(original.favorite_charts, cloned.favorite_charts);
    }

    #[test]
    fn test_dashboard_settings_debug() {
        let settings = DashboardSettings::default();
        let debug_str = format!("{:?}", settings);
        assert!(debug_str.contains("DashboardSettings"));
    }

    // ==================== ApiPreferences Tests ====================

    #[test]
    fn test_api_preferences_default() {
        let prefs = ApiPreferences::default();
        assert!(prefs.default_model.is_none());
        assert!(prefs.default_temperature.is_none());
        assert!(prefs.default_max_tokens.is_none());
        assert!(prefs.preferred_providers.is_empty());
    }

    #[test]
    fn test_api_preferences_with_values() {
        let prefs = ApiPreferences {
            default_model: Some("gpt-4".to_string()),
            default_temperature: Some(0.7),
            default_max_tokens: Some(2048),
            preferred_providers: vec!["openai".to_string(), "anthropic".to_string()],
        };

        assert_eq!(prefs.default_model, Some("gpt-4".to_string()));
        assert_eq!(prefs.default_temperature, Some(0.7));
        assert_eq!(prefs.default_max_tokens, Some(2048));
        assert_eq!(prefs.preferred_providers.len(), 2);
    }

    #[test]
    fn test_api_preferences_serialize() {
        let prefs = ApiPreferences {
            default_model: Some("claude-3".to_string()),
            default_temperature: Some(0.5),
            default_max_tokens: Some(1000),
            preferred_providers: vec!["anthropic".to_string()],
        };

        let json = serde_json::to_string(&prefs).unwrap();
        assert!(json.contains("claude-3"));
        assert!(json.contains("0.5"));
        assert!(json.contains("1000"));
        assert!(json.contains("anthropic"));
    }

    #[test]
    fn test_api_preferences_deserialize() {
        let json = r#"{
            "default_model": "gemini-pro",
            "default_temperature": 0.8,
            "default_max_tokens": 4096,
            "preferred_providers": ["google", "openai"]
        }"#;

        let prefs: ApiPreferences = serde_json::from_str(json).unwrap();
        assert_eq!(prefs.default_model, Some("gemini-pro".to_string()));
        assert_eq!(prefs.default_temperature, Some(0.8));
        assert_eq!(prefs.default_max_tokens, Some(4096));
        assert_eq!(prefs.preferred_providers.len(), 2);
    }

    #[test]
    fn test_api_preferences_clone() {
        let original = ApiPreferences {
            default_model: Some("test".to_string()),
            default_temperature: Some(1.0),
            default_max_tokens: Some(100),
            preferred_providers: vec!["test".to_string()],
        };

        let cloned = original.clone();
        assert_eq!(original.default_model, cloned.default_model);
        assert_eq!(original.default_temperature, cloned.default_temperature);
    }

    #[test]
    fn test_api_preferences_debug() {
        let prefs = ApiPreferences::default();
        let debug_str = format!("{:?}", prefs);
        assert!(debug_str.contains("ApiPreferences"));
    }

    #[test]
    fn test_api_preferences_temperature_range() {
        // Temperature = 0 (deterministic)
        let prefs = ApiPreferences {
            default_temperature: Some(0.0),
            ..Default::default()
        };
        assert_eq!(prefs.default_temperature, Some(0.0));

        // Temperature = 1 (more creative)
        let prefs = ApiPreferences {
            default_temperature: Some(1.0),
            ..Default::default()
        };
        assert_eq!(prefs.default_temperature, Some(1.0));

        // Temperature = 2 (very creative)
        let prefs = ApiPreferences {
            default_temperature: Some(2.0),
            ..Default::default()
        };
        assert_eq!(prefs.default_temperature, Some(2.0));
    }

    #[test]
    fn test_api_preferences_max_tokens_range() {
        // Small max tokens
        let prefs = ApiPreferences {
            default_max_tokens: Some(1),
            ..Default::default()
        };
        assert_eq!(prefs.default_max_tokens, Some(1));

        // Large max tokens
        let prefs = ApiPreferences {
            default_max_tokens: Some(100_000),
            ..Default::default()
        };
        assert_eq!(prefs.default_max_tokens, Some(100_000));
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_full_user_preferences() {
        let prefs = UserPreferences {
            language: Some("en-US".to_string()),
            timezone: Some("UTC".to_string()),
            theme: Some("system".to_string()),
            notifications: NotificationSettings {
                email_enabled: true,
                slack_enabled: true,
                webhook_enabled: false,
                types: vec![
                    NotificationType::RateLimitWarning,
                    NotificationType::SecurityAlert,
                    NotificationType::UsageReport,
                ],
            },
            dashboard: DashboardSettings {
                default_time_range: Some("24h".to_string()),
                favorite_charts: vec!["requests".to_string(), "latency".to_string()],
                layout: Some(serde_json::json!({"columns": 3, "rows": 2})),
            },
            api: ApiPreferences {
                default_model: Some("gpt-4-turbo".to_string()),
                default_temperature: Some(0.7),
                default_max_tokens: Some(4096),
                preferred_providers: vec!["openai".to_string(), "anthropic".to_string()],
            },
        };

        // Serialize and deserialize
        let json = serde_json::to_string(&prefs).unwrap();
        let deserialized: UserPreferences = serde_json::from_str(&json).unwrap();

        assert_eq!(prefs.language, deserialized.language);
        assert_eq!(prefs.notifications.email_enabled, deserialized.notifications.email_enabled);
        assert_eq!(prefs.api.default_model, deserialized.api.default_model);
    }
}
