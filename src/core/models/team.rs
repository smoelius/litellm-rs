//! Team models for the Gateway
//!
//! This module defines team-related data structures.

use super::{Metadata, UsageStats, UserRateLimits};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Team/Organization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    /// Team metadata
    #[serde(flatten)]
    pub metadata: Metadata,
    /// Team name (unique)
    pub name: String,
    /// Team display name
    pub display_name: Option<String>,
    /// Team description
    pub description: Option<String>,
    /// Team status
    pub status: TeamStatus,
    /// Team settings
    pub settings: TeamSettings,
    /// Usage statistics
    pub usage_stats: UsageStats,
    /// Team rate limits
    pub rate_limits: Option<UserRateLimits>,
    /// Billing information
    pub billing: Option<TeamBilling>,
    /// Team metadata
    pub team_metadata: HashMap<String, serde_json::Value>,
}

/// Team status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TeamStatus {
    /// Active team
    Active,
    /// Inactive team
    Inactive,
    /// Suspended team
    Suspended,
    /// Deleted team (soft delete)
    Deleted,
}

/// Team settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TeamSettings {
    /// Default user role for new members
    pub default_member_role: Option<String>,
    /// Require approval for new members
    pub require_approval: bool,
    /// Allow members to invite others
    pub allow_member_invites: bool,
    /// Team visibility
    pub visibility: TeamVisibility,
    /// API access settings
    pub api_access: ApiAccessSettings,
    /// Notification settings
    pub notifications: TeamNotificationSettings,
    /// Security settings
    pub security: TeamSecuritySettings,
}

/// Team visibility
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TeamVisibility {
    /// Public team
    Public,
    /// Private team
    Private,
    /// Internal team
    Internal,
}

impl Default for TeamVisibility {
    fn default() -> Self {
        Self::Private
    }
}

/// API access settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiAccessSettings {
    /// Enable API access
    pub enabled: bool,
    /// Allowed IP addresses
    pub allowed_ips: Vec<String>,
    /// Allowed domains
    pub allowed_domains: Vec<String>,
    /// Require API key authentication
    pub require_api_key: bool,
    /// Default API settings
    pub default_settings: HashMap<String, serde_json::Value>,
}

/// Team notification settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TeamNotificationSettings {
    /// Slack webhook URL
    pub slack_webhook: Option<String>,
    /// Email notifications
    pub email_notifications: bool,
    /// Webhook notifications
    pub webhook_notifications: bool,
    /// Notification channels
    pub channels: Vec<NotificationChannel>,
}

/// Notification channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationChannel {
    /// Channel name
    pub name: String,
    /// Channel type
    pub channel_type: ChannelType,
    /// Channel configuration
    pub config: HashMap<String, serde_json::Value>,
    /// Enabled
    pub enabled: bool,
}

/// Channel type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChannelType {
    /// Email channel
    Email,
    /// Slack channel
    Slack,
    /// Webhook channel
    Webhook,
    /// Microsoft Teams channel
    Teams,
    /// Discord channel
    Discord,
}

/// Team security settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TeamSecuritySettings {
    /// Require two-factor authentication
    pub require_2fa: bool,
    /// Password policy
    pub password_policy: PasswordPolicy,
    /// Session timeout in minutes
    pub session_timeout: Option<u32>,
    /// IP whitelist
    pub ip_whitelist: Vec<String>,
    /// Audit logging enabled
    pub audit_logging: bool,
}

/// Password policy
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PasswordPolicy {
    /// Minimum length
    pub min_length: u32,
    /// Require uppercase
    pub require_uppercase: bool,
    /// Require lowercase
    pub require_lowercase: bool,
    /// Require numbers
    pub require_numbers: bool,
    /// Require special characters
    pub require_special: bool,
    /// Password expiry in days
    pub expiry_days: Option<u32>,
}

/// Team billing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamBilling {
    /// Billing plan
    pub plan: BillingPlan,
    /// Billing status
    pub status: BillingStatus,
    /// Monthly budget limit
    pub monthly_budget: Option<f64>,
    /// Current month usage
    pub current_usage: f64,
    /// Billing cycle start
    pub cycle_start: chrono::DateTime<chrono::Utc>,
    /// Billing cycle end
    pub cycle_end: chrono::DateTime<chrono::Utc>,
    /// Payment method
    pub payment_method: Option<PaymentMethod>,
    /// Billing address
    pub billing_address: Option<BillingAddress>,
}

/// Billing plan
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BillingPlan {
    /// Free plan
    Free,
    /// Starter plan
    Starter,
    /// Professional plan
    Professional,
    /// Enterprise plan
    Enterprise,
    /// Custom plan
    Custom,
}

