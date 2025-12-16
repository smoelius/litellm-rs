//! Team invitation models

use super::member::TeamRole;
use crate::core::models::Metadata;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Team invitation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamInvitation {
    /// Invitation metadata
    #[serde(flatten)]
    pub metadata: Metadata,
    /// Team ID
    pub team_id: Uuid,
    /// Email address
    pub email: String,
    /// Invited role
    pub role: TeamRole,
    /// Invitation token
    #[serde(skip_serializing)]
    pub token: String,
    /// Invited by
    pub invited_by: Uuid,
    /// Expires at
    pub expires_at: chrono::DateTime<chrono::Utc>,
    /// Invitation status
    pub status: InvitationStatus,
}

/// Invitation status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InvitationStatus {
    /// Pending acceptance
    Pending,
    /// Accepted
    Accepted,
    /// Declined
    Declined,
    /// Expired
    Expired,
    /// Cancelled
    Cancelled,
}

impl TeamInvitation {
    /// Create a new invitation
    pub fn new(
        team_id: Uuid,
        email: String,
        role: TeamRole,
        token: String,
        invited_by: Uuid,
        expires_at: chrono::DateTime<chrono::Utc>,
    ) -> Self {
        Self {
            metadata: Metadata::new(),
            team_id,
            email,
            role,
            token,
            invited_by,
            expires_at,
            status: InvitationStatus::Pending,
        }
    }

    /// Check if invitation is expired
    pub fn is_expired(&self) -> bool {
        chrono::Utc::now() > self.expires_at
    }

    /// Accept invitation
    pub fn accept(&mut self) {
        self.status = InvitationStatus::Accepted;
        self.metadata.touch();
    }

    /// Decline invitation
    pub fn decline(&mut self) {
        self.status = InvitationStatus::Declined;
        self.metadata.touch();
    }

    /// Cancel invitation
    pub fn cancel(&mut self) {
        self.status = InvitationStatus::Cancelled;
        self.metadata.touch();
    }
}
