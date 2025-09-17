//! Role-Based Access Control (RBAC) system
//!
//! This module provides comprehensive RBAC functionality for authorization.

use crate::config::RbacConfig;
use crate::core::models::{TeamRole, User, UserRole};
use crate::utils::error::{GatewayError, Result};
use std::collections::{HashMap, HashSet};
use tracing::{debug, info};

/// RBAC system for managing roles and permissions
#[derive(Debug, Clone)]
pub struct RbacSystem {
    /// RBAC configuration
    config: RbacConfig,
    /// Role definitions
    roles: HashMap<String, Role>,
    /// Permission definitions
    permissions: HashMap<String, Permission>,
}

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

impl RbacSystem {
    /// Create a new RBAC system
    pub async fn new(config: &RbacConfig) -> Result<Self> {
        info!("Initializing RBAC system");

        let mut rbac = Self {
            config: config.clone(),
            roles: HashMap::new(),
            permissions: HashMap::new(),
        };

        // Initialize default permissions and roles
        rbac.initialize_default_permissions().await?;
        rbac.initialize_default_roles().await?;

        info!("RBAC system initialized successfully");
        Ok(rbac)
    }

    /// Initialize default permissions
    async fn initialize_default_permissions(&mut self) -> Result<()> {
        debug!("Initializing default permissions");

        let default_permissions = vec![
            // User management
            Permission {
                name: "users.read".to_string(),
                description: "Read user information".to_string(),
                resource: "users".to_string(),
                action: "read".to_string(),
                is_system: true,
            },
            Permission {
                name: "users.write".to_string(),
                description: "Create and update users".to_string(),
                resource: "users".to_string(),
                action: "write".to_string(),
                is_system: true,
            },
            Permission {
                name: "users.delete".to_string(),
                description: "Delete users".to_string(),
                resource: "users".to_string(),
                action: "delete".to_string(),
                is_system: true,
            },
            // Team management
            Permission {
                name: "teams.read".to_string(),
                description: "Read team information".to_string(),
                resource: "teams".to_string(),
                action: "read".to_string(),
                is_system: true,
            },
            Permission {
                name: "teams.write".to_string(),
                description: "Create and update teams".to_string(),
                resource: "teams".to_string(),
                action: "write".to_string(),
                is_system: true,
            },
            Permission {
                name: "teams.delete".to_string(),
                description: "Delete teams".to_string(),
                resource: "teams".to_string(),
                action: "delete".to_string(),
                is_system: true,
            },
            // API access
            Permission {
                name: "api.chat".to_string(),
                description: "Access chat completion API".to_string(),
                resource: "api".to_string(),
                action: "chat".to_string(),
                is_system: true,
            },
            Permission {
                name: "api.embeddings".to_string(),
                description: "Access embeddings API".to_string(),
                resource: "api".to_string(),
                action: "embeddings".to_string(),
                is_system: true,
            },
            Permission {
                name: "api.images".to_string(),
                description: "Access image generation API".to_string(),
                resource: "api".to_string(),
                action: "images".to_string(),
                is_system: true,
            },
            // API key management
            Permission {
                name: "api_keys.read".to_string(),
                description: "Read API key information".to_string(),
                resource: "api_keys".to_string(),
                action: "read".to_string(),
                is_system: true,
            },
            Permission {
                name: "api_keys.write".to_string(),
                description: "Create and update API keys".to_string(),
                resource: "api_keys".to_string(),
                action: "write".to_string(),
                is_system: true,
            },
            Permission {
                name: "api_keys.delete".to_string(),
                description: "Delete API keys".to_string(),
                resource: "api_keys".to_string(),
                action: "delete".to_string(),
                is_system: true,
            },
            // Analytics and monitoring
            Permission {
                name: "analytics.read".to_string(),
                description: "Read analytics and usage data".to_string(),
                resource: "analytics".to_string(),
                action: "read".to_string(),
                is_system: true,
            },
            Permission {
                name: "system.admin".to_string(),
                description: "Full system administration access".to_string(),
                resource: "system".to_string(),
                action: "admin".to_string(),
                is_system: true,
            },
        ];

        for permission in default_permissions {
            self.permissions.insert(permission.name.clone(), permission);
        }

        debug!("Initialized {} default permissions", self.permissions.len());
        Ok(())
    }

