use crate::core::providers::unified_provider::ProviderError;
use base64::{Engine, engine::general_purpose::STANDARD};

pub struct Base64Ops;

impl Base64Ops {
    pub fn is_base64_encoded(s: &str) -> bool {
        if s.is_empty() {
            return false;
        }

        if !s.len().is_multiple_of(4) {
            return false;
        }

        STANDARD.decode(s).is_ok()
    }

    pub fn get_base64_string(s: &str) -> String {
        if Self::is_base64_encoded(s) {
            s.to_string()
        } else {
            STANDARD.encode(s.as_bytes())
        }
    }

    pub fn decode_base64(s: &str) -> Result<String, ProviderError> {
        let decoded_bytes = STANDARD
            .decode(s)
            .map_err(|e| ProviderError::InvalidRequest {
                provider: "unknown",
                message: format!("Invalid base64 string: {}", e),
            })?;

        String::from_utf8(decoded_bytes).map_err(|e| ProviderError::InvalidRequest {
            provider: "unknown",
            message: format!("Invalid UTF-8 in decoded base64: {}", e),
        })
    }

    pub fn encode_base64(s: &str) -> String {
        STANDARD.encode(s.as_bytes())
    }
}
