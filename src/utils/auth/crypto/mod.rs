//! Cryptographic utilities for the Gateway
//!
//! This module provides cryptographic functions for password hashing, API key generation,
//! and authenticated encryption using AES-256-GCM.

#![allow(dead_code)]

pub mod backup;
pub mod encryption;
pub mod hmac;
pub mod keys;
pub mod password;
pub mod webhooks;

#[cfg(test)]
mod tests;

// Re-export all public functions for backward compatibility
pub use backup::{generate_backup_code, generate_backup_codes, hash_backup_code};
pub use encryption::{
    decrypt_data, encrypt_data, generate_salt, generate_totp_secret, hash_with_salt,
};
pub use hmac::{create_hmac_signature, verify_hmac_signature};
pub use keys::{
    extract_api_key_prefix, generate_api_key, generate_jwt_secret, generate_session_token,
    generate_token, hash_api_key,
};
pub use password::{hash_password, verify_password};
pub use webhooks::{
    generate_upload_token, generate_webhook_signature, verify_upload_token,
    verify_webhook_signature,
};
