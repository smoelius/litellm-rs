//! RBAC type definitions

use std::collections::HashSet;

/// Role definition
#[derive(Debug, Clone)]
pub struct Role {
    /// Role name
    pub name: String,
    /// Role description
    pub description: String,
    /// Permissions granted by this role
    pub permissions: HashSet<String>,
    /// Parent roles (inheritance)
    pub parent_roles: HashSet<String>,
    /// Whether this is a system role
    pub is_system: bool,
}

/// Permission definition
#[derive(Debug, Clone)]
pub struct Permission {
    /// Permission name
    pub name: String,
    /// Permission description
    pub description: String,
    /// Resource this permission applies to
    pub resource: String,
    /// Action this permission allows
    pub action: String,
    /// Whether this is a system permission
    pub is_system: bool,
}

/// Permission check result
#[derive(Debug, Clone)]
pub struct PermissionCheck {
    /// Whether permission is granted
    pub granted: bool,
    /// Roles that granted the permission
    pub granted_by_roles: Vec<String>,
    /// Reason for denial (if not granted)
    pub denial_reason: Option<String>,
}
