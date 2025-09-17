//! Authentication and authorization system
//!
//! This module provides comprehensive authentication and authorization functionality.

#![allow(dead_code)]

pub mod api_key;
pub mod jwt;
pub mod rbac;

// Re-export commonly used types
pub use crate::core::models::{ApiKey, User, UserRole, UserSession};

use crate::config::AuthConfig;
use crate::core::models::RequestContext;
use crate::storage::StorageLayer;
use crate::utils::error::{GatewayError, Result};
use std::sync::Arc;
use tracing::{debug, info};
use uuid::Uuid;

/// Main authentication system
#[derive(Clone)]
pub struct AuthSystem {
    /// Authentication configuration
    config: Arc<AuthConfig>,
    /// Storage layer for user data
    storage: Arc<StorageLayer>,
    /// JWT handler
    jwt: Arc<jwt::JwtHandler>,
    /// API key handler
    api_key: Arc<api_key::ApiKeyHandler>,
    /// RBAC system
    rbac: Arc<rbac::RbacSystem>,
    // /// Session manager
    // session: Arc<session::SessionManager>,
}

/// Authentication result
#[derive(Debug, Clone)]
pub struct AuthResult {
    /// Whether authentication was successful
    pub success: bool,
    /// Authenticated user (if any)
    pub user: Option<User>,
    /// API key used (if any)
    pub api_key: Option<ApiKey>,
    /// Session information (if any)
    pub session: Option<UserSession>,
    /// Error message (if authentication failed)
    pub error: Option<String>,
    /// Request context
    pub context: RequestContext,
}

/// Authorization result
#[derive(Debug, Clone)]
pub struct AuthzResult {
    /// Whether authorization was successful
    pub allowed: bool,
    /// Required permissions that were checked
    pub required_permissions: Vec<String>,
    /// User's actual permissions
    pub user_permissions: Vec<String>,
    /// Reason for denial (if not allowed)
    pub reason: Option<String>,
}

/// Authentication method
#[derive(Debug, Clone)]
pub enum AuthMethod {
    /// JWT token authentication
    Jwt(String),
    /// API key authentication
    ApiKey(String),
    /// Session-based authentication
    Session(String),
    /// No authentication
    None,
}

impl AuthSystem {
    /// Create a new authentication system
    pub async fn new(config: &AuthConfig, storage: Arc<StorageLayer>) -> Result<Self> {
        info!("Initializing authentication system");

        let config = Arc::new(config.clone());

        // Initialize JWT handler
        let jwt = Arc::new(jwt::JwtHandler::new(&config).await?);

        // Initialize API key handler
        let api_key = Arc::new(api_key::ApiKeyHandler::new(storage.clone()).await?);

        // Initialize RBAC system
        let rbac = Arc::new(rbac::RbacSystem::new(&config.rbac).await?);

        // Initialize session manager
        // let session = Arc::new(session::SessionManager::new(storage.clone()).await?);

        info!("Authentication system initialized successfully");

        Ok(Self {
            config,
            storage,
            jwt,
            api_key,
            rbac,
            // session,
        })
    }

    /// Authenticate a request
    pub async fn authenticate(
        &self,
        auth_method: AuthMethod,
        context: RequestContext,
    ) -> Result<AuthResult> {
        debug!("Authenticating request: {:?}", auth_method);

        match auth_method {
            AuthMethod::Jwt(token) => self.authenticate_jwt(&token, context).await,
            AuthMethod::ApiKey(key) => self.authenticate_api_key(&key, context).await,
            AuthMethod::Session(session_id) => {
                self.authenticate_session(&session_id, context).await
            }
            AuthMethod::None => Ok(AuthResult {
                success: false,
                user: None,
                api_key: None,
                session: None,
                error: Some("No authentication provided".to_string()),
                context,
            }),
        }
    }

    /// Authenticate using JWT token
    async fn authenticate_jwt(
        &self,
        token: &str,
        mut context: RequestContext,
    ) -> Result<AuthResult> {
        match self.jwt.verify_token(token).await {
            Ok(claims) => {
                // Get user from database
                if let Ok(Some(user)) = self.storage.db().find_user_by_id(claims.sub).await {
                    if user.is_active() {
                        context.user_id = Some(user.id());
                        context.team_id = user.team_ids.first().copied();

                        Ok(AuthResult {
                            success: true,
                            user: Some(user),
                            api_key: None,
                            session: None,
                            error: None,
                            context,
                        })
                    } else {
                        Ok(AuthResult {
                            success: false,
                            user: None,
                            api_key: None,
                            session: None,
                            error: Some("User account is not active".to_string()),
                            context,
                        })
                    }
                } else {
                    Ok(AuthResult {
                        success: false,
                        user: None,
                        api_key: None,
                        session: None,
                        error: Some("User not found".to_string()),
                        context,
                    })
                }
            }
            Err(e) => Ok(AuthResult {
                success: false,
                user: None,
                api_key: None,
                session: None,
                error: Some(format!("Invalid JWT token: {}", e)),
                context,
            }),
        }
    }

