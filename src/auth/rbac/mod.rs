//! Role-Based Access Control (RBAC) system
//!
//! This module provides comprehensive RBAC functionality for authorization.

mod helpers;
mod permissions;
mod roles;
mod system;
#[cfg(test)]
mod tests;
mod types;

// Re-export public types and structs
pub use system::RbacSystem;
pub use types::{Permission, PermissionCheck, Role};
