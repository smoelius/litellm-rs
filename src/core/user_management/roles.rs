//! User and team role definitions

use serde::{Deserialize, Serialize};

/// User roles
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum UserRole {
    /// Super admin with all permissions
    SuperAdmin,
    /// Organization admin
    OrgAdmin,
    /// Team admin
    TeamAdmin,
    /// Regular user
    User,
    /// Read-only user
    ReadOnly,
    /// Service account
    ServiceAccount,
}

/// Team roles
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TeamRole {
    /// Team owner
    Owner,
    /// Team admin
    Admin,
    /// Team member
    Member,
    /// Read-only member
    ReadOnly,
}
