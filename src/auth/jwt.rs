//! JWT token handling
//!
//! This module provides JWT token creation, verification, and management.

use crate::config::AuthConfig;
use crate::utils::error::{GatewayError, Result};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, warn};
use uuid::Uuid;

/// JWT handler for token operations
#[derive(Clone)]
pub struct JwtHandler {
    /// Encoding key for signing tokens
    encoding_key: EncodingKey,
    /// Decoding key for verifying tokens
    decoding_key: DecodingKey,
    /// JWT algorithm
    algorithm: Algorithm,
    /// Token expiration time in seconds
    expiration: u64,
    /// Token issuer
    issuer: String,
}

impl std::fmt::Debug for JwtHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JwtHandler")
            .field("algorithm", &self.algorithm)
            .field("expiration", &self.expiration)
            .field("issuer", &self.issuer)
            .field("encoding_key", &"[REDACTED]")
            .field("decoding_key", &"[REDACTED]")
            .finish()
    }
}

/// JWT claims structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: Uuid,
    /// Issued at timestamp
    pub iat: u64,
    /// Expiration timestamp
    pub exp: u64,
    /// Issuer
    pub iss: String,
    /// Audience
    pub aud: String,
    /// JWT ID
    pub jti: String,
    /// User role
    pub role: String,
    /// User permissions
    pub permissions: Vec<String>,
    /// Team ID (optional)
    pub team_id: Option<Uuid>,
    /// Session ID (optional)
    pub session_id: Option<String>,
    /// Token type
    pub token_type: TokenType,
}

/// Token type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenType {
    /// Access token for API access
    Access,
    /// Refresh token for obtaining new access tokens
    Refresh,
    /// Password reset token
    PasswordReset,
    /// Email verification token
    EmailVerification,
    /// Invitation token
    Invitation,
}

/// Token pair (access + refresh)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    /// Access token
    pub access_token: String,
    /// Refresh token
    pub refresh_token: String,
    /// Token type (always "Bearer")
    pub token_type: String,
    /// Expires in seconds
    pub expires_in: u64,
}

impl JwtHandler {
    /// Create a new JWT handler
    pub async fn new(config: &AuthConfig) -> Result<Self> {
        let secret = config.jwt_secret.as_bytes();

        Ok(Self {
            encoding_key: EncodingKey::from_secret(secret),
            decoding_key: DecodingKey::from_secret(secret),
            algorithm: Algorithm::HS256,
            expiration: config.jwt_expiration,
            issuer: "litellm-rs".to_string(),
        })
    }

