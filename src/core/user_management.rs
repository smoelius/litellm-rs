//! User and Team management system
//!
//! This module provides comprehensive user and team management for enterprise features.

use crate::storage::database::Database;
use crate::utils::error::{GatewayError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// User entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Unique user identifier
    pub user_id: String,
    /// User email
    pub email: String,
    /// User display name
    pub display_name: Option<String>,
    /// First name
    pub first_name: Option<String>,
    /// Last name
    pub last_name: Option<String>,
    /// User role
    pub role: UserRole,
    /// Teams the user belongs to
    pub teams: Vec<String>,
    /// User permissions
    pub permissions: Vec<String>,
    /// User metadata
    pub metadata: HashMap<String, String>,
    /// Maximum budget for the user
    pub max_budget: Option<f64>,
    /// Current spend
    pub spend: f64,
    /// Budget duration
    pub budget_duration: Option<String>,
    /// Budget reset timestamp
    pub budget_reset_at: Option<DateTime<Utc>>,
    /// Whether user is active
    pub is_active: bool,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last login timestamp
    pub last_login_at: Option<DateTime<Utc>>,
    /// User preferences
    pub preferences: UserPreferences,
}

/// Team entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    /// Unique team identifier
    pub team_id: String,
    /// Team name
    pub team_name: String,
    /// Team description
    pub description: Option<String>,
    /// Organization ID
    pub organization_id: Option<String>,
    /// Team members
    pub members: Vec<TeamMember>,
    /// Team permissions
    pub permissions: Vec<String>,
    /// Models the team can access
    pub models: Vec<String>,
    /// Maximum budget for the team
    pub max_budget: Option<f64>,
    /// Current spend
    pub spend: f64,
    /// Budget duration
    pub budget_duration: Option<String>,
    /// Budget reset timestamp
    pub budget_reset_at: Option<DateTime<Utc>>,
    /// Team metadata
    pub metadata: HashMap<String, String>,
    /// Whether team is active
    pub is_active: bool,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Team settings
    pub settings: TeamSettings,
}

/// Organization entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    /// Unique organization identifier
    pub organization_id: String,
    /// Organization name
    pub organization_name: String,
    /// Organization description
    pub description: Option<String>,
    /// Organization domain
    pub domain: Option<String>,
    /// Teams in the organization
    pub teams: Vec<String>,
    /// Organization admins
    pub admins: Vec<String>,
    /// Maximum budget for the organization
    pub max_budget: Option<f64>,
    /// Current spend
    pub spend: f64,
    /// Budget duration
    pub budget_duration: Option<String>,
    /// Budget reset timestamp
    pub budget_reset_at: Option<DateTime<Utc>>,
    /// Organization metadata
    pub metadata: HashMap<String, String>,
    /// Whether organization is active
    pub is_active: bool,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Organization settings
    pub settings: OrganizationSettings,
}

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

/// Team member with role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMember {
    /// User ID
    pub user_id: String,
    /// Role in the team
    pub role: TeamRole,
    /// When the user joined the team
    pub joined_at: DateTime<Utc>,
    /// Whether the member is active
    pub is_active: bool,
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

/// User preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    /// Preferred language
    pub language: Option<String>,
    /// Timezone
    pub timezone: Option<String>,
    /// Email notifications enabled
    pub email_notifications: bool,
    /// Slack notifications enabled
    pub slack_notifications: bool,
    /// Dashboard preferences
    pub dashboard_config: HashMap<String, serde_json::Value>,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            language: Some("en".to_string()),
            timezone: Some("UTC".to_string()),
            email_notifications: true,
            slack_notifications: false,
            dashboard_config: HashMap::new(),
        }
    }
}

/// Team settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamSettings {
    /// Default model for the team
    pub default_model: Option<String>,
    /// Auto-approve new members
    pub auto_approve_members: bool,
    /// Require approval for high-cost requests
    pub require_approval_for_high_cost: bool,
    /// High cost threshold
    pub high_cost_threshold: Option<f64>,
    /// Team-specific rate limits
    pub rate_limits: Option<crate::core::virtual_keys::RateLimits>,
}

impl Default for TeamSettings {
    fn default() -> Self {
        Self {
            default_model: None,
            auto_approve_members: true,
            require_approval_for_high_cost: false,
            high_cost_threshold: Some(10.0),
            rate_limits: None,
        }
    }
}

