use crate::core::providers::unified_provider::ProviderError;
use serde_json::Value;
use sha2::Digest;

pub struct Serialization;

impl Serialization {
    pub fn deep_clone_json(data: &Value) -> Value {
        data.clone()
    }

    pub fn json_size_bytes(data: &Value) -> usize {
        serde_json::to_string(data).map(|s| s.len()).unwrap_or(0)
    }

    pub fn pretty_print_json(data: &Value) -> Result<String, ProviderError> {
        serde_json::to_string_pretty(data).map_err(|e| ProviderError::InvalidRequest {
            provider: "unknown",
            message: format!("Failed to pretty print JSON: {}", e),
        })
    }

    pub fn compact_json(data: &Value) -> Result<String, ProviderError> {
        serde_json::to_string(data).map_err(|e| ProviderError::InvalidRequest {
            provider: "unknown",
            message: format!("Failed to compact JSON: {}", e),
        })
    }

    pub fn hash_json(data: &Value) -> Result<String, ProviderError> {
        let json_str = Self::compact_json(data)?;
        let hash = sha2::Sha256::digest(json_str.as_bytes());
        Ok(format!("{:x}", hash))
    }
}
