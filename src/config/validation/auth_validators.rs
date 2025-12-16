//! Authentication configuration validators
//!
//! This module provides validation implementations for authentication-related
//! configuration structures including AuthConfig and RbacConfig.

use super::trait_def::Validate;
use crate::config::models::*;
use tracing::debug;

impl Validate for AuthConfig {
    fn validate(&self) -> Result<(), String> {
        debug!("Validating auth configuration");

        if self.jwt_secret.is_empty() {
            return Err("JWT secret cannot be empty".to_string());
        }

        if self.jwt_secret == "change-me-in-production" && !cfg!(test) {
            return Err("JWT secret must be changed from default value in production".to_string());
        }

        if self.jwt_secret.len() < 32 {
            return Err("JWT secret should be at least 32 characters long".to_string());
        }

        if self.jwt_expiration == 0 {
            return Err("JWT expiration must be greater than 0".to_string());
        }

        if self.jwt_expiration > 86400 * 30 { // 30 days
            return Err("JWT expiration should not exceed 30 days".to_string());
        }

        if self.api_key_header.is_empty() {
            return Err("API key header cannot be empty".to_string());
        }

        self.rbac.validate()?;

        Ok(())
    }
}

impl Validate for RbacConfig {
    fn validate(&self) -> Result<(), String> {
        if self.enabled && self.default_role.is_empty() {
            return Err("Default role cannot be empty when RBAC is enabled".to_string());
        }

        Ok(())
    }
}