/// Organization settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationSettings {
    /// SSO configuration
    pub sso_config: Option<SSOConfig>,
    /// Default team for new users
    pub default_team: Option<String>,
    /// Require email verification
    pub require_email_verification: bool,
    /// Password policy
    pub password_policy: PasswordPolicy,
    /// Session timeout in minutes
    pub session_timeout_minutes: u32,
    /// Allowed email domains
    pub allowed_email_domains: Vec<String>,
}

/// SSO configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SSOConfig {
    /// SSO provider type
    pub provider: SSOProvider,
    /// Client ID
    pub client_id: String,
    /// Client secret (encrypted)
    pub client_secret: String,
    /// Authorization endpoint
    pub auth_endpoint: String,
    /// Token endpoint
    pub token_endpoint: String,
    /// User info endpoint
    pub userinfo_endpoint: String,
    /// Scopes
    pub scopes: Vec<String>,
    /// Attribute mappings
    pub attribute_mappings: HashMap<String, String>,
}

/// SSO providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SSOProvider {
    Google,
    Microsoft,
    Okta,
    Auth0,
    Generic,
}

/// Password policy
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub require_special_chars: bool,
    /// Password expiry in days
    pub expiry_days: Option<u32>,
}

impl Default for PasswordPolicy {
    fn default() -> Self {
        Self {
            min_length: 8,
            require_uppercase: true,
            require_lowercase: true,
            require_numbers: true,
            require_special_chars: false,
            expiry_days: None,
        }
    }
}

impl Default for OrganizationSettings {
    fn default() -> Self {
        Self {
            sso_config: None,
            default_team: None,
            require_email_verification: true,
            password_policy: PasswordPolicy::default(),
            session_timeout_minutes: 480, // 8 hours
            allowed_email_domains: vec![],
        }
    }
}

/// User management system
pub struct UserManager {
    /// Database connection
    database: Arc<Database>,
}

impl UserManager {
    /// Create a new user manager
    pub fn new(database: Arc<Database>) -> Self {
        Self { database }
    }

    /// Create a new user
    pub async fn create_user(&self, email: String, display_name: Option<String>) -> Result<User> {
        info!("Creating user: {}", email);

        // Check if user already exists
        if self.database.get_user_by_email(&email).await?.is_some() {
            return Err(GatewayError::Conflict("User already exists".to_string()));
        }

        let user = User {
            user_id: Uuid::new_v4().to_string(),
            email,
            display_name,
            first_name: None,
            last_name: None,
            role: UserRole::User,
            teams: vec![],
            permissions: vec![],
            metadata: HashMap::new(),
            max_budget: Some(100.0), // Default budget
            spend: 0.0,
            budget_duration: Some("1m".to_string()),
            budget_reset_at: Some(Utc::now() + chrono::Duration::days(30)),
            is_active: true,
            created_at: Utc::now(),
            last_login_at: None,
            preferences: UserPreferences::default(),
        };

        self.database.create_user(&user).await?;
        info!("User created successfully: {}", user.user_id);
        Ok(user)
    }

    /// Get user by ID
    pub async fn get_user(&self, user_id: &str) -> Result<Option<User>> {
        self.database.get_user(user_id).await
    }

