//! Data validation utilities

use crate::utils::error::{GatewayError, Result};
use regex::Regex;
use std::collections::HashSet;

/// Data validation utilities
pub struct DataValidator;

impl DataValidator {
    /// Validate username
    pub fn validate_username(username: &str) -> Result<()> {
        if username.trim().is_empty() {
            return Err(GatewayError::Validation(
                "Username cannot be empty".to_string(),
            ));
        }

        if username.len() < 3 {
            return Err(GatewayError::Validation(
                "Username must be at least 3 characters".to_string(),
            ));
        }

        if username.len() > 50 {
            return Err(GatewayError::Validation(
                "Username cannot exceed 50 characters".to_string(),
            ));
        }

        let username_regex = Regex::new(r"^[a-zA-Z0-9_-]+$")
            .map_err(|e| GatewayError::Internal(format!("Regex error: {}", e)))?;

        if !username_regex.is_match(username) {
            return Err(GatewayError::Validation(
                "Username can only contain letters, numbers, underscores, and hyphens".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate password strength
    pub fn validate_password(password: &str) -> Result<()> {
        if password.len() < 8 {
            return Err(GatewayError::Validation(
                "Password must be at least 8 characters".to_string(),
            ));
        }

        if password.len() > 128 {
            return Err(GatewayError::Validation(
                "Password cannot exceed 128 characters".to_string(),
            ));
        }

        let has_lowercase = password.chars().any(|c| c.is_lowercase());
        let has_uppercase = password.chars().any(|c| c.is_uppercase());
        let has_digit = password.chars().any(|c| c.is_ascii_digit());
        let has_special = password
            .chars()
            .any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c));

        let strength_count = [has_lowercase, has_uppercase, has_digit, has_special]
            .iter()
            .filter(|&&x| x)
            .count();

        if strength_count < 3 {
            return Err(GatewayError::Validation(
                "Password must contain at least 3 of: lowercase, uppercase, digit, special character".to_string()
            ));
        }

        Ok(())
    }

    /// Validate team name
    pub fn validate_team_name(name: &str) -> Result<()> {
        if name.trim().is_empty() {
            return Err(GatewayError::Validation(
                "Team name cannot be empty".to_string(),
            ));
        }

        if name.len() < 2 {
            return Err(GatewayError::Validation(
                "Team name must be at least 2 characters".to_string(),
            ));
        }

        if name.len() > 100 {
            return Err(GatewayError::Validation(
                "Team name cannot exceed 100 characters".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate tags
    pub fn validate_tags(tags: &[String]) -> Result<()> {
        if tags.len() > 20 {
            return Err(GatewayError::Validation(
                "Cannot have more than 20 tags".to_string(),
            ));
        }

        let mut unique_tags = HashSet::new();
        for tag in tags {
            if tag.trim().is_empty() {
                return Err(GatewayError::Validation("Tag cannot be empty".to_string()));
            }

            if tag.len() > 50 {
                return Err(GatewayError::Validation(
                    "Tag cannot exceed 50 characters".to_string(),
                ));
            }

            if !unique_tags.insert(tag.to_lowercase()) {
                return Err(GatewayError::Validation(format!("Duplicate tag: {}", tag)));
            }
        }

        Ok(())
    }
}
