//! Encryption utilities including AES-GCM and hashing with salt

use crate::utils::error::{GatewayError, Result};
use aes_gcm::{
    Aes256Gcm, Key, Nonce,
    aead::{Aead, KeyInit},
};
use base64::{Engine as _, engine::general_purpose};
use rand::{Rng, RngCore};
use sha2::{Digest, Sha256};

/// AES-256-GCM nonce size (96 bits / 12 bytes as recommended by NIST)
const AES_GCM_NONCE_SIZE: usize = 12;

/// Derive a 256-bit key from arbitrary-length input using SHA-256
fn derive_key(key: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(key);
    hasher.finalize().into()
}

/// Encrypt data using AES-256-GCM with authenticated encryption.
///
/// The output format is: base64(nonce || ciphertext || tag)
/// - nonce: 12 bytes (randomly generated)
/// - ciphertext: variable length (same as plaintext)
/// - tag: 16 bytes (authentication tag)
///
/// # Security
/// - Uses cryptographically secure random nonce for each encryption
/// - Provides both confidentiality and integrity protection
/// - Key is derived using SHA-256 if not exactly 32 bytes
pub fn encrypt_data(key: &[u8], data: &str) -> Result<String> {
    // Derive 256-bit key from input
    let derived_key = derive_key(key);
    let cipher_key = Key::<Aes256Gcm>::from_slice(&derived_key);
    let cipher = Aes256Gcm::new(cipher_key);

    // Generate random 96-bit nonce (12 bytes)
    let mut nonce_bytes = [0u8; AES_GCM_NONCE_SIZE];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Encrypt the data
    let ciphertext = cipher
        .encrypt(nonce, data.as_bytes())
        .map_err(|e| GatewayError::Crypto(format!("Encryption failed: {}", e)))?;

    // Prepend nonce to ciphertext for storage
    let mut output = Vec::with_capacity(AES_GCM_NONCE_SIZE + ciphertext.len());
    output.extend_from_slice(&nonce_bytes);
    output.extend_from_slice(&ciphertext);

    // Encode as base64 for safe storage/transmission
    Ok(general_purpose::STANDARD.encode(&output))
}

/// Decrypt data encrypted with AES-256-GCM.
///
/// Expects input format: base64(nonce || ciphertext || tag)
///
/// # Security
/// - Verifies authentication tag before returning plaintext
/// - Returns error if data has been tampered with
pub fn decrypt_data(key: &[u8], encrypted_data: &str) -> Result<String> {
    // Decode base64 encrypted data
    let encrypted_bytes = general_purpose::STANDARD
        .decode(encrypted_data)
        .map_err(|e| GatewayError::Crypto(format!("Failed to decode encrypted data: {}", e)))?;

    // Validate minimum length (nonce + at least 16-byte auth tag)
    if encrypted_bytes.len() < AES_GCM_NONCE_SIZE + 16 {
        return Err(GatewayError::Crypto(
            "Encrypted data too short - possible corruption or tampering".to_string(),
        ));
    }

    // Derive 256-bit key from input
    let derived_key = derive_key(key);
    let cipher_key = Key::<Aes256Gcm>::from_slice(&derived_key);
    let cipher = Aes256Gcm::new(cipher_key);

    // Extract nonce and ciphertext
    let nonce = Nonce::from_slice(&encrypted_bytes[..AES_GCM_NONCE_SIZE]);
    let ciphertext = &encrypted_bytes[AES_GCM_NONCE_SIZE..];

    // Decrypt and verify authentication tag
    let plaintext = cipher.decrypt(nonce, ciphertext).map_err(|_| {
        GatewayError::Crypto(
            "Decryption failed - data may have been tampered with or wrong key".to_string(),
        )
    })?;

    String::from_utf8(plaintext).map_err(|e| {
        GatewayError::Crypto(format!("Failed to convert decrypted data to string: {}", e))
    })
}

/// Generate a secure random salt
pub fn generate_salt() -> String {
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..16).map(|_| rng.r#gen()).collect();
    general_purpose::STANDARD.encode(&bytes)
}

/// Hash data with salt
pub fn hash_with_salt(data: &str, salt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    hasher.update(salt.as_bytes());
    hex::encode(hasher.finalize())
}

/// Generate a time-based one-time password (TOTP) secret
pub fn generate_totp_secret() -> String {
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..20).map(|_| rng.r#gen()).collect();
    general_purpose::STANDARD.encode(&bytes)
}
