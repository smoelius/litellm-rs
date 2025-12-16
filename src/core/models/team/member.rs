//! Team member models

use crate::core::models::Metadata;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Team member
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMember {
    /// Member metadata
    #[serde(flatten)]
    pub metadata: Metadata,
    /// Team ID
    pub team_id: Uuid,
    /// User ID
    pub user_id: Uuid,
    /// Member role
    pub role: TeamRole,
    /// Member status
    pub status: MemberStatus,
    /// Joined at
    pub joined_at: chrono::DateTime<chrono::Utc>,
    /// Invited by
    pub invited_by: Option<Uuid>,
    /// Member permissions
    pub permissions: Vec<String>,
}

/// Team role
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TeamRole {
    /// Team owner
    Owner,
    /// Team admin
    Admin,
    /// Team manager
    Manager,
    /// Team member
    Member,
    /// Read-only member
    Viewer,
}

/// Member status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemberStatus {
    /// Active member
    Active,
    /// Pending invitation
    Pending,
    /// Suspended member
    Suspended,
    /// Left team
    Left,
}

impl TeamMember {
    /// Create a new team member
    pub fn new(team_id: Uuid, user_id: Uuid, role: TeamRole, invited_by: Option<Uuid>) -> Self {
        Self {
            metadata: Metadata::new(),
            team_id,
            user_id,
            role,
            status: MemberStatus::Active,
            joined_at: chrono::Utc::now(),
            invited_by,
            permissions: vec![],
        }
    }

    /// Check if member is active
    pub fn is_active(&self) -> bool {
        matches!(self.status, MemberStatus::Active)
    }

    /// Check if member has permission
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.contains(&permission.to_string())
    }

    /// Add permission
    pub fn add_permission(&mut self, permission: String) {
        if !self.permissions.contains(&permission) {
            self.permissions.push(permission);
            self.metadata.touch();
        }
    }

    /// Remove permission
    pub fn remove_permission(&mut self, permission: &str) {
        if let Some(pos) = self.permissions.iter().position(|p| p == permission) {
            self.permissions.remove(pos);
            self.metadata.touch();
        }
    }
}
