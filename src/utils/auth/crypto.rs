//! Cryptographic utilities for the Gateway
//!
//! This module provides cryptographic functions for password hashing, API key generation,
//! and authenticated encryption using AES-256-GCM.

#![allow(dead_code)]

use crate::utils::error::{GatewayError, Result};
use aes_gcm::{
    Aes256Gcm, Key, Nonce,
    aead::{Aead, KeyInit},
};
use argon2::password_hash::{SaltString, rand_core::OsRng};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use base64::{Engine as _, engine::general_purpose};
use hmac::{Hmac, Mac, digest::KeyInit as HmacKeyInit};
use rand::{Rng, RngCore, distributions::Alphanumeric};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

type HmacSha256 = Hmac<Sha256>;

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

/// Generate a secure API key
pub fn generate_api_key() -> String {
    let prefix = "gw";
    let random_part: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    format!("{}-{}", prefix, random_part)
}

/// Generate a JWT secret
pub fn generate_jwt_secret() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(64)
        .map(char::from)
        .collect()
}

/// Generate a secure random token
pub fn generate_token(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

/// Generate a secure session token
pub fn generate_session_token() -> String {
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..32).map(|_| rng.r#gen()).collect();
    general_purpose::URL_SAFE_NO_PAD.encode(&bytes)
}

/// Hash API key for storage
pub fn hash_api_key(api_key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(api_key.as_bytes());
    hex::encode(hasher.finalize())
}

/// Generate API key prefix for identification
pub fn extract_api_key_prefix(api_key: &str) -> String {
    if api_key.len() >= 8 {
        format!("{}...{}", &api_key[..4], &api_key[api_key.len() - 4..])
    } else {
        api_key.to_string()
    }
}

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
fn constant_time_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (a_byte, b_byte) in a.bytes().zip(b.bytes()) {
        result |= a_byte ^ b_byte;
    }

    result == 0
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

/// Generate a secure backup code
pub fn generate_backup_code() -> String {
    let mut rng = rand::thread_rng();
    let code: String = (0..8)
        .map(|_| rng.gen_range(0..10).to_string())
        .collect::<Vec<_>>()
        .chunks(4)
        .map(|chunk| chunk.join(""))
        .collect::<Vec<_>>()
        .join("-");
    code
}

/// Generate multiple backup codes
pub fn generate_backup_codes(count: usize) -> Vec<String> {
    (0..count).map(|_| generate_backup_code()).collect()
}

/// Hash backup code for storage
pub fn hash_backup_code(code: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(code.as_bytes());
    hasher.update(b"backup_code_salt"); // Simple salt
    hex::encode(hasher.finalize())
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let password = "test_password_123";
        let hash = hash_password(password).unwrap();

        assert!(verify_password(password, &hash).unwrap());
        assert!(!verify_password("wrong_password", &hash).unwrap());
    }

    #[test]
    fn test_api_key_generation() {
        let api_key = generate_api_key();
        assert!(api_key.starts_with("gw-"));
        assert_eq!(api_key.len(), 35); // "gw-" + 32 characters
    }

    #[test]
    fn test_jwt_secret_generation() {
        let secret = generate_jwt_secret();
        assert_eq!(secret.len(), 64);
        assert!(secret.chars().all(|c| c.is_alphanumeric()));
    }

    #[test]
    fn test_api_key_hashing() {
        let api_key = "gw-test123456789";
        let hash = hash_api_key(api_key);
        assert_eq!(hash.len(), 64); // SHA256 hex string
    }

    #[test]
    fn test_api_key_prefix() {
        let api_key = "gw-test123456789";
        let prefix = extract_api_key_prefix(api_key);
        assert_eq!(prefix, "gw-t...6789");
    }

    #[test]
    fn test_hmac_signature() {
        let secret = "test_secret";
        let data = "test_data";

        let signature = create_hmac_signature(secret, data).unwrap();
        assert!(verify_hmac_signature(secret, data, &signature).unwrap());
        assert!(!verify_hmac_signature(secret, "wrong_data", &signature).unwrap());
    }

    #[test]
    fn test_hmac_sha256_specific_case() {
        // Test the specific case mentioned in the question
        let key = "key";
        let message = "message";

        let signature = create_hmac_signature(key, message).unwrap();
        println!(
            "HMAC-SHA256 for key='{}', message='{}': {}",
            key, message, signature
        );

        // Verify the signature is correctly calculated
        assert!(verify_hmac_signature(key, message, &signature).unwrap());

        // The correct HMAC-SHA256 for key="key" and message="message"
        let expected = "6e9ef29b75fffc5b7abae527d58fdadb2fe42e7219011976917343065f58ed4a";
        assert_eq!(signature, expected, "HMAC-SHA256 calculation mismatch");

        // Also test against the incorrect value that was mentioned
        let incorrect = "6e9ef29b75fffc5b7abae527d58fdadb2fe42e7219011e917a9c6e0c3d5e4c3b";
        assert_ne!(signature, incorrect, "Should not match the incorrect hash");
    }

    #[test]
    fn test_hmac_sha256_rfc4231_vectors() {
        // Test Case 2 from RFC 4231
        let key = "Jefe";
        let data = "what do ya want for nothing?";
        let expected = "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843";

        let signature = create_hmac_signature(key, data).unwrap();
        assert_eq!(signature, expected, "RFC 4231 Test Case 2 failed");
    }

    #[test]
    fn test_constant_time_eq() {
        assert!(constant_time_eq("hello", "hello"));
        assert!(!constant_time_eq("hello", "world"));
        assert!(!constant_time_eq("hello", "hello2"));
    }

    #[test]
    fn test_backup_code_generation() {
        let code = generate_backup_code();
        assert_eq!(code.len(), 9); // 4 digits + "-" + 4 digits
        assert!(code.contains('-'));

        let codes = generate_backup_codes(5);
        assert_eq!(codes.len(), 5);
        assert!(codes.iter().all(|c| c.len() == 9));
    }

    #[test]
    fn test_webhook_signature() {
        let secret = "webhook_secret";
        let payload = r#"{"test": "data"}"#;
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let signature = generate_webhook_signature(secret, payload, timestamp).unwrap();
        assert!(verify_webhook_signature(secret, payload, timestamp, &signature).unwrap());

        // Test with wrong payload
        assert!(!verify_webhook_signature(secret, "wrong", timestamp, &signature).unwrap());

        // Test with old timestamp (should fail)
        let old_timestamp = timestamp - 400; // More than 5 minutes old
        let old_signature = generate_webhook_signature(secret, payload, old_timestamp).unwrap();
        assert!(!verify_webhook_signature(secret, payload, old_timestamp, &old_signature).unwrap());
    }

    #[test]
    fn test_aes_gcm_encryption_decryption() {
        let key = b"my_secret_encryption_key_123456";
        let plaintext = "Hello, World! This is sensitive data.";

        // Encrypt
        let encrypted = encrypt_data(key, plaintext).unwrap();

        // Encrypted output should be base64 and different from plaintext
        assert_ne!(encrypted, plaintext);
        assert!(encrypted.len() > plaintext.len()); // Includes nonce + tag

        // Decrypt
        let decrypted = decrypt_data(key, &encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_aes_gcm_different_nonces() {
        let key = b"test_key_for_nonce_uniqueness!!";
        let plaintext = "Same message encrypted twice";

        // Encrypt same plaintext twice
        let encrypted1 = encrypt_data(key, plaintext).unwrap();
        let encrypted2 = encrypt_data(key, plaintext).unwrap();

        // Each encryption should produce different ciphertext (due to random nonce)
        assert_ne!(encrypted1, encrypted2);

        // Both should decrypt to the same plaintext
        assert_eq!(decrypt_data(key, &encrypted1).unwrap(), plaintext);
        assert_eq!(decrypt_data(key, &encrypted2).unwrap(), plaintext);
    }

    #[test]
    fn test_aes_gcm_wrong_key() {
        let key1 = b"correct_key_for_encryption_1234";
        let key2 = b"wrong_key_for_decryption_5678!!";
        let plaintext = "Secret message";

        let encrypted = encrypt_data(key1, plaintext).unwrap();

        // Decryption with wrong key should fail
        let result = decrypt_data(key2, &encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn test_aes_gcm_tampered_data() {
        let key = b"key_for_tamper_test_1234567890!";
        let plaintext = "Important data";

        let encrypted = encrypt_data(key, plaintext).unwrap();

        // Tamper with the encrypted data
        let mut tampered_bytes = general_purpose::STANDARD.decode(&encrypted).unwrap();
        if let Some(byte) = tampered_bytes.last_mut() {
            *byte ^= 0xFF; // Flip bits in the last byte
        }
        let tampered = general_purpose::STANDARD.encode(&tampered_bytes);

        // Decryption should fail due to authentication tag mismatch
        let result = decrypt_data(key, &tampered);
        assert!(result.is_err());
    }

    #[test]
    fn test_aes_gcm_short_data_rejected() {
        let key = b"test_key_for_short_data_check!!";

        // Data too short (less than nonce + auth tag)
        let short_data = general_purpose::STANDARD.encode([0u8; 10]);
        let result = decrypt_data(key, &short_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_aes_gcm_empty_plaintext() {
        let key = b"key_for_empty_plaintext_test!!!";
        let plaintext = "";

        let encrypted = encrypt_data(key, plaintext).unwrap();
        let decrypted = decrypt_data(key, &encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_aes_gcm_unicode_plaintext() {
        let key = b"key_for_unicode_test_1234567890";
        let plaintext = "Hello 世界! Привет мир! 🔐🔑";

        let encrypted = encrypt_data(key, plaintext).unwrap();
        let decrypted = decrypt_data(key, &encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_aes_gcm_large_plaintext() {
        let key = b"key_for_large_data_test_1234567";
        let plaintext = "A".repeat(10000); // 10KB of data

        let encrypted = encrypt_data(key, &plaintext).unwrap();
        let decrypted = decrypt_data(key, &encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }
}
