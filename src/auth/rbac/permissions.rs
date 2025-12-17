//! Permission checking methods

use crate::core::models::user::types::User;
use crate::utils::error::{GatewayError, Result};
use std::collections::HashSet;

use super::helpers::RbacHelpers;
use super::system::RbacSystem;
use super::types::{Permission, PermissionCheck};

impl RbacSystem {
    /// Get all permissions for a user
    pub async fn get_user_permissions(&self, user: &User) -> Result<Vec<String>> {
        let mut permissions = HashSet::new();

        // Get permissions from user role
        let role_name = self.user_role_to_string(&user.role);
        if let Some(role) = self.roles.get(&role_name) {
            permissions.extend(self.get_role_permissions(role));
        }

        Ok(permissions.into_iter().collect())
    }

    /// Check if user has specific permissions
    pub fn check_permissions(
        &self,
        user_permissions: &[String],
        required_permissions: &[String],
    ) -> bool {
        let user_perms: HashSet<&String> = user_permissions.iter().collect();

        // Check for wildcard permission
        if user_perms
            .iter()
            .any(|p| p.as_str() == "*" || p.as_str() == "system.admin")
        {
            return true;
        }

        // Check if user has all required permissions
        required_permissions
            .iter()
            .all(|perm| user_perms.contains(&perm))
    }

    /// Check if user has any of the required permissions
    pub fn check_any_permission(
        &self,
        user_permissions: &[String],
        required_permissions: &[String],
    ) -> bool {
        let user_perms: HashSet<&String> = user_permissions.iter().collect();

        // Check for wildcard permission
        if user_perms
            .iter()
            .any(|p| p.as_str() == "*" || p.as_str() == "system.admin")
        {
            return true;
        }

        // Check if user has any of the required permissions
        required_permissions
            .iter()
            .any(|perm| user_perms.contains(&perm))
    }

    /// Detailed permission check
    pub async fn check_permission_detailed(
        &self,
        user: &User,
        required_permission: &str,
    ) -> Result<PermissionCheck> {
        let user_permissions = self.get_user_permissions(user).await?;
        let user_perms: HashSet<&String> = user_permissions.iter().collect();

        // Check for wildcard or admin permission
        if user_perms
            .iter()
            .any(|p| p.as_str() == "*" || p.as_str() == "system.admin")
        {
            return Ok(PermissionCheck {
                granted: true,
                granted_by_roles: vec![self.user_role_to_string(&user.role)],
                denial_reason: None,
            });
        }

        // Check specific permission
        if user_perms.iter().any(|p| p.as_str() == required_permission) {
            Ok(PermissionCheck {
                granted: true,
                granted_by_roles: vec![self.user_role_to_string(&user.role)],
                denial_reason: None,
            })
        } else {
            Ok(PermissionCheck {
                granted: false,
                granted_by_roles: vec![],
                denial_reason: Some(format!("Missing permission: {}", required_permission)),
            })
        }
    }

    /// Check resource-level permissions
    pub fn check_resource_permission(
        &self,
        user_permissions: &[String],
        resource: &str,
        action: &str,
    ) -> bool {
        let required_permission = format!("{}.{}", resource, action);
        self.check_permissions(user_permissions, &[required_permission])
    }

    /// Check if user is admin
    pub fn is_admin(&self, user: &User) -> bool {
        let role_name = self.user_role_to_string(&user.role);
        self.config.admin_roles.contains(&role_name)
    }

    /// Get permission by name
    pub fn get_permission(&self, permission_name: &str) -> Option<&Permission> {
        self.permissions.get(permission_name)
    }

    /// Add custom permission
    pub fn add_permission(&mut self, permission: Permission) -> Result<()> {
        if permission.is_system {
            return Err(GatewayError::auth("Cannot modify system permissions"));
        }

        self.permissions.insert(permission.name.clone(), permission);
        Ok(())
    }
}