    /// Initialize default roles
    async fn initialize_default_roles(&mut self) -> Result<()> {
        debug!("Initializing default roles");

        let default_roles = vec![
            // Super Admin - full access
            Role {
                name: "super_admin".to_string(),
                description: "Super administrator with full system access".to_string(),
                permissions: self.permissions.keys().cloned().collect(),
                parent_roles: HashSet::new(),
                is_system: true,
            },
            // Admin - most access except super admin functions
            Role {
                name: "admin".to_string(),
                description: "Administrator with broad system access".to_string(),
                permissions: [
                    "users.read",
                    "users.write",
                    "teams.read",
                    "teams.write",
                    "api.chat",
                    "api.embeddings",
                    "api.images",
                    "api_keys.read",
                    "api_keys.write",
                    "api_keys.delete",
                    "analytics.read",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                parent_roles: HashSet::new(),
                is_system: true,
            },
            // Manager - team management and API access
            Role {
                name: "manager".to_string(),
                description: "Team manager with API access and team management".to_string(),
                permissions: [
                    "teams.read",
                    "teams.write",
                    "api.chat",
                    "api.embeddings",
                    "api.images",
                    "api_keys.read",
                    "api_keys.write",
                    "analytics.read",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                parent_roles: HashSet::new(),
                is_system: true,
            },
            // User - basic API access
            Role {
                name: "user".to_string(),
                description: "Regular user with API access".to_string(),
                permissions: ["api.chat", "api.embeddings", "api_keys.read"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                parent_roles: HashSet::new(),
                is_system: true,
            },
            // Viewer - read-only access
            Role {
                name: "viewer".to_string(),
                description: "Read-only access to resources".to_string(),
                permissions: [
                    "users.read",
                    "teams.read",
                    "api_keys.read",
                    "analytics.read",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                parent_roles: HashSet::new(),
                is_system: true,
            },
            // API User - API access only
            Role {
                name: "api_user".to_string(),
                description: "API-only access for programmatic use".to_string(),
                permissions: ["api.chat", "api.embeddings", "api.images"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                parent_roles: HashSet::new(),
                is_system: true,
            },
        ];

        for role in default_roles {
            self.roles.insert(role.name.clone(), role);
        }

        debug!("Initialized {} default roles", self.roles.len());
        Ok(())
    }

    /// Get all permissions for a user
    pub async fn get_user_permissions(&self, user: &User) -> Result<Vec<String>> {
        let mut permissions = HashSet::new();

        // Get permissions from user role
        let role_name = self.user_role_to_string(&user.role);
        if let Some(role) = self.roles.get(&role_name) {
            permissions.extend(self.get_role_permissions(role));
        }

        Ok(permissions.into_iter().collect())
    }

    /// Get all permissions for a role (including inherited)
    fn get_role_permissions(&self, role: &Role) -> HashSet<String> {
        let mut permissions = role.permissions.clone();

        // Add permissions from parent roles
        for parent_role_name in &role.parent_roles {
            if let Some(parent_role) = self.roles.get(parent_role_name) {
                permissions.extend(self.get_role_permissions(parent_role));
            }
        }

        permissions
    }

    /// Check if user has specific permissions
    pub fn check_permissions(
        &self,
        user_permissions: &[String],
        required_permissions: &[String],
    ) -> bool {
        let user_perms: HashSet<&String> = user_permissions.iter().collect();

        // Check for wildcard permission
        if user_perms
            .iter()
            .any(|p| p.as_str() == "*" || p.as_str() == "system.admin")
        {
            return true;
        }

        // Check if user has all required permissions
        required_permissions
            .iter()
            .all(|perm| user_perms.contains(&perm))
    }

    /// Check if user has any of the required permissions
    pub fn check_any_permission(
        &self,
        user_permissions: &[String],
        required_permissions: &[String],
    ) -> bool {
        let user_perms: HashSet<&String> = user_permissions.iter().collect();

        // Check for wildcard permission
        if user_perms
            .iter()
            .any(|p| p.as_str() == "*" || p.as_str() == "system.admin")
        {
            return true;
        }

        // Check if user has any of the required permissions
        required_permissions
            .iter()
            .any(|perm| user_perms.contains(&perm))
    }

    /// Detailed permission check
    pub async fn check_permission_detailed(
        &self,
        user: &User,
        required_permission: &str,
    ) -> Result<PermissionCheck> {
        let user_permissions = self.get_user_permissions(user).await?;
        let user_perms: HashSet<&String> = user_permissions.iter().collect();

        // Check for wildcard or admin permission
        if user_perms
            .iter()
            .any(|p| p.as_str() == "*" || p.as_str() == "system.admin")
        {
            return Ok(PermissionCheck {
                granted: true,
                granted_by_roles: vec![self.user_role_to_string(&user.role)],
                denial_reason: None,
            });
        }

        // Check specific permission
        if user_perms.iter().any(|p| p.as_str() == required_permission) {
            Ok(PermissionCheck {
                granted: true,
                granted_by_roles: vec![self.user_role_to_string(&user.role)],
                denial_reason: None,
            })
        } else {
            Ok(PermissionCheck {
                granted: false,
                granted_by_roles: vec![],
                denial_reason: Some(format!("Missing permission: {}", required_permission)),
            })
        }
    }

    /// Check if user is admin
    pub fn is_admin(&self, user: &User) -> bool {
        let role_name = self.user_role_to_string(&user.role);
        self.config.admin_roles.contains(&role_name)
    }

    /// Get role by name
    pub fn get_role(&self, role_name: &str) -> Option<&Role> {
        self.roles.get(role_name)
    }

    /// Get permission by name
    pub fn get_permission(&self, permission_name: &str) -> Option<&Permission> {
        self.permissions.get(permission_name)
    }

    /// List all roles
    pub fn list_roles(&self) -> Vec<&Role> {
        self.roles.values().collect()
    }

    /// List all permissions
    pub fn list_permissions(&self) -> Vec<&Permission> {
        self.permissions.values().collect()
    }

    /// Add custom role
    pub fn add_role(&mut self, role: Role) -> Result<()> {
        if role.is_system {
            return Err(GatewayError::auth("Cannot modify system roles"));
        }

        self.roles.insert(role.name.clone(), role);
        Ok(())
    }

    /// Add custom permission
    pub fn add_permission(&mut self, permission: Permission) -> Result<()> {
        if permission.is_system {
            return Err(GatewayError::auth("Cannot modify system permissions"));
        }

        self.permissions.insert(permission.name.clone(), permission);
        Ok(())
    }

    /// Convert UserRole to string
    fn user_role_to_string(&self, role: &UserRole) -> String {
        match role {
            UserRole::SuperAdmin => "super_admin".to_string(),
            UserRole::Admin => "admin".to_string(),
            UserRole::Manager => "manager".to_string(),
            UserRole::User => "user".to_string(),
            UserRole::Viewer => "viewer".to_string(),
            UserRole::ApiUser => "api_user".to_string(),
        }
    }

    /// Convert TeamRole to permissions
    pub fn team_role_to_permissions(&self, role: &TeamRole) -> Vec<String> {
        match role {
            TeamRole::Owner => vec![
                "teams.read".to_string(),
                "teams.write".to_string(),
                "teams.delete".to_string(),
                "users.read".to_string(),
                "users.write".to_string(),
                "api_keys.read".to_string(),
                "api_keys.write".to_string(),
                "api_keys.delete".to_string(),
                "analytics.read".to_string(),
            ],
            TeamRole::Admin => vec![
                "teams.read".to_string(),
                "teams.write".to_string(),
                "users.read".to_string(),
                "users.write".to_string(),
                "api_keys.read".to_string(),
                "api_keys.write".to_string(),
                "analytics.read".to_string(),
            ],
            TeamRole::Manager => vec![
                "teams.read".to_string(),
                "users.read".to_string(),
                "api_keys.read".to_string(),
                "api_keys.write".to_string(),
                "analytics.read".to_string(),
            ],
            TeamRole::Member => vec![
                "teams.read".to_string(),
                "api_keys.read".to_string(),
                "analytics.read".to_string(),
            ],
            TeamRole::Viewer => vec!["teams.read".to_string(), "analytics.read".to_string()],
        }
    }

    /// Check resource-level permissions
    pub fn check_resource_permission(
        &self,
        user_permissions: &[String],
        resource: &str,
        action: &str,
    ) -> bool {
        let required_permission = format!("{}.{}", resource, action);
        self.check_permissions(user_permissions, &[required_permission])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RbacConfig;

    async fn create_test_rbac() -> RbacSystem {
        let config = RbacConfig {
            enabled: true,
            default_role: "user".to_string(),
            admin_roles: vec!["admin".to_string(), "super_admin".to_string()],
        };

        RbacSystem::new(&config).await.unwrap()
    }

    #[tokio::test]
    async fn test_rbac_initialization() {
        let rbac = create_test_rbac().await;

        assert!(!rbac.roles.is_empty());
        assert!(!rbac.permissions.is_empty());
        assert!(rbac.get_role("user").is_some());
        assert!(rbac.get_role("admin").is_some());
        assert!(rbac.get_permission("api.chat").is_some());
    }

    #[tokio::test]
    async fn test_permission_checking() {
        let rbac = create_test_rbac().await;

        let user_permissions = vec!["api.chat".to_string(), "api.embeddings".to_string()];
        let required_permissions = vec!["api.chat".to_string()];

        assert!(rbac.check_permissions(&user_permissions, &required_permissions));

        let required_permissions = vec!["system.admin".to_string()];
        assert!(!rbac.check_permissions(&user_permissions, &required_permissions));
    }

    #[tokio::test]
    async fn test_admin_permissions() {
        let rbac = create_test_rbac().await;

        let admin_permissions = vec!["system.admin".to_string()];
        let any_permission = vec!["api.chat".to_string()];

        assert!(rbac.check_permissions(&admin_permissions, &any_permission));
    }

    #[test]
    fn test_role_creation() {
        let role = Role {
            name: "test_role".to_string(),
            description: "Test role".to_string(),
            permissions: ["api.chat".to_string()].iter().cloned().collect(),
            parent_roles: HashSet::new(),
            is_system: false,
        };

        assert_eq!(role.name, "test_role");
        assert!(role.permissions.contains("api.chat"));
        assert!(!role.is_system);
    }

    #[test]
    fn test_permission_creation() {
        let permission = Permission {
            name: "test.permission".to_string(),
            description: "Test permission".to_string(),
            resource: "test".to_string(),
            action: "permission".to_string(),
            is_system: false,
        };

        assert_eq!(permission.name, "test.permission");
        assert_eq!(permission.resource, "test");
        assert_eq!(permission.action, "permission");
    }
}