    /// Authenticate using API key
    async fn authenticate_api_key(
        &self,
        key: &str,
        mut context: RequestContext,
    ) -> Result<AuthResult> {
        match self.api_key.verify_key(key).await {
            Ok(Some((api_key, user))) => {
                context.api_key_id = Some(api_key.metadata.id);
                context.user_id = api_key.user_id;
                context.team_id = api_key.team_id;

                Ok(AuthResult {
                    success: true,
                    user,
                    api_key: Some(api_key),
                    session: None,
                    error: None,
                    context,
                })
            }
            Ok(None) => Ok(AuthResult {
                success: false,
                user: None,
                api_key: None,
                session: None,
                error: Some("Invalid API key".to_string()),
                context,
            }),
            Err(e) => Ok(AuthResult {
                success: false,
                user: None,
                api_key: None,
                session: None,
                error: Some(format!("API key verification failed: {}", e)),
                context,
            }),
        }
    }

    /// Authenticate using session
    async fn authenticate_session(
        &self,
        session_id: &str,
        mut context: RequestContext,
    ) -> Result<AuthResult> {
        // TODO: Implement session verification
        match self.jwt.verify_token(session_id).await {
            Ok(claims) => {
                // Try to find user by ID from claims
                match self.storage.db().find_user_by_id(claims.sub).await {
                    Ok(Some(user)) => {
                        context.user_id = Some(user.id());
                        context.team_id = user.team_ids.first().copied();

                        Ok(AuthResult {
                            success: true,
                            user: Some(user),
                            api_key: None,
                            session: None, // TODO: Create proper session object
                            error: None,
                            context,
                        })
                    }
                    Ok(None) => Ok(AuthResult {
                        success: false,
                        user: None,
                        api_key: None,
                        session: None,
                        error: Some("User not found".to_string()),
                        context,
                    }),
                    Err(e) => Ok(AuthResult {
                        success: false,
                        user: None,
                        api_key: None,
                        session: None,
                        error: Some(format!("User lookup failed: {}", e)),
                        context,
                    }),
                }
            }
            Err(e) => Ok(AuthResult {
                success: false,
                user: None,
                api_key: None,
                session: None,
                error: Some(format!("Session verification failed: {}", e)),
                context,
            }),
        }
    }

    /// Authorize a user for specific permissions
    pub async fn authorize(&self, user: &User, permissions: &[String]) -> Result<AuthzResult> {
        debug!(
            "Authorizing user {} for permissions: {:?}",
            user.username, permissions
        );

        let user_permissions = self.rbac.get_user_permissions(user).await?;
        let allowed = self.rbac.check_permissions(&user_permissions, permissions);

        Ok(AuthzResult {
            allowed,
            required_permissions: permissions.to_vec(),
            user_permissions,
            reason: if !allowed {
                Some("Insufficient permissions".to_string())
            } else {
                None
            },
        })
    }

    /// Create a new user
    pub async fn create_user(
        &self,
        username: String,
        email: String,
        password: String,
    ) -> Result<User> {
        info!("Creating new user: {}", username);

        // Hash password
        let password_hash = crate::utils::auth::crypto::hash_password(&password)?;

        // Create user
        let user = User::new(username, email, password_hash);

        // Store in database
        self.storage.db().create_user(&user).await
    }

    /// Login user and create session
    pub async fn login(&self, username: &str, password: &str) -> Result<(User, String)> {
        info!("User login attempt: {}", username);

        // Find user
        let user = self
            .storage
            .db()
            .find_user_by_username(username)
            .await?
            .ok_or_else(|| GatewayError::auth("Invalid username or password"))?;

        // Verify password
        if !crate::utils::auth::crypto::verify_password(password, &user.password_hash)? {
            return Err(GatewayError::auth("Invalid username or password"));
        }

        // Check if user is active
        if !user.is_active() {
            return Err(GatewayError::auth("Account is not active"));
        }

        // Create session
        let session_id = uuid::Uuid::new_v4();
        let permissions = self.get_user_permissions(&user).await?;
        let session_token = self
            .jwt
            .create_access_token(
                user.id(),
                format!("{:?}", user.role),
                permissions,
                user.team_ids.first().copied(),
                Some(session_id),
            )
            .await?;

        // Update last login
        self.storage.db().update_user_last_login(user.id()).await?;

        info!("User logged in successfully: {}", username);
        Ok((user, session_token))
    }

