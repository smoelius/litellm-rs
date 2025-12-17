//! JWT types and data structures

use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// JWT handler for token operations
#[derive(Clone)]
pub struct JwtHandler {
    /// Encoding key for signing tokens
    pub(super) encoding_key: EncodingKey,
    /// Decoding key for verifying tokens
    pub(super) decoding_key: DecodingKey,
    /// JWT algorithm
    pub(super) algorithm: Algorithm,
    /// Token expiration time in seconds
    pub(super) expiration: u64,
    /// Token issuer
    pub(super) issuer: String,
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