    /// Get user by email
    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        self.database.get_user_by_email(email).await
    }

    /// Update user
    pub async fn update_user(&self, user: &User) -> Result<()> {
        self.database.update_user(user).await
    }

    /// Delete user
    pub async fn delete_user(&self, user_id: &str) -> Result<()> {
        info!("Deleting user: {}", user_id);
        self.database.delete_user(user_id).await
    }

    /// Create a new team
    pub async fn create_team(
        &self,
        team_name: String,
        description: Option<String>,
        organization_id: Option<String>,
        creator_id: String,
    ) -> Result<Team> {
        info!("Creating team: {}", team_name);

        let team = Team {
            team_id: Uuid::new_v4().to_string(),
            team_name,
            description,
            organization_id,
            members: vec![TeamMember {
                user_id: creator_id,
                role: TeamRole::Owner,
                joined_at: Utc::now(),
                is_active: true,
            }],
            permissions: vec![],
            models: vec![],
            max_budget: Some(1000.0), // Default team budget
            spend: 0.0,
            budget_duration: Some("1m".to_string()),
            budget_reset_at: Some(Utc::now() + chrono::Duration::days(30)),
            metadata: HashMap::new(),
            is_active: true,
            created_at: Utc::now(),
            settings: TeamSettings::default(),
        };

        self.database.create_team(&team).await?;
        info!("Team created successfully: {}", team.team_id);
        Ok(team)
    }

    /// Get team by ID
    pub async fn get_team(&self, team_id: &str) -> Result<Option<Team>> {
        self.database.get_team(team_id).await
    }

    /// Add user to team
    pub async fn add_user_to_team(
        &self,
        team_id: &str,
        user_id: &str,
        role: TeamRole,
    ) -> Result<()> {
        info!("Adding user {} to team {} with role {:?}", user_id, team_id, role);

        let mut team = self.database.get_team(team_id).await?
            .ok_or_else(|| GatewayError::NotFound("Team not found".to_string()))?;

        // Check if user is already a member
        if team.members.iter().any(|m| m.user_id == user_id) {
            return Err(GatewayError::Conflict("User is already a team member".to_string()));
        }

        // Add member
        team.members.push(TeamMember {
            user_id: user_id.to_string(),
            role,
            joined_at: Utc::now(),
            is_active: true,
        });

        self.database.update_team(&team).await?;

        // Update user's teams list
        if let Some(mut user) = self.database.get_user(user_id).await? {
            user.teams.push(team_id.to_string());
            self.database.update_user(&user).await?;
        }

        Ok(())
    }

    /// Remove user from team
    pub async fn remove_user_from_team(&self, team_id: &str, user_id: &str) -> Result<()> {
        info!("Removing user {} from team {}", user_id, team_id);

        let mut team = self.database.get_team(team_id).await?
            .ok_or_else(|| GatewayError::NotFound("Team not found".to_string()))?;

        // Remove member
        team.members.retain(|m| m.user_id != user_id);
        self.database.update_team(&team).await?;

        // Update user's teams list
        if let Some(mut user) = self.database.get_user(user_id).await? {
            user.teams.retain(|t| t != team_id);
            self.database.update_user(&user).await?;
        }

        Ok(())
    }

    /// Create organization
    pub async fn create_organization(
        &self,
        organization_name: String,
        description: Option<String>,
        creator_id: String,
    ) -> Result<Organization> {
        info!("Creating organization: {}", organization_name);

        let organization = Organization {
            organization_id: Uuid::new_v4().to_string(),
            organization_name,
            description,
            domain: None,
            teams: vec![],
            admins: vec![creator_id],
            max_budget: Some(10000.0), // Default org budget
            spend: 0.0,
            budget_duration: Some("1m".to_string()),
            budget_reset_at: Some(Utc::now() + chrono::Duration::days(30)),
            metadata: HashMap::new(),
            is_active: true,
            created_at: Utc::now(),
            settings: OrganizationSettings::default(),
        };

        self.database.create_organization(&organization).await?;
        info!("Organization created successfully: {}", organization.organization_id);
        Ok(organization)
    }

    /// Get organization by ID
    pub async fn get_organization(&self, organization_id: &str) -> Result<Option<Organization>> {
        self.database.get_organization(organization_id).await
    }

    /// Check if user has permission
    pub async fn check_permission(&self, user_id: &str, permission: &str) -> Result<bool> {
        let user = self.database.get_user(user_id).await?
            .ok_or_else(|| GatewayError::NotFound("User not found".to_string()))?;

        // Super admin has all permissions
        if user.role == UserRole::SuperAdmin {
            return Ok(true);
        }

        // Check direct user permissions
        if user.permissions.contains(&permission.to_string()) {
            return Ok(true);
        }

        // Check team permissions
        for team_id in &user.teams {
            if let Some(team) = self.database.get_team(team_id).await? {
                if team.permissions.contains(&permission.to_string()) {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// Update user spend
    pub async fn update_user_spend(&self, user_id: &str, cost: f64) -> Result<()> {
        self.database.update_user_spend(user_id, cost).await
    }

    /// Update team spend
    pub async fn update_team_spend(&self, team_id: &str, cost: f64) -> Result<()> {
        self.database.update_team_spend(team_id, cost).await
    }

    /// List users with pagination
    pub async fn list_users(&self, offset: u32, limit: u32) -> Result<Vec<User>> {
        self.database.list_users(offset, limit).await
    }

    /// List teams with pagination
    pub async fn list_teams(&self, offset: u32, limit: u32) -> Result<Vec<Team>> {
        self.database.list_teams(offset, limit).await
    }

    /// Get user teams
    pub async fn get_user_teams(&self, user_id: &str) -> Result<Vec<Team>> {
        let user = self.database.get_user(user_id).await?
            .ok_or_else(|| GatewayError::NotFound("User not found".to_string()))?;

        let mut teams = Vec::new();
        for team_id in &user.teams {
            if let Some(team) = self.database.get_team(team_id).await? {
                teams.push(team);
            }
        }

        Ok(teams)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test UserRole enum equality and variants
    #[test]
    fn test_user_roles() {
        assert_eq!(UserRole::SuperAdmin, UserRole::SuperAdmin);
        assert_ne!(UserRole::User, UserRole::Admin);

        // Test all variants exist
        let roles = vec![
            UserRole::SuperAdmin,
            UserRole::Admin,
            UserRole::User,
            UserRole::Guest,
        ];
        assert_eq!(roles.len(), 4);
    }

    /// Test TeamRole enum equality and variants
    #[test]
    fn test_team_roles() {
        assert_eq!(TeamRole::Owner, TeamRole::Owner);
        assert_ne!(TeamRole::Member, TeamRole::Admin);

        // Test all variants exist
        let roles = vec![
            TeamRole::Owner,
            TeamRole::Admin,
            TeamRole::Member,
            TeamRole::Viewer,
        ];
        assert_eq!(roles.len(), 4);
    }

    /// Test UserProfile structure
    #[test]
    fn test_user_profile_creation() {
        let profile = UserProfile {
            user_id: "user123".to_string(),
            email: "test@example.com".to_string(),
            display_name: Some("Test User".to_string()),
            role: UserRole::User,
            is_active: true,
            created_at: Utc::now(),
            teams: vec![],
            metadata: HashMap::new(),
        };

        assert_eq!(profile.user_id, "user123");
        assert_eq!(profile.email, "test@example.com");
        assert!(profile.is_active);
        assert_eq!(profile.role, UserRole::User);
    }

    /// Test Team structure
    #[test]
    fn test_team_structure() {
        let owner = TeamMember {
            user_id: "owner123".to_string(),
            role: TeamRole::Owner,
            added_at: Utc::now(),
            added_by: None,
        };

        let team = Team {
            team_id: "team123".to_string(),
            team_name: "Test Team".to_string(),
            description: Some("A test team".to_string()),
            organization_id: None,
            members: vec![owner.clone()],
            created_at: Utc::now(),
            is_active: true,
            settings: TeamSettings::default(),
            metadata: HashMap::new(),
        };

        assert_eq!(team.team_name, "Test Team");
        assert_eq!(team.members.len(), 1);
        assert_eq!(team.members[0].role, TeamRole::Owner);
    }

    /// Test TeamMember role assignment
    #[test]
    fn test_team_member_roles() {
        let owner = TeamMember {
            user_id: "u1".to_string(),
            role: TeamRole::Owner,
            added_at: Utc::now(),
            added_by: None,
        };

        let admin = TeamMember {
            user_id: "u2".to_string(),
            role: TeamRole::Admin,
            added_at: Utc::now(),
            added_by: Some("u1".to_string()),
        };

        let member = TeamMember {
            user_id: "u3".to_string(),
            role: TeamRole::Member,
            added_at: Utc::now(),
            added_by: Some("u1".to_string()),
        };

        assert_eq!(owner.role, TeamRole::Owner);
        assert_eq!(admin.role, TeamRole::Admin);
        assert_eq!(member.role, TeamRole::Member);
    }

    /// Test TeamSettings default values
    #[test]
    fn test_team_settings_defaults() {
        let settings = TeamSettings::default();

        // Verify default settings are reasonable
        assert!(settings.max_members.is_none() || settings.max_members.unwrap() > 0);
    }

    /// Test AddTeamMemberRequest structure
    #[test]
    fn test_add_team_member_request() {
        let request = AddTeamMemberRequest {
            user_id: "user456".to_string(),
            role: TeamRole::Member,
        };

        assert_eq!(request.user_id, "user456");
        assert_eq!(request.role, TeamRole::Member);
    }

    /// Test UserRole hierarchy (conceptually)
    #[test]
    fn test_user_role_hierarchy() {
        // SuperAdmin > Admin > User > Guest
        let super_admin = UserRole::SuperAdmin;
        let admin = UserRole::Admin;
        let user = UserRole::User;
        let guest = UserRole::Guest;

        // Verify they are distinct
        assert_ne!(super_admin, admin);
        assert_ne!(admin, user);
        assert_ne!(user, guest);
    }
}