    /// Get user permissions based on role
    async fn get_user_permissions(&self, user: &User) -> Result<Vec<String>> {
        let permissions = match user.role {
            UserRole::Admin => vec![
                "read:all".to_string(),
                "write:all".to_string(),
                "delete:all".to_string(),
                "manage:users".to_string(),
                "manage:api_keys".to_string(),
                "manage:teams".to_string(),
            ],
            UserRole::SuperAdmin => vec![
                "read:all".to_string(),
                "write:all".to_string(),
                "delete:all".to_string(),
                "manage:users".to_string(),
                "manage:api_keys".to_string(),
                "manage:teams".to_string(),
                "manage:system".to_string(),
            ],
            UserRole::Manager => vec![
                "read:team".to_string(),
                "write:team".to_string(),
                "manage:team_users".to_string(),
                "manage:team_api_keys".to_string(),
            ],
            UserRole::User => vec![
                "read:own".to_string(),
                "write:own".to_string(),
                "use:api".to_string(),
            ],
            UserRole::ApiUser => vec!["use:api".to_string()],
            UserRole::Viewer => vec!["read:own".to_string()],
        };
        Ok(permissions)
    }

    /// Logout user
    pub async fn logout(&self, session_token: &str) -> Result<()> {
        info!("User logout");

        // Extract session ID from token
        if let Ok(claims) = self.jwt.verify_token(session_token).await {
            if let Some(session_id) = claims.session_id {
                // Store invalidated session (in practice, you'd use Redis or similar)
                // For now, just log the session invalidation
                info!("Invalidated session: {}", session_id);
            }
        }

        Ok(())
    }

    /// Create API key for user
    pub async fn create_api_key(
        &self,
        user_id: Uuid,
        name: String,
        permissions: Vec<String>,
    ) -> Result<(ApiKey, String)> {
        info!("Creating API key for user: {}", user_id);
        self.api_key
            .create_key(Some(user_id), None, name, permissions)
            .await
    }

    /// Revoke API key
    pub async fn revoke_api_key(&self, key_id: Uuid) -> Result<()> {
        info!("Revoking API key: {}", key_id);
        self.api_key.revoke_key(key_id).await
    }

    /// Change user password
    pub async fn change_password(
        &self,
        user_id: Uuid,
        old_password: &str,
        new_password: &str,
    ) -> Result<()> {
        info!("Changing password for user: {}", user_id);

        // Get user
        let user = self
            .storage
            .db()
            .find_user_by_id(user_id)
            .await?
            .ok_or_else(|| GatewayError::not_found("User not found"))?;

        // Verify old password
        if !crate::utils::auth::crypto::verify_password(old_password, &user.password_hash)? {
            return Err(GatewayError::auth("Invalid current password"));
        }

        // Hash new password
        let new_password_hash = crate::utils::auth::crypto::hash_password(new_password)?;

        // Update password
        self.storage
            .db()
            .update_user_password(user_id, &new_password_hash)
            .await?;

        info!("Password changed successfully for user: {}", user_id);
        Ok(())
    }

    /// Reset password (generate reset token)
    pub async fn request_password_reset(&self, email: &str) -> Result<String> {
        info!("Password reset requested for email: {}", email);

        // Find user by email
        let user = self
            .storage
            .db()
            .find_user_by_email(email)
            .await?
            .ok_or_else(|| GatewayError::not_found("User not found"))?;

        // Generate reset token
        let reset_token = crate::utils::auth::crypto::generate_token(32);
        let expires_at = chrono::Utc::now() + chrono::Duration::hours(1);

        // Store reset token
        self.storage
            .db()
            .store_password_reset_token(user.id(), &reset_token, expires_at)
            .await?;

        info!("Password reset token generated for user: {}", user.id());
        Ok(reset_token)
    }

    /// Reset password using token
    pub async fn reset_password(&self, token: &str, new_password: &str) -> Result<()> {
        info!("Resetting password with token");

        // Verify reset token
        let user_id = self
            .storage
            .db()
            .verify_password_reset_token(token)
            .await?
            .ok_or_else(|| GatewayError::auth("Invalid or expired reset token"))?;

        // Hash new password
        let password_hash = crate::utils::auth::crypto::hash_password(new_password)?;

        // Update password
        self.storage
            .db()
            .update_user_password(user_id, &password_hash)
            .await?;

        // Invalidate reset token
        self.storage
            .db()
            .invalidate_password_reset_token(token)
            .await?;

        info!("Password reset successfully for user: {}", user_id);
        Ok(())
    }

    /// Get authentication configuration
    pub fn config(&self) -> &AuthConfig {
        &self.config
    }

    /// Get JWT handler
    pub fn jwt(&self) -> &jwt::JwtHandler {
        &self.jwt
    }

    /// Get API key handler
    pub fn api_key(&self) -> &api_key::ApiKeyHandler {
        &self.api_key
    }

    /// Get RBAC system
    pub fn rbac(&self) -> &rbac::RbacSystem {
        &self.rbac
    }

    // /// Get session manager
    // pub fn session(&self) -> &session::SessionManager {
    //     &self.session
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

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
