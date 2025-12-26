//! User session management

use crate::core::models::Metadata;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

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
    use chrono::{Duration, Utc};

    // ==================== SessionType Tests ====================

    #[test]
    fn test_session_type_web() {
        let session_type = SessionType::Web;
        let json = serde_json::to_string(&session_type).unwrap();
        assert_eq!(json, "\"web\"");
    }

    #[test]
    fn test_session_type_api() {
        let session_type = SessionType::Api;
        let json = serde_json::to_string(&session_type).unwrap();
        assert_eq!(json, "\"api\"");
    }

    #[test]
    fn test_session_type_mobile() {
        let session_type = SessionType::Mobile;
        let json = serde_json::to_string(&session_type).unwrap();
        assert_eq!(json, "\"mobile\"");
    }

    #[test]
    fn test_session_type_cli() {
        let session_type = SessionType::Cli;
        let json = serde_json::to_string(&session_type).unwrap();
        assert_eq!(json, "\"cli\"");
    }

    #[test]
    fn test_session_type_deserialize() {
        let web: SessionType = serde_json::from_str("\"web\"").unwrap();
        assert!(matches!(web, SessionType::Web));

        let api: SessionType = serde_json::from_str("\"api\"").unwrap();
        assert!(matches!(api, SessionType::Api));
    }

    #[test]
    fn test_session_type_clone() {
        let original = SessionType::Mobile;
        let cloned = original.clone();
        let json1 = serde_json::to_string(&original).unwrap();
        let json2 = serde_json::to_string(&cloned).unwrap();
        assert_eq!(json1, json2);
    }

    #[test]
    fn test_session_type_debug() {
        let session_type = SessionType::Cli;
        let debug_str = format!("{:?}", session_type);
        assert!(debug_str.contains("Cli"));
    }

    // ==================== UserSession Creation Tests ====================

    #[test]
    fn test_user_session_new() {
        let user_id = Uuid::new_v4();
        let token = "test_token_123".to_string();
        let expires_at = Utc::now() + Duration::hours(24);

        let session = UserSession::new(user_id, token.clone(), SessionType::Web, expires_at);

        assert_eq!(session.user_id, user_id);
        assert_eq!(session.token, token);
        assert!(matches!(session.session_type, SessionType::Web));
        assert_eq!(session.expires_at, expires_at);
        assert!(session.ip_address.is_none());
        assert!(session.user_agent.is_none());
        assert!(session.data.is_empty());
    }

    #[test]
    fn test_user_session_new_with_api_type() {
        let user_id = Uuid::new_v4();
        let expires_at = Utc::now() + Duration::days(30);

        let session = UserSession::new(
            user_id,
            "api_token".to_string(),
            SessionType::Api,
            expires_at,
        );

        assert!(matches!(session.session_type, SessionType::Api));
    }

    #[test]
    fn test_user_session_new_with_mobile_type() {
        let user_id = Uuid::new_v4();
        let expires_at = Utc::now() + Duration::days(7);

        let session = UserSession::new(
            user_id,
            "mobile_token".to_string(),
            SessionType::Mobile,
            expires_at,
        );

        assert!(matches!(session.session_type, SessionType::Mobile));
    }

    #[test]
    fn test_user_session_new_with_cli_type() {
        let user_id = Uuid::new_v4();
        let expires_at = Utc::now() + Duration::hours(1);

        let session = UserSession::new(
            user_id,
            "cli_token".to_string(),
            SessionType::Cli,
            expires_at,
        );

        assert!(matches!(session.session_type, SessionType::Cli));
    }

    // ==================== UserSession Expiration Tests ====================

    #[test]
    fn test_user_session_not_expired() {
        let user_id = Uuid::new_v4();
        let expires_at = Utc::now() + Duration::hours(24);

        let session = UserSession::new(
            user_id,
            "token".to_string(),
            SessionType::Web,
            expires_at,
        );

        assert!(!session.is_expired());
    }

    #[test]
    fn test_user_session_expired() {
        let user_id = Uuid::new_v4();
        let expires_at = Utc::now() - Duration::hours(1);

        let session = UserSession::new(
            user_id,
            "token".to_string(),
            SessionType::Web,
            expires_at,
        );

        assert!(session.is_expired());
    }

    #[test]
    fn test_user_session_just_expired() {
        let user_id = Uuid::new_v4();
        let expires_at = Utc::now() - Duration::seconds(1);

        let session = UserSession::new(
            user_id,
            "token".to_string(),
            SessionType::Web,
            expires_at,
        );

        assert!(session.is_expired());
    }

    // ==================== UserSession Activity Tests ====================

    #[test]
    fn test_user_session_update_activity() {
        let user_id = Uuid::new_v4();
        let expires_at = Utc::now() + Duration::hours(24);

        let mut session = UserSession::new(
            user_id,
            "token".to_string(),
            SessionType::Web,
            expires_at,
        );

        let initial_activity = session.last_activity;
        std::thread::sleep(std::time::Duration::from_millis(10));
        session.update_activity();

        assert!(session.last_activity >= initial_activity);
    }

    // ==================== UserSession Data Tests ====================

    #[test]
    fn test_user_session_set_data() {
        let user_id = Uuid::new_v4();
        let expires_at = Utc::now() + Duration::hours(24);

        let mut session = UserSession::new(
            user_id,
            "token".to_string(),
            SessionType::Web,
            expires_at,
        );

        session.set_data("key1", "value1");
        session.set_data("key2", 42);
        session.set_data("key3", true);

        assert_eq!(session.data.len(), 3);
    }

    #[test]
    fn test_user_session_get_data() {
        let user_id = Uuid::new_v4();
        let expires_at = Utc::now() + Duration::hours(24);

        let mut session = UserSession::new(
            user_id,
            "token".to_string(),
            SessionType::Web,
            expires_at,
        );

        session.set_data("test_key", "test_value");

        let value = session.get_data("test_key");
        assert!(value.is_some());
        assert_eq!(value.unwrap(), "test_value");
    }

    #[test]
    fn test_user_session_get_data_missing() {
        let user_id = Uuid::new_v4();
        let expires_at = Utc::now() + Duration::hours(24);

        let session = UserSession::new(
            user_id,
            "token".to_string(),
            SessionType::Web,
            expires_at,
        );

        assert!(session.get_data("nonexistent").is_none());
    }

    #[test]
    fn test_user_session_set_data_overwrite() {
        let user_id = Uuid::new_v4();
        let expires_at = Utc::now() + Duration::hours(24);

        let mut session = UserSession::new(
            user_id,
            "token".to_string(),
            SessionType::Web,
            expires_at,
        );

        session.set_data("key", "original");
        session.set_data("key", "updated");

        let value = session.get_data("key");
        assert_eq!(value.unwrap(), "updated");
    }

    #[test]
    fn test_user_session_set_data_various_types() {
        let user_id = Uuid::new_v4();
        let expires_at = Utc::now() + Duration::hours(24);

        let mut session = UserSession::new(
            user_id,
            "token".to_string(),
            SessionType::Web,
            expires_at,
        );

        session.set_data("string", "hello");
        session.set_data("number", 123);
        session.set_data("float", 3.14);
        session.set_data("bool", true);
        session.set_data("null", serde_json::Value::Null);

        assert_eq!(session.data.len(), 5);
    }

    // ==================== UserSession Serialization Tests ====================

    #[test]
    fn test_user_session_serialize() {
        let user_id = Uuid::new_v4();
        let expires_at = Utc::now() + Duration::hours(24);

        let session = UserSession::new(
            user_id,
            "secret_token".to_string(),
            SessionType::Web,
            expires_at,
        );

        let json = serde_json::to_string(&session).unwrap();

        // Token should NOT be serialized (skip_serializing)
        assert!(!json.contains("secret_token"));
        // User ID should be serialized
        assert!(json.contains(&user_id.to_string()));
        // Session type should be serialized
        assert!(json.contains("\"session_type\":\"web\""));
    }

    #[test]
    fn test_user_session_serialize_with_optional_fields() {
        let user_id = Uuid::new_v4();
        let expires_at = Utc::now() + Duration::hours(24);

        let mut session = UserSession::new(
            user_id,
            "token".to_string(),
            SessionType::Api,
            expires_at,
        );

        session.ip_address = Some("192.168.1.1".to_string());
        session.user_agent = Some("Mozilla/5.0".to_string());

        let json = serde_json::to_string(&session).unwrap();

        assert!(json.contains("192.168.1.1"));
        assert!(json.contains("Mozilla/5.0"));
    }

    // ==================== UserSession Clone Tests ====================

    #[test]
    fn test_user_session_clone() {
        let user_id = Uuid::new_v4();
        let expires_at = Utc::now() + Duration::hours(24);

        let mut session = UserSession::new(
            user_id,
            "token".to_string(),
            SessionType::Mobile,
            expires_at,
        );

        session.ip_address = Some("10.0.0.1".to_string());
        session.set_data("key", "value");

        let cloned = session.clone();

        assert_eq!(session.user_id, cloned.user_id);
        assert_eq!(session.token, cloned.token);
        assert_eq!(session.ip_address, cloned.ip_address);
        assert_eq!(session.data.len(), cloned.data.len());
    }

    // ==================== UserSession Debug Tests ====================

    #[test]
    fn test_user_session_debug() {
        let user_id = Uuid::new_v4();
        let expires_at = Utc::now() + Duration::hours(24);

        let session = UserSession::new(
            user_id,
            "token".to_string(),
            SessionType::Cli,
            expires_at,
        );

        let debug_str = format!("{:?}", session);
        assert!(debug_str.contains("UserSession"));
        assert!(debug_str.contains("Cli"));
    }

    // ==================== UserSession Edge Cases ====================

    #[test]
    fn test_user_session_empty_token() {
        let user_id = Uuid::new_v4();
        let expires_at = Utc::now() + Duration::hours(24);

        let session = UserSession::new(
            user_id,
            "".to_string(),
            SessionType::Web,
            expires_at,
        );

        assert!(session.token.is_empty());
    }

    #[test]
    fn test_user_session_long_token() {
        let user_id = Uuid::new_v4();
        let expires_at = Utc::now() + Duration::hours(24);
        let long_token = "a".repeat(1000);

        let session = UserSession::new(
            user_id,
            long_token.clone(),
            SessionType::Web,
            expires_at,
        );

        assert_eq!(session.token.len(), 1000);
    }

    #[test]
    fn test_user_session_far_future_expiry() {
        let user_id = Uuid::new_v4();
        let expires_at = Utc::now() + Duration::days(365 * 10);

        let session = UserSession::new(
            user_id,
            "token".to_string(),
            SessionType::Api,
            expires_at,
        );

        assert!(!session.is_expired());
    }
}
