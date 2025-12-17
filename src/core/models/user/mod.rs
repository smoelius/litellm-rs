//! User models for the Gateway
//!
//! This module defines user-related data structures.

mod activity;
mod preferences;
mod session;
mod types;

#[cfg(test)]
mod tests;

// Re-export all public types for backward compatibility
pub use activity::{ActivityType, UserActivity};
pub use preferences::{
    ApiPreferences, DashboardSettings, NotificationSettings, NotificationType, UserPreferences,
};
pub use session::{SessionType, UserSession};
pub use types::{User, UserProfile, UserRateLimits, UserRole, UserStatus};
