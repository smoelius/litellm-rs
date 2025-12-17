//! Tests for cryptographic utilities

use super::*;
use base64::{Engine as _, engine::general_purpose};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn test_password_hashing() {
    let password = "test_password_123";
    let hash = password::hash_password(password).unwrap();

    assert!(password::verify_password(password, &hash).unwrap());
    assert!(!password::verify_password("wrong_password", &hash).unwrap());
}

#[test]
fn test_api_key_generation() {
    let api_key = keys::generate_api_key();
    assert!(api_key.starts_with("gw-"));
    assert_eq!(api_key.len(), 35); // "gw-" + 32 characters
}

#[test]
fn test_jwt_secret_generation() {
    let secret = keys::generate_jwt_secret();
    assert_eq!(secret.len(), 64);
    assert!(secret.chars().all(|c| c.is_alphanumeric()));
}

#[test]
fn test_api_key_hashing() {
    let api_key = "gw-test123456789";
    let hash = keys::hash_api_key(api_key);
    assert_eq!(hash.len(), 64); // SHA256 hex string
}

#[test]
fn test_api_key_prefix() {
    let api_key = "gw-test123456789";
    let prefix = keys::extract_api_key_prefix(api_key);
    assert_eq!(prefix, "gw-t...6789");
}

#[test]
fn test_hmac_signature() {
    let secret = "test_secret";
    let data = "test_data";

    let signature = hmac::create_hmac_signature(secret, data).unwrap();
    assert!(hmac::verify_hmac_signature(secret, data, &signature).unwrap());
    assert!(!hmac::verify_hmac_signature(secret, "wrong_data", &signature).unwrap());
}

#[test]
fn test_hmac_sha256_specific_case() {
    // Test the specific case mentioned in the question
    let key = "key";
    let message = "message";

    let signature = hmac::create_hmac_signature(key, message).unwrap();
    println!(
        "HMAC-SHA256 for key='{}', message='{}': {}",
        key, message, signature
    );

    // Verify the signature is correctly calculated
    assert!(hmac::verify_hmac_signature(key, message, &signature).unwrap());

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

    let signature = hmac::create_hmac_signature(key, data).unwrap();
    assert_eq!(signature, expected, "RFC 4231 Test Case 2 failed");
}

#[test]
fn test_constant_time_eq() {
    assert!(hmac::constant_time_eq("hello", "hello"));
    assert!(!hmac::constant_time_eq("hello", "world"));
    assert!(!hmac::constant_time_eq("hello", "hello2"));
}

#[test]
fn test_backup_code_generation() {
    let code = backup::generate_backup_code();
    assert_eq!(code.len(), 9); // 4 digits + "-" + 4 digits
    assert!(code.contains('-'));

    let codes = backup::generate_backup_codes(5);
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

    let signature = webhooks::generate_webhook_signature(secret, payload, timestamp).unwrap();
    assert!(webhooks::verify_webhook_signature(secret, payload, timestamp, &signature).unwrap());

    // Test with wrong payload
    assert!(!webhooks::verify_webhook_signature(secret, "wrong", timestamp, &signature).unwrap());

    // Test with old timestamp (should fail)
    let old_timestamp = timestamp - 400; // More than 5 minutes old
    let old_signature = webhooks::generate_webhook_signature(secret, payload, old_timestamp).unwrap();
    assert!(!webhooks::verify_webhook_signature(secret, payload, old_timestamp, &old_signature).unwrap());
}

#[test]
fn test_aes_gcm_encryption_decryption() {
    let key = b"my_secret_encryption_key_123456";
    let plaintext = "Hello, World! This is sensitive data.";

    // Encrypt
    let encrypted = encryption::encrypt_data(key, plaintext).unwrap();

    // Encrypted output should be base64 and different from plaintext
    assert_ne!(encrypted, plaintext);
    assert!(encrypted.len() > plaintext.len()); // Includes nonce + tag

    // Decrypt
    let decrypted = encryption::decrypt_data(key, &encrypted).unwrap();
    assert_eq!(decrypted, plaintext);
}

#[test]
fn test_aes_gcm_different_nonces() {
    let key = b"test_key_for_nonce_uniqueness!!";
    let plaintext = "Same message encrypted twice";

    // Encrypt same plaintext twice
    let encrypted1 = encryption::encrypt_data(key, plaintext).unwrap();
    let encrypted2 = encryption::encrypt_data(key, plaintext).unwrap();

    // Each encryption should produce different ciphertext (due to random nonce)
    assert_ne!(encrypted1, encrypted2);

    // Both should decrypt to the same plaintext
    assert_eq!(encryption::decrypt_data(key, &encrypted1).unwrap(), plaintext);
    assert_eq!(encryption::decrypt_data(key, &encrypted2).unwrap(), plaintext);
}

#[test]
fn test_aes_gcm_wrong_key() {
    let key1 = b"correct_key_for_encryption_1234";
    let key2 = b"wrong_key_for_decryption_5678!!";
    let plaintext = "Secret message";

    let encrypted = encryption::encrypt_data(key1, plaintext).unwrap();

    // Decryption with wrong key should fail
    let result = encryption::decrypt_data(key2, &encrypted);
    assert!(result.is_err());
}

#[test]
fn test_aes_gcm_tampered_data() {
    let key = b"key_for_tamper_test_1234567890!";
    let plaintext = "Important data";

    let encrypted = encryption::encrypt_data(key, plaintext).unwrap();

    // Tamper with the encrypted data
    let mut tampered_bytes = general_purpose::STANDARD.decode(&encrypted).unwrap();
    if let Some(byte) = tampered_bytes.last_mut() {
        *byte ^= 0xFF; // Flip bits in the last byte
    }
    let tampered = general_purpose::STANDARD.encode(&tampered_bytes);

    // Decryption should fail due to authentication tag mismatch
    let result = encryption::decrypt_data(key, &tampered);
    assert!(result.is_err());
}

#[test]
fn test_aes_gcm_short_data_rejected() {
    let key = b"test_key_for_short_data_check!!";

    // Data too short (less than nonce + auth tag)
    let short_data = general_purpose::STANDARD.encode([0u8; 10]);
    let result = encryption::decrypt_data(key, &short_data);
    assert!(result.is_err());
}

#[test]
fn test_aes_gcm_empty_plaintext() {
    let key = b"key_for_empty_plaintext_test!!!";
    let plaintext = "";

    let encrypted = encryption::encrypt_data(key, plaintext).unwrap();
    let decrypted = encryption::decrypt_data(key, &encrypted).unwrap();
    assert_eq!(decrypted, plaintext);
}

#[test]
fn test_aes_gcm_unicode_plaintext() {
    let key = b"key_for_unicode_test_1234567890";
    let plaintext = "Hello 世界! Привет мир! 🔐🔑";

    let encrypted = encryption::encrypt_data(key, plaintext).unwrap();
    let decrypted = encryption::decrypt_data(key, &encrypted).unwrap();
    assert_eq!(decrypted, plaintext);
}

#[test]
fn test_aes_gcm_large_plaintext() {
    let key = b"key_for_large_data_test_1234567";
    let plaintext = "A".repeat(10000); // 10KB of data

    let encrypted = encryption::encrypt_data(key, &plaintext).unwrap();
    let decrypted = encryption::decrypt_data(key, &encrypted).unwrap();
    assert_eq!(decrypted, plaintext);
}