/// Billing status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BillingStatus {
    /// Active billing status
    Active,
    /// Past due billing status
    PastDue,
    /// Cancelled billing status
    Cancelled,
    /// Suspended billing status
    Suspended,
    /// Trial billing status
    Trial,
}

/// Payment method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentMethod {
    /// Payment method type
    pub method_type: PaymentMethodType,
    /// Last 4 digits (for cards)
    pub last_four: Option<String>,
    /// Expiry month (for cards)
    pub expiry_month: Option<u32>,
    /// Expiry year (for cards)
    pub expiry_year: Option<u32>,
    /// Brand (for cards)
    pub brand: Option<String>,
}

/// Payment method type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentMethodType {
    /// Credit card payment
    CreditCard,
    /// Debit card payment
    DebitCard,
    /// Bank transfer payment
    BankTransfer,
    /// PayPal payment
    PayPal,
    /// Stripe payment
    Stripe,
}

/// Billing address
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingAddress {
    /// Company name
    pub company: Option<String>,
    /// Address line 1
    pub line1: String,
    /// Address line 2
    pub line2: Option<String>,
    /// City
    pub city: String,
    /// State/Province
    pub state: Option<String>,
    /// Postal code
    pub postal_code: String,
    /// Country
    pub country: String,
    /// Tax ID
    pub tax_id: Option<String>,
}

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

impl Team {
    /// Create a new team
    pub fn new(name: String, display_name: Option<String>) -> Self {
        Self {
            metadata: Metadata::new(),
            name,
            display_name,
            description: None,
            status: TeamStatus::Active,
            settings: TeamSettings::default(),
            usage_stats: UsageStats::default(),
            rate_limits: None,
            billing: None,
            team_metadata: HashMap::new(),
        }
    }

    /// Get team ID
    pub fn id(&self) -> Uuid {
        self.metadata.id
    }

    /// Check if team is active
    pub fn is_active(&self) -> bool {
        matches!(self.status, TeamStatus::Active)
    }

    /// Update usage statistics
    pub fn update_usage(&mut self, requests: u64, tokens: u64, cost: f64) {
        self.usage_stats.total_requests += requests;
        self.usage_stats.total_tokens += tokens;
        self.usage_stats.total_cost += cost;

        // Update daily stats
        let today = chrono::Utc::now().date_naive();
        let last_reset = self.usage_stats.last_reset.date_naive();

        if today != last_reset {
            self.usage_stats.requests_today = 0;
            self.usage_stats.tokens_today = 0;
            self.usage_stats.cost_today = 0.0;
            self.usage_stats.last_reset = chrono::Utc::now();
        }

        self.usage_stats.requests_today += requests as u32;
        self.usage_stats.tokens_today += tokens as u32;
        self.usage_stats.cost_today += cost;

        // Update billing usage if applicable
        if let Some(billing) = &mut self.billing {
            billing.current_usage += cost;
        }

        self.metadata.touch();
    }

    /// Check if team is over budget
    pub fn is_over_budget(&self) -> bool {
        if let Some(billing) = &self.billing {
            if let Some(budget) = billing.monthly_budget {
                return billing.current_usage >= budget;
            }
        }
        false
    }

    /// Get remaining budget
    pub fn remaining_budget(&self) -> Option<f64> {
        if let Some(billing) = &self.billing {
            if let Some(budget) = billing.monthly_budget {
                return Some((budget - billing.current_usage).max(0.0));
            }
        }
        None
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_team_creation() {
        let team = Team::new("test-team".to_string(), Some("Test Team".to_string()));

        assert_eq!(team.name, "test-team");
        assert_eq!(team.display_name, Some("Test Team".to_string()));
        assert!(team.is_active());
    }

    #[test]
    fn test_team_usage_update() {
        let mut team = Team::new("test-team".to_string(), None);

        team.update_usage(10, 1000, 0.50);

        assert_eq!(team.usage_stats.total_requests, 10);
        assert_eq!(team.usage_stats.total_tokens, 1000);
        assert_eq!(team.usage_stats.total_cost, 0.50);
    }

    #[test]
    fn test_team_member_permissions() {
        let team_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let mut member = TeamMember::new(team_id, user_id, TeamRole::Member, None);

        assert!(!member.has_permission("admin"));

        member.add_permission("admin".to_string());
        assert!(member.has_permission("admin"));

        member.remove_permission("admin");
        assert!(!member.has_permission("admin"));
    }

    #[test]
    fn test_invitation_expiry() {
        let team_id = Uuid::new_v4();
        let invited_by = Uuid::new_v4();
        let expires_at = chrono::Utc::now() - chrono::Duration::hours(1); // Expired

        let invitation = TeamInvitation::new(
            team_id,
            "test@example.com".to_string(),
            TeamRole::Member,
            "token".to_string(),
            invited_by,
            expires_at,
        );

        assert!(invitation.is_expired());
    }
}
