//! HMAC signature creation and verification

use crate::utils::error::{GatewayError, Result};
use hmac::{Hmac, Mac, digest::KeyInit as HmacKeyInit};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Create HMAC signature
pub fn create_hmac_signature(secret: &str, data: &str) -> Result<String> {
    let mut mac = <HmacSha256 as HmacKeyInit>::new_from_slice(secret.as_bytes())
        .map_err(|e| GatewayError::Crypto(format!("Invalid HMAC key: {}", e)))?;

    mac.update(data.as_bytes());
    let result = mac.finalize();
    Ok(hex::encode(result.into_bytes()))
}

/// Verify HMAC signature
pub fn verify_hmac_signature(secret: &str, data: &str, signature: &str) -> Result<bool> {
    let expected_signature = create_hmac_signature(secret, data)?;
    Ok(constant_time_eq(&expected_signature, signature))
}

/// Constant-time string comparison
pub(crate) fn constant_time_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (a_byte, b_byte) in a.bytes().zip(b.bytes()) {
        result |= a_byte ^ b_byte;
    }

    result == 0
}
