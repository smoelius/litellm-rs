//! Webhook and upload token signature utilities

use super::hmac::{constant_time_eq, create_hmac_signature};
use crate::utils::error::Result;
use base64::{Engine as _, engine::general_purpose};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

/// Generate a webhook signature
pub fn generate_webhook_signature(secret: &str, payload: &str, timestamp: u64) -> Result<String> {
    let data = format!("{}.{}", timestamp, payload);
    create_hmac_signature(secret, &data)
}

/// Verify webhook signature
pub fn verify_webhook_signature(
    secret: &str,
    payload: &str,
    timestamp: u64,
    signature: &str,
) -> Result<bool> {
    // Check timestamp is within acceptable range (e.g., 5 minutes)
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    if now.saturating_sub(timestamp) > 300 {
        return Ok(false); // Timestamp too old
    }

    let expected_signature = generate_webhook_signature(secret, payload, timestamp)?;
    Ok(constant_time_eq(&expected_signature, signature))
}

/// Generate a secure file upload token
pub fn generate_upload_token(user_id: &str, expires_at: u64) -> Result<String> {
    let data = format!("{}:{}", user_id, expires_at);
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    Ok(general_purpose::URL_SAFE_NO_PAD.encode(hasher.finalize()))
}

/// Verify file upload token
pub fn verify_upload_token(token: &str, user_id: &str, expires_at: u64) -> Result<bool> {
    let expected_token = generate_upload_token(user_id, expires_at)?;
    Ok(constant_time_eq(&expected_token, token))
}
