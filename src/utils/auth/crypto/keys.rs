//! Key and token generation utilities

use base64::{Engine as _, engine::general_purpose};
use rand::{Rng, distributions::Alphanumeric};
use sha2::{Digest, Sha256};

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