    /// Create an access token for a user
    pub async fn create_access_token(
        &self,
        user_id: Uuid,
        role: String,
        permissions: Vec<String>,
        team_id: Option<Uuid>,
        session_id: Option<Uuid>,
    ) -> Result<String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| GatewayError::internal(format!("System time error: {}", e)))?
            .as_secs();

        let claims = Claims {
            sub: user_id,
            iat: now,
            exp: now + self.expiration,
            iss: self.issuer.clone(),
            aud: "api".to_string(),
            jti: Uuid::new_v4().to_string(),
            role,
            permissions,
            team_id,
            session_id: session_id.map(|id| id.to_string()),
            token_type: TokenType::Access,
        };

        let header = Header::new(self.algorithm);
        let token = encode(&header, &claims, &self.encoding_key).map_err(GatewayError::Jwt)?;

        debug!("Created access token for user: {}", user_id);
        Ok(token)
    }

    /// Create a refresh token for a user
    pub async fn create_refresh_token(
        &self,
        user_id: Uuid,
        session_id: Option<String>,
    ) -> Result<String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| GatewayError::internal(format!("System time error: {}", e)))?
            .as_secs();

        let claims = Claims {
            sub: user_id,
            iat: now,
            exp: now + (self.expiration * 24), // Refresh tokens last 24x longer
            iss: self.issuer.clone(),
            aud: "refresh".to_string(),
            jti: Uuid::new_v4().to_string(),
            role: "".to_string(), // No role in refresh token
            permissions: vec![],  // No permissions in refresh token
            team_id: None,
            session_id,
            token_type: TokenType::Refresh,
        };

        let header = Header::new(self.algorithm);
        let token = encode(&header, &claims, &self.encoding_key).map_err(GatewayError::Jwt)?;

        debug!("Created refresh token for user: {}", user_id);
        Ok(token)
    }

    /// Create a token pair (access + refresh)
    pub async fn create_token_pair(
        &self,
        user_id: Uuid,
        role: String,
        permissions: Vec<String>,
        team_id: Option<Uuid>,
        session_id: Option<Uuid>,
    ) -> Result<TokenPair> {
        let access_token = self
            .create_access_token(user_id, role, permissions, team_id, session_id)
            .await?;

        let refresh_token = self
            .create_refresh_token(user_id, session_id.map(|id| id.to_string()))
            .await?;

        Ok(TokenPair {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: self.expiration,
        })
    }

    /// Verify and decode a token
    pub async fn verify_token(&self, token: &str) -> Result<Claims> {
        let mut validation = Validation::new(self.algorithm);
        validation.set_issuer(&[&self.issuer]);
        validation.set_audience(&["api", "refresh"]);

        let token_data = decode::<Claims>(token, &self.decoding_key, &validation).map_err(|e| {
            warn!("JWT verification failed: {}", e);
            GatewayError::Jwt(e)
        })?;

        debug!("Token verified for user: {}", token_data.claims.sub);
        Ok(token_data.claims)
    }

    /// Verify a refresh token and return user ID
    pub async fn verify_refresh_token(&self, token: &str) -> Result<Uuid> {
        let claims = self.verify_token(token).await?;

        if !matches!(claims.token_type, TokenType::Refresh) {
            return Err(GatewayError::auth("Invalid token type for refresh"));
        }

        Ok(claims.sub)
    }

    /// Create a password reset token
    pub async fn create_password_reset_token(&self, user_id: Uuid) -> Result<String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| GatewayError::internal(format!("System time error: {}", e)))?
            .as_secs();

        let claims = Claims {
            sub: user_id,
            iat: now,
            exp: now + 3600, // 1 hour expiration
            iss: self.issuer.clone(),
            aud: "password_reset".to_string(),
            jti: Uuid::new_v4().to_string(),
            role: "".to_string(),
            permissions: vec![],
            team_id: None,
            session_id: None,
            token_type: TokenType::PasswordReset,
        };

        let header = Header::new(self.algorithm);
        let token = encode(&header, &claims, &self.encoding_key).map_err(GatewayError::Jwt)?;

        debug!("Created password reset token for user: {}", user_id);
        Ok(token)
    }

    /// Verify a password reset token
    pub async fn verify_password_reset_token(&self, token: &str) -> Result<Uuid> {
        let mut validation = Validation::new(self.algorithm);
        validation.set_issuer(&[&self.issuer]);
        validation.set_audience(&["password_reset"]);

        let token_data =
            decode::<Claims>(token, &self.decoding_key, &validation).map_err(GatewayError::Jwt)?;

        if !matches!(token_data.claims.token_type, TokenType::PasswordReset) {
            return Err(GatewayError::auth("Invalid token type for password reset"));
        }

        Ok(token_data.claims.sub)
    }

    /// Create an email verification token
    pub async fn create_email_verification_token(&self, user_id: Uuid) -> Result<String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| GatewayError::internal(format!("System time error: {}", e)))?
            .as_secs();

        let claims = Claims {
            sub: user_id,
            iat: now,
            exp: now + 86400, // 24 hours expiration
            iss: self.issuer.clone(),
            aud: "email_verification".to_string(),
            jti: Uuid::new_v4().to_string(),
            role: "".to_string(),
            permissions: vec![],
            team_id: None,
            session_id: None,
            token_type: TokenType::EmailVerification,
        };

        let header = Header::new(self.algorithm);
        let token = encode(&header, &claims, &self.encoding_key).map_err(GatewayError::Jwt)?;

        debug!("Created email verification token for user: {}", user_id);
        Ok(token)
    }

    /// Verify an email verification token
    pub async fn verify_email_verification_token(&self, token: &str) -> Result<Uuid> {
        let mut validation = Validation::new(self.algorithm);
        validation.set_issuer(&[&self.issuer]);
        validation.set_audience(&["email_verification"]);

        let token_data =
            decode::<Claims>(token, &self.decoding_key, &validation).map_err(GatewayError::Jwt)?;

        if !matches!(token_data.claims.token_type, TokenType::EmailVerification) {
            return Err(GatewayError::auth(
                "Invalid token type for email verification",
            ));
        }

        Ok(token_data.claims.sub)
    }

    /// Create an invitation token
    pub async fn create_invitation_token(
        &self,
        user_id: Uuid,
        team_id: Uuid,
        role: String,
    ) -> Result<String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| GatewayError::internal(format!("System time error: {}", e)))?
            .as_secs();

        let claims = Claims {
            sub: user_id,
            iat: now,
            exp: now + 604800, // 7 days expiration
            iss: self.issuer.clone(),
            aud: "invitation".to_string(),
            jti: Uuid::new_v4().to_string(),
            role,
            permissions: vec![],
            team_id: Some(team_id),
            session_id: None,
            token_type: TokenType::Invitation,
        };

        let header = Header::new(self.algorithm);
        let token = encode(&header, &claims, &self.encoding_key).map_err(GatewayError::Jwt)?;

        debug!(
            "Created invitation token for user: {} team: {}",
            user_id, team_id
        );
        Ok(token)
    }

    /// Verify an invitation token
    pub async fn verify_invitation_token(&self, token: &str) -> Result<(Uuid, Uuid, String)> {
        let mut validation = Validation::new(self.algorithm);
        validation.set_issuer(&[&self.issuer]);
        validation.set_audience(&["invitation"]);

        let token_data =
            decode::<Claims>(token, &self.decoding_key, &validation).map_err(GatewayError::Jwt)?;

        if !matches!(token_data.claims.token_type, TokenType::Invitation) {
            return Err(GatewayError::auth("Invalid token type for invitation"));
        }

        let team_id = token_data
            .claims
            .team_id
            .ok_or_else(|| GatewayError::auth("Missing team ID in invitation token"))?;

        Ok((token_data.claims.sub, team_id, token_data.claims.role))
    }

    /// Extract token from Authorization header
    pub fn extract_token_from_header(header_value: &str) -> Option<String> {
        header_value
            .strip_prefix("Bearer ")
            .map(|token| token.to_string())
    }

    /// Get token expiration time
    pub fn get_expiration(&self) -> u64 {
        self.expiration
    }

    /// Check if token is expired
    pub fn is_token_expired(&self, claims: &Claims) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(u64::MAX); // If system time is invalid, treat as expired

        claims.exp < now
    }

    /// Get time until token expires
    pub fn time_until_expiry(&self, claims: &Claims) -> Option<u64> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();

        if claims.exp > now {
            Some(claims.exp - now)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AuthConfig;

    async fn create_test_handler() -> JwtHandler {
        let config = AuthConfig {
            jwt_secret: "test_secret_key_for_testing_only".to_string(),
            jwt_expiration: 3600,
            api_key_header: "Authorization".to_string(),
            enable_api_key: true,
            enable_jwt: true,
            rbac: crate::config::RbacConfig {
                enabled: true,
                default_role: "user".to_string(),
                admin_roles: vec!["admin".to_string()],
            },
        };

        JwtHandler::new(&config).await.unwrap()
    }

    #[tokio::test]
    async fn test_create_and_verify_access_token() {
        let handler = create_test_handler().await;
        let user_id = Uuid::new_v4();

        let token = handler
            .create_access_token(
                user_id,
                "user".to_string(),
                vec!["read".to_string()],
                None,
                None,
            )
            .await
            .unwrap();

        let claims = handler.verify_token(&token).await.unwrap();
        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.role, "user");
        assert_eq!(claims.permissions, vec!["read"]);
        assert!(matches!(claims.token_type, TokenType::Access));
    }

    #[tokio::test]
    async fn test_create_token_pair() {
        let handler = create_test_handler().await;
        let user_id = Uuid::new_v4();

        let token_pair = handler
            .create_token_pair(
                user_id,
                "user".to_string(),
                vec!["read".to_string()],
                None,
                None,
            )
            .await
            .unwrap();

        assert!(!token_pair.access_token.is_empty());
        assert!(!token_pair.refresh_token.is_empty());
        assert_eq!(token_pair.token_type, "Bearer");
        assert_eq!(token_pair.expires_in, 3600);

        // Verify both tokens
        let access_claims = handler
            .verify_token(&token_pair.access_token)
            .await
            .unwrap();
        let refresh_user_id = handler
            .verify_refresh_token(&token_pair.refresh_token)
            .await
            .unwrap();

        assert_eq!(access_claims.sub, user_id);
        assert_eq!(refresh_user_id, user_id);
    }

    #[tokio::test]
    async fn test_password_reset_token() {
        let handler = create_test_handler().await;
        let user_id = Uuid::new_v4();

        let token = handler.create_password_reset_token(user_id).await.unwrap();
        let verified_user_id = handler.verify_password_reset_token(&token).await.unwrap();

        assert_eq!(verified_user_id, user_id);
    }

    #[tokio::test]
    async fn test_email_verification_token() {
        let handler = create_test_handler().await;
        let user_id = Uuid::new_v4();

        let token = handler
            .create_email_verification_token(user_id)
            .await
            .unwrap();
        let verified_user_id = handler
            .verify_email_verification_token(&token)
            .await
            .unwrap();

        assert_eq!(verified_user_id, user_id);
    }

    #[tokio::test]
    async fn test_invitation_token() {
        let handler = create_test_handler().await;
        let user_id = Uuid::new_v4();
        let team_id = Uuid::new_v4();

        let token = handler
            .create_invitation_token(user_id, team_id, "member".to_string())
            .await
            .unwrap();
        let (verified_user_id, verified_team_id, role) =
            handler.verify_invitation_token(&token).await.unwrap();

        assert_eq!(verified_user_id, user_id);
        assert_eq!(verified_team_id, team_id);
        assert_eq!(role, "member");
    }

    #[test]
    fn test_extract_token_from_header() {
        let header = "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9";
        let token = JwtHandler::extract_token_from_header(header).unwrap();
        assert_eq!(token, "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9");

        let invalid_header = "Basic dXNlcjpwYXNz";
        assert!(JwtHandler::extract_token_from_header(invalid_header).is_none());
    }

    #[tokio::test]
    async fn test_invalid_token_verification() {
        let handler = create_test_handler().await;
        let invalid_token = "invalid.jwt.token";

        let result = handler.verify_token(invalid_token).await;
        assert!(result.is_err());
    }
}
