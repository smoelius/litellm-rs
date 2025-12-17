//! Tests for RBAC functionality

#[cfg(test)]
mod tests {
    use crate::auth::rbac::types::{Permission, Role};
    use crate::auth::rbac::RbacSystem;
    use crate::config::RbacConfig;
    use std::collections::HashSet;

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

        assert!(!rbac.list_roles().is_empty());
        assert!(!rbac.list_permissions().is_empty());
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
