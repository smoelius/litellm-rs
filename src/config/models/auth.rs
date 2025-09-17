//! Authentication configuration

use super::*;
use rand::distributions::Alphanumeric;
use rand::{Rng, thread_rng};
use serde::{Deserialize, Serialize};
use tracing::warn;

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Enable JWT authentication
    #[serde(default = "default_true")]
    pub enable_jwt: bool,
    /// Enable API key authentication
    #[serde(default = "default_true")]
    pub enable_api_key: bool,
    /// JWT secret
    pub jwt_secret: String,
    /// JWT expiration in seconds
    #[serde(default = "default_jwt_expiration")]
    pub jwt_expiration: u64,
    /// API key header name
    #[serde(default = "default_api_key_header")]
    pub api_key_header: String,
    /// RBAC configuration
    #[serde(default)]
    pub rbac: RbacConfig,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            enable_jwt: true,
            enable_api_key: true,
            jwt_secret: generate_secure_jwt_secret(),
            jwt_expiration: default_jwt_expiration(),
            api_key_header: default_api_key_header(),
            rbac: RbacConfig::default(),
        }
    }
}

#[allow(dead_code)]
impl AuthConfig {
    /// Merge auth configurations
    pub fn merge(mut self, other: Self) -> Self {
        if !other.enable_jwt {
            self.enable_jwt = other.enable_jwt;
        }
        if !other.enable_api_key {
            self.enable_api_key = other.enable_api_key;
        }
        if !other.jwt_secret.is_empty() && other.jwt_secret != "your-secret-key" {
            self.jwt_secret = other.jwt_secret;
        }
        if other.jwt_expiration != default_jwt_expiration() {
            self.jwt_expiration = other.jwt_expiration;
        }
        if other.api_key_header != default_api_key_header() {
            self.api_key_header = other.api_key_header;
        }
        self.rbac = self.rbac.merge(other.rbac);
        self
    }

    /// Validate authentication configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate JWT secret strength
        if self.enable_jwt {
            if self.jwt_secret.len() < 32 {
                return Err(
                    "JWT secret must be at least 32 characters long for security".to_string(),
                );
            }

            if self.jwt_secret == "your-secret-key" || self.jwt_secret == "change-me" {
                return Err("JWT secret must not use default values. Please generate a secure random secret.".to_string());
            }

            // Check for common weak patterns
            if self.jwt_secret.chars().all(|c| c.is_ascii_lowercase()) {
                return Err(
                    "JWT secret should contain mixed case letters, numbers, and special characters"
                        .to_string(),
                );
            }
        }

        // Validate JWT expiration
        if self.jwt_expiration < 300 {
            return Err("JWT expiration should be at least 5 minutes (300 seconds)".to_string());
        }

        if self.jwt_expiration > 86400 * 30 {
            return Err(
                "JWT expiration should not exceed 30 days for security reasons".to_string(),
            );
        }

        // Validate API key header
        if self.enable_api_key && self.api_key_header.is_empty() {
            return Err(
                "API key header name cannot be empty when API key auth is enabled".to_string(),
            );
        }

        Ok(())
    }

    /// Check if authentication is properly configured for production
    pub fn is_production_ready(&self) -> bool {
        self.enable_jwt || self.enable_api_key
    }
}

/// RBAC configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RbacConfig {
    /// Enable RBAC
    #[serde(default)]
    pub enabled: bool,
    /// Default role for new users
    #[serde(default = "default_role")]
    pub default_role: String,
    /// Admin roles
    #[serde(default = "default_admin_roles")]
    pub admin_roles: Vec<String>,
}

impl Default for RbacConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            default_role: default_role(),
            admin_roles: default_admin_roles(),
        }
    }
}

#[allow(dead_code)]
impl RbacConfig {
    /// Merge RBAC configurations
    pub fn merge(mut self, other: Self) -> Self {
        if other.enabled {
            self.enabled = other.enabled;
        }
        if other.default_role != default_role() {
            self.default_role = other.default_role;
        }
        if other.admin_roles != default_admin_roles() {
            self.admin_roles = other.admin_roles;
        }
        self
    }
}

fn default_true() -> bool {
    true
}

/// Generate a secure random JWT secret
fn generate_secure_jwt_secret() -> String {
    // Generate a 64-character secure random string
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(64)
        .map(char::from)
        .collect()
}

/// Warn about insecure configuration in development
pub fn warn_insecure_config(config: &AuthConfig) {
    if !config.is_production_ready() {
        warn!(
            "Authentication is disabled! This is insecure for production use. Enable JWT or API key authentication before deploying to production."
        );
    }
}
