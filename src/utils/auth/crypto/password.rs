//! Password hashing and verification using Argon2

use crate::utils::error::{GatewayError, Result};
use argon2::password_hash::{SaltString, rand_core::OsRng};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};

/// Hash a password using Argon2
pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| GatewayError::Crypto(format!("Failed to hash password: {}", e)))?;

    Ok(password_hash.to_string())
}

/// Verify a password against its hash
pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| GatewayError::Crypto(format!("Failed to parse password hash: {}", e)))?;

    let argon2 = Argon2::default();

    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(()) => Ok(true),
        Err(argon2::password_hash::Error::Password) => Ok(false),
        Err(e) => Err(GatewayError::Crypto(format!(
            "Password verification failed: {}",
            e
        ))),
    }
}
