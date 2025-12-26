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

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== is_base64_encoded Tests ====================

    #[test]
    fn test_is_base64_encoded_valid() {
        let encoded = STANDARD.encode("hello world");
        assert!(Base64Ops::is_base64_encoded(&encoded));
    }

    #[test]
    fn test_is_base64_encoded_empty() {
        assert!(!Base64Ops::is_base64_encoded(""));
    }

    #[test]
    fn test_is_base64_encoded_invalid_length() {
        assert!(!Base64Ops::is_base64_encoded("abc"));
    }

    #[test]
    fn test_is_base64_encoded_invalid_chars() {
        assert!(!Base64Ops::is_base64_encoded("!!!!"));
    }

    #[test]
    fn test_is_base64_encoded_plain_text() {
        assert!(!Base64Ops::is_base64_encoded("hello world"));
    }

    #[test]
    fn test_is_base64_encoded_with_padding() {
        let encoded = STANDARD.encode("hi");
        assert!(Base64Ops::is_base64_encoded(&encoded));
    }

    // ==================== get_base64_string Tests ====================

    #[test]
    fn test_get_base64_string_already_encoded() {
        let encoded = STANDARD.encode("test");
        let result = Base64Ops::get_base64_string(&encoded);
        assert_eq!(result, encoded);
    }

    #[test]
    fn test_get_base64_string_not_encoded() {
        let plain = "hello world";
        let result = Base64Ops::get_base64_string(plain);
        let expected = STANDARD.encode(plain);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_get_base64_string_empty() {
        let result = Base64Ops::get_base64_string("");
        assert_eq!(result, "");
    }

    #[test]
    fn test_get_base64_string_unicode() {
        let unicode = "你好世界";
        let result = Base64Ops::get_base64_string(unicode);
        let decoded = STANDARD.decode(&result).unwrap();
        assert_eq!(String::from_utf8(decoded).unwrap(), unicode);
    }

    // ==================== decode_base64 Tests ====================

    #[test]
    fn test_decode_base64_valid() {
        let encoded = STANDARD.encode("hello world");
        let result = Base64Ops::decode_base64(&encoded).unwrap();
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_decode_base64_empty() {
        let result = Base64Ops::decode_base64("").unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_decode_base64_invalid() {
        let result = Base64Ops::decode_base64("not-valid-base64!!!");
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_base64_unicode() {
        let original = "こんにちは";
        let encoded = STANDARD.encode(original);
        let result = Base64Ops::decode_base64(&encoded).unwrap();
        assert_eq!(result, original);
    }

    #[test]
    fn test_decode_base64_special_chars() {
        let original = "line1\nline2\ttab";
        let encoded = STANDARD.encode(original);
        let result = Base64Ops::decode_base64(&encoded).unwrap();
        assert_eq!(result, original);
    }

    // ==================== encode_base64 Tests ====================

    #[test]
    fn test_encode_base64_simple() {
        let result = Base64Ops::encode_base64("hello");
        assert_eq!(result, STANDARD.encode("hello"));
    }

    #[test]
    fn test_encode_base64_empty() {
        let result = Base64Ops::encode_base64("");
        assert_eq!(result, "");
    }

    #[test]
    fn test_encode_base64_unicode() {
        let unicode = "emoji: 🚀";
        let result = Base64Ops::encode_base64(unicode);
        let decoded = STANDARD.decode(&result).unwrap();
        assert_eq!(String::from_utf8(decoded).unwrap(), unicode);
    }

    #[test]
    fn test_encode_base64_binary_chars() {
        let binary_str = "binary\x00\x01\x02chars";
        let result = Base64Ops::encode_base64(binary_str);
        let decoded = STANDARD.decode(&result).unwrap();
        assert_eq!(String::from_utf8(decoded).unwrap(), binary_str);
    }

    // ==================== Roundtrip Tests ====================

    #[test]
    fn test_roundtrip_encode_decode() {
        let original = "The quick brown fox jumps over the lazy dog";
        let encoded = Base64Ops::encode_base64(original);
        let decoded = Base64Ops::decode_base64(&encoded).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_roundtrip_with_special_chars() {
        let original = "Special: \"quotes\" and 'apostrophes' & <brackets>";
        let encoded = Base64Ops::encode_base64(original);
        let decoded = Base64Ops::decode_base64(&encoded).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_roundtrip_long_string() {
        let original = "x".repeat(10000);
        let encoded = Base64Ops::encode_base64(&original);
        let decoded = Base64Ops::decode_base64(&encoded).unwrap();
        assert_eq!(decoded, original);
    }
}
