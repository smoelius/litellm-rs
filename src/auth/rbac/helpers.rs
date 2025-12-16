//! Helper methods for RBAC operations

use std::collections::HashSet;

use super::system::RbacSystem;
use super::types::Role;

pub(super) trait RbacHelpers {
    /// Get all permissions for a role (including inherited)
    fn get_role_permissions(&self, role: &Role) -> HashSet<String>;
}

impl RbacHelpers for RbacSystem {
    /// Get all permissions for a role (including inherited)
    fn get_role_permissions(&self, role: &Role) -> HashSet<String> {
        let mut permissions = role.permissions.clone();

        // Add permissions from parent roles
        for parent_role_name in &role.parent_roles {
            if let Some(parent_role) = self.roles.get(parent_role_name) {
                permissions.extend(self.get_role_permissions(parent_role));
            }
        }

        permissions
    }
}
