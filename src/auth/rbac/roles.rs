//! Role management methods

use crate::core::models::TeamRole;
use crate::core::models::user::types::UserRole;
use crate::utils::error::{GatewayError, Result};

use super::system::RbacSystem;
use super::types::Role;

impl RbacSystem {
    /// Get role by name
    pub fn get_role(&self, role_name: &str) -> Option<&Role> {
        self.roles.get(role_name)
    }

    /// Add custom role
    pub fn add_role(&mut self, role: Role) -> Result<()> {
        if role.is_system {
            return Err(GatewayError::auth("Cannot modify system roles"));
        }

        self.roles.insert(role.name.clone(), role);
        Ok(())
    }

    /// Convert UserRole to string
    pub(super) fn user_role_to_string(&self, role: &UserRole) -> String {
        match role {
            UserRole::SuperAdmin => "super_admin".to_string(),
            UserRole::Admin => "admin".to_string(),
            UserRole::Manager => "manager".to_string(),
            UserRole::User => "user".to_string(),
            UserRole::Viewer => "viewer".to_string(),
            UserRole::ApiUser => "api_user".to_string(),
        }
    }

    /// Convert TeamRole to permissions
    pub fn team_role_to_permissions(&self, role: &TeamRole) -> Vec<String> {
        match role {
            TeamRole::Owner => vec![
                "teams.read".to_string(),
                "teams.write".to_string(),
                "teams.delete".to_string(),
                "users.read".to_string(),
                "users.write".to_string(),
                "api_keys.read".to_string(),
                "api_keys.write".to_string(),
                "api_keys.delete".to_string(),
                "analytics.read".to_string(),
            ],
            TeamRole::Admin => vec![
                "teams.read".to_string(),
                "teams.write".to_string(),
                "users.read".to_string(),
                "users.write".to_string(),
                "api_keys.read".to_string(),
                "api_keys.write".to_string(),
                "analytics.read".to_string(),
            ],
            TeamRole::Manager => vec![
                "teams.read".to_string(),
                "users.read".to_string(),
                "api_keys.read".to_string(),
                "api_keys.write".to_string(),
                "analytics.read".to_string(),
            ],
            TeamRole::Member => vec![
                "teams.read".to_string(),
                "api_keys.read".to_string(),
                "analytics.read".to_string(),
            ],
            TeamRole::Viewer => vec!["teams.read".to_string(), "analytics.read".to_string()],
        }
    }
}
