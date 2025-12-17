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
