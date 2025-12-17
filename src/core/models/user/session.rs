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
