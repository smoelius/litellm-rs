//! Authentication and authorization system
//!
//! This module provides comprehensive authentication and authorization functionality.

#![allow(dead_code)]

// Submodules
pub mod api_key;
pub mod jwt;
pub mod rbac;

// Internal submodules
mod api_keys;
mod password;
mod system;
#[cfg(test)]
mod tests;
mod types;
mod user_management;

// Re-export commonly used types from core models
pub use crate::core::models::{ApiKey, User, UserRole, UserSession};

// Re-export types from submodules
pub use system::AuthSystem;
pub use types::{AuthMethod, AuthResult, AuthzResult};
