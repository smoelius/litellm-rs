//! User activity logging

use crate::core::models::Metadata;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_metadata() -> Metadata {
        Metadata::new()
    }

    fn create_test_activity(activity_type: ActivityType) -> UserActivity {
        UserActivity {
            metadata: create_test_metadata(),
            user_id: Uuid::new_v4(),
            activity_type,
            description: "Test activity".to_string(),
            ip_address: None,
            user_agent: None,
            data: HashMap::new(),
        }
    }

    // ==================== ActivityType Tests ====================

    #[test]
    fn test_activity_type_login() {
        let t = ActivityType::Login;
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, "\"login\"");
    }

    #[test]
    fn test_activity_type_logout() {
        let t = ActivityType::Logout;
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, "\"logout\"");
    }

    #[test]
    fn test_activity_type_password_change() {
        let t = ActivityType::PasswordChange;
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, "\"password_change\"");
    }

    #[test]
    fn test_activity_type_profile_update() {
        let t = ActivityType::ProfileUpdate;
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, "\"profile_update\"");
    }

    #[test]
    fn test_activity_type_api_key_created() {
        let t = ActivityType::ApiKeyCreated;
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, "\"api_key_created\"");
    }

    #[test]
    fn test_activity_type_api_key_deleted() {
        let t = ActivityType::ApiKeyDeleted;
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, "\"api_key_deleted\"");
    }

    #[test]
    fn test_activity_type_team_joined() {
        let t = ActivityType::TeamJoined;
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, "\"team_joined\"");
    }

    #[test]
    fn test_activity_type_team_left() {
        let t = ActivityType::TeamLeft;
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, "\"team_left\"");
    }

    #[test]
    fn test_activity_type_settings_changed() {
        let t = ActivityType::SettingsChanged;
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, "\"settings_changed\"");
    }

    #[test]
    fn test_activity_type_security_event() {
        let t = ActivityType::SecurityEvent;
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, "\"security_event\"");
    }

    #[test]
    fn test_activity_type_deserialize() {
        let t: ActivityType = serde_json::from_str("\"login\"").unwrap();
        assert!(matches!(t, ActivityType::Login));

        let t: ActivityType = serde_json::from_str("\"security_event\"").unwrap();
        assert!(matches!(t, ActivityType::SecurityEvent));
    }

    #[test]
    fn test_activity_type_clone() {
        let original = ActivityType::PasswordChange;
        let cloned = original.clone();
        let json1 = serde_json::to_string(&original).unwrap();
        let json2 = serde_json::to_string(&cloned).unwrap();
        assert_eq!(json1, json2);
    }

    #[test]
    fn test_activity_type_debug() {
        let t = ActivityType::TeamJoined;
        let debug_str = format!("{:?}", t);
        assert!(debug_str.contains("TeamJoined"));
    }

    // ==================== UserActivity Creation Tests ====================

    #[test]
    fn test_user_activity_creation() {
        let user_id = Uuid::new_v4();
        let activity = UserActivity {
            metadata: create_test_metadata(),
            user_id,
            activity_type: ActivityType::Login,
            description: "User logged in".to_string(),
            ip_address: Some("192.168.1.1".to_string()),
            user_agent: Some("Mozilla/5.0".to_string()),
            data: HashMap::new(),
        };

        assert_eq!(activity.user_id, user_id);
        assert!(matches!(activity.activity_type, ActivityType::Login));
        assert_eq!(activity.description, "User logged in");
        assert_eq!(activity.ip_address, Some("192.168.1.1".to_string()));
        assert_eq!(activity.user_agent, Some("Mozilla/5.0".to_string()));
    }

    #[test]
    fn test_user_activity_minimal() {
        let activity = create_test_activity(ActivityType::Logout);

        assert!(activity.ip_address.is_none());
        assert!(activity.user_agent.is_none());
        assert!(activity.data.is_empty());
    }

    #[test]
    fn test_user_activity_with_data() {
        let mut activity = create_test_activity(ActivityType::SettingsChanged);

        activity.data.insert(
            "changed_setting".to_string(),
            serde_json::json!("theme"),
        );
        activity.data.insert(
            "old_value".to_string(),
            serde_json::json!("light"),
        );
        activity.data.insert(
            "new_value".to_string(),
            serde_json::json!("dark"),
        );

        assert_eq!(activity.data.len(), 3);
        assert_eq!(activity.data.get("changed_setting").unwrap(), "theme");
    }

    // ==================== UserActivity Serialization Tests ====================

    #[test]
    fn test_user_activity_serialize() {
        let activity = create_test_activity(ActivityType::ApiKeyCreated);

        let json = serde_json::to_string(&activity).unwrap();

        assert!(json.contains("api_key_created"));
        assert!(json.contains("Test activity"));
    }

    #[test]
    fn test_user_activity_serialize_with_ip() {
        let mut activity = create_test_activity(ActivityType::Login);
        activity.ip_address = Some("10.0.0.1".to_string());

        let json = serde_json::to_string(&activity).unwrap();
        assert!(json.contains("10.0.0.1"));
    }

    #[test]
    fn test_user_activity_serialize_with_user_agent() {
        let mut activity = create_test_activity(ActivityType::Login);
        activity.user_agent = Some("Chrome/120.0".to_string());

        let json = serde_json::to_string(&activity).unwrap();
        assert!(json.contains("Chrome/120.0"));
    }

    #[test]
    fn test_user_activity_serialize_with_data() {
        let mut activity = create_test_activity(ActivityType::ProfileUpdate);
        activity.data.insert(
            "field".to_string(),
            serde_json::json!("display_name"),
        );

        let json = serde_json::to_string(&activity).unwrap();
        assert!(json.contains("display_name"));
    }

    // ==================== UserActivity Clone Tests ====================

    #[test]
    fn test_user_activity_clone() {
        let mut activity = create_test_activity(ActivityType::TeamJoined);
        activity.ip_address = Some("1.2.3.4".to_string());
        activity.data.insert("team_id".to_string(), serde_json::json!("team-123"));

        let cloned = activity.clone();

        assert_eq!(activity.user_id, cloned.user_id);
        assert_eq!(activity.ip_address, cloned.ip_address);
        assert_eq!(activity.data.len(), cloned.data.len());
    }

    // ==================== UserActivity Debug Tests ====================

    #[test]
    fn test_user_activity_debug() {
        let activity = create_test_activity(ActivityType::SecurityEvent);

        let debug_str = format!("{:?}", activity);
        assert!(debug_str.contains("UserActivity"));
        assert!(debug_str.contains("SecurityEvent"));
    }

    // ==================== All Activity Types Tests ====================

    #[test]
    fn test_all_activity_types_serialize() {
        let types = vec![
            ActivityType::Login,
            ActivityType::Logout,
            ActivityType::PasswordChange,
            ActivityType::ProfileUpdate,
            ActivityType::ApiKeyCreated,
            ActivityType::ApiKeyDeleted,
            ActivityType::TeamJoined,
            ActivityType::TeamLeft,
            ActivityType::SettingsChanged,
            ActivityType::SecurityEvent,
        ];

        for t in types {
            let activity = create_test_activity(t);
            let json = serde_json::to_string(&activity);
            assert!(json.is_ok());
        }
    }

    #[test]
    fn test_all_activity_types_deserialize() {
        let type_strings = vec![
            "\"login\"",
            "\"logout\"",
            "\"password_change\"",
            "\"profile_update\"",
            "\"api_key_created\"",
            "\"api_key_deleted\"",
            "\"team_joined\"",
            "\"team_left\"",
            "\"settings_changed\"",
            "\"security_event\"",
        ];

        for s in type_strings {
            let result: Result<ActivityType, _> = serde_json::from_str(s);
            assert!(result.is_ok());
        }
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_user_activity_empty_description() {
        let mut activity = create_test_activity(ActivityType::Login);
        activity.description = "".to_string();

        assert!(activity.description.is_empty());
    }

    #[test]
    fn test_user_activity_long_description() {
        let mut activity = create_test_activity(ActivityType::SecurityEvent);
        activity.description = "a".repeat(10000);

        assert_eq!(activity.description.len(), 10000);
    }

    #[test]
    fn test_user_activity_special_characters_in_user_agent() {
        let mut activity = create_test_activity(ActivityType::Login);
        activity.user_agent = Some("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36".to_string());

        let json = serde_json::to_string(&activity).unwrap();
        let deserialized: UserActivity = serde_json::from_str(&json).unwrap();

        assert_eq!(activity.user_agent, deserialized.user_agent);
    }

    #[test]
    fn test_user_activity_ipv6_address() {
        let mut activity = create_test_activity(ActivityType::Login);
        activity.ip_address = Some("2001:0db8:85a3:0000:0000:8a2e:0370:7334".to_string());

        let json = serde_json::to_string(&activity).unwrap();
        assert!(json.contains("2001:0db8"));
    }

    #[test]
    fn test_user_activity_complex_data() {
        let mut activity = create_test_activity(ActivityType::SettingsChanged);

        activity.data.insert(
            "changes".to_string(),
            serde_json::json!({
                "before": {"theme": "light", "language": "en"},
                "after": {"theme": "dark", "language": "zh"}
            }),
        );

        let json = serde_json::to_string(&activity).unwrap();
        let deserialized: UserActivity = serde_json::from_str(&json).unwrap();

        assert_eq!(activity.data.len(), deserialized.data.len());
    }

    #[test]
    fn test_user_activity_array_data() {
        let mut activity = create_test_activity(ActivityType::ApiKeyCreated);

        activity.data.insert(
            "permissions".to_string(),
            serde_json::json!(["read", "write", "delete"]),
        );

        let json = serde_json::to_string(&activity).unwrap();
        assert!(json.contains("[\"read\""));
    }
}
