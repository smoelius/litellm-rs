//! Tests for authentication module

#[cfg(test)]
mod tests {
    use crate::auth::types::{AuthMethod, AuthResult, AuthzResult};
    use crate::core::models::RequestContext;

    #[test]
    fn test_auth_result_creation() {
        let context = RequestContext::new();
        let result = AuthResult {
            success: true,
            user: None,
            api_key: None,
            session: None,
            error: None,
            context,
        };

        assert!(result.success);
        assert!(result.error.is_none());
    }

    #[test]
    fn test_authz_result_creation() {
        let result = AuthzResult {
            allowed: true,
            required_permissions: vec!["read".to_string()],
            user_permissions: vec!["read".to_string(), "write".to_string()],
            reason: None,
        };

        assert!(result.allowed);
        assert_eq!(result.required_permissions.len(), 1);
        assert_eq!(result.user_permissions.len(), 2);
    }

    #[test]
    fn test_auth_method_variants() {
        let jwt_method = AuthMethod::Jwt("token".to_string());
        let api_key_method = AuthMethod::ApiKey("key".to_string());
        let session_method = AuthMethod::Session("session".to_string());
        let none_method = AuthMethod::None;

        assert!(matches!(jwt_method, AuthMethod::Jwt(_)));
        assert!(matches!(api_key_method, AuthMethod::ApiKey(_)));
        assert!(matches!(session_method, AuthMethod::Session(_)));
        assert!(matches!(none_method, AuthMethod::None));
    }
}
