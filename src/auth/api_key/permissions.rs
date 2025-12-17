//! API key permission checking utilities
//!
//! This module provides permission checking functionality for API keys.

use super::creation::ApiKeyHandler;
use crate::core::models::ApiKey;

impl ApiKeyHandler {
    /// Check if API key has permission
    pub fn has_permission(&self, api_key: &ApiKey, permission: &str) -> bool {
        api_key.permissions.contains(&permission.to_string())
            || api_key.permissions.contains(&"*".to_string()) // Wildcard permission
    }

    /// Check if API key has any of the permissions
    pub fn has_any_permission(&self, api_key: &ApiKey, permissions: &[String]) -> bool {
        if api_key.permissions.contains(&"*".to_string()) {
            return true;
        }

        permissions
            .iter()
            .any(|perm| api_key.permissions.contains(perm))
    }

    /// Check if API key has all permissions
    pub fn has_all_permissions(&self, api_key: &ApiKey, permissions: &[String]) -> bool {
        if api_key.permissions.contains(&"*".to_string()) {
            return true;
        }

        permissions
            .iter()
            .all(|perm| api_key.permissions.contains(perm))
    }
}
