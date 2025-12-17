//! Backup code generation and hashing

use rand::Rng;
use sha2::{Digest, Sha256};

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
