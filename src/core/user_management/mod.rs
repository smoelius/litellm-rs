//! User and Team management system
//!
//! This module provides comprehensive user and team management for enterprise features.

mod manager;
mod roles;
mod settings;
mod team_ops;
#[cfg(test)]
mod tests;
mod types;
mod user_ops;

// Re-export all public types for backward compatibility
pub use manager::UserManager;
pub use roles::{TeamRole, UserRole};
pub use settings::{
    OrganizationSettings, PasswordPolicy, SSOConfig, SSOProvider, TeamSettings, UserPreferences,
};
pub use types::{Organization, Team, TeamMember, User};
