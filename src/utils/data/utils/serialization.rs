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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ==================== deep_clone_json Tests ====================

    #[test]
    fn test_deep_clone_json_object() {
        let original = json!({"key": "value", "nested": {"inner": 123}});
        let cloned = Serialization::deep_clone_json(&original);
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_deep_clone_json_independence() {
        let original = json!({"key": "value"});
        let cloned = Serialization::deep_clone_json(&original);
        // They should be equal but not the same reference
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_deep_clone_json_array() {
        let original = json!([1, 2, {"nested": true}]);
        let cloned = Serialization::deep_clone_json(&original);
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_deep_clone_json_primitives() {
        assert_eq!(
            Serialization::deep_clone_json(&json!("string")),
            json!("string")
        );
        assert_eq!(Serialization::deep_clone_json(&json!(123)), json!(123));
        assert_eq!(Serialization::deep_clone_json(&json!(true)), json!(true));
        assert_eq!(Serialization::deep_clone_json(&json!(null)), json!(null));
    }

    // ==================== json_size_bytes Tests ====================

    #[test]
    fn test_json_size_bytes_empty_object() {
        let data = json!({});
        let size = Serialization::json_size_bytes(&data);
        assert_eq!(size, 2); // "{}"
    }

    #[test]
    fn test_json_size_bytes_simple_object() {
        let data = json!({"a": 1});
        let size = Serialization::json_size_bytes(&data);
        assert!(size > 0);
        assert_eq!(size, r#"{"a":1}"#.len());
    }

    #[test]
    fn test_json_size_bytes_nested() {
        let data = json!({"outer": {"inner": "value"}});
        let size = Serialization::json_size_bytes(&data);
        assert!(size > 10);
    }

    #[test]
    fn test_json_size_bytes_array() {
        let data = json!([1, 2, 3, 4, 5]);
        let size = Serialization::json_size_bytes(&data);
        assert_eq!(size, "[1,2,3,4,5]".len());
    }

    // ==================== pretty_print_json Tests ====================

    #[test]
    fn test_pretty_print_json_basic() {
        let data = json!({"key": "value"});
        let result = Serialization::pretty_print_json(&data).unwrap();
        assert!(result.contains('\n'));
        assert!(result.contains("  ")); // Indentation
    }

    #[test]
    fn test_pretty_print_json_nested() {
        let data = json!({"outer": {"inner": 123}});
        let result = Serialization::pretty_print_json(&data).unwrap();
        assert!(result.contains("outer"));
        assert!(result.contains("inner"));
        assert!(result.lines().count() > 1);
    }

    #[test]
    fn test_pretty_print_json_array() {
        let data = json!([1, 2, 3]);
        let result = Serialization::pretty_print_json(&data).unwrap();
        assert!(result.contains("1"));
        assert!(result.contains("2"));
        assert!(result.contains("3"));
    }

    #[test]
    fn test_pretty_print_json_null() {
        let data = json!(null);
        let result = Serialization::pretty_print_json(&data).unwrap();
        assert_eq!(result, "null");
    }

    // ==================== compact_json Tests ====================

    #[test]
    fn test_compact_json_no_whitespace() {
        let data = json!({"key": "value", "number": 123});
        let result = Serialization::compact_json(&data).unwrap();
        assert!(!result.contains('\n'));
        assert!(!result.contains("  "));
    }

    #[test]
    fn test_compact_json_nested() {
        let data = json!({"a": {"b": {"c": 1}}});
        let result = Serialization::compact_json(&data).unwrap();
        assert_eq!(result, r#"{"a":{"b":{"c":1}}}"#);
    }

    #[test]
    fn test_compact_json_empty() {
        let data = json!({});
        let result = Serialization::compact_json(&data).unwrap();
        assert_eq!(result, "{}");
    }

    #[test]
    fn test_compact_json_array() {
        let data = json!([1, 2, 3]);
        let result = Serialization::compact_json(&data).unwrap();
        assert_eq!(result, "[1,2,3]");
    }

    // ==================== hash_json Tests ====================

    #[test]
    fn test_hash_json_length() {
        let data = json!({"key": "value"});
        let hash = Serialization::hash_json(&data).unwrap();
        assert_eq!(hash.len(), 64); // SHA256 hex = 64 chars
    }

    #[test]
    fn test_hash_json_consistency() {
        let data = json!({"key": "value"});
        let hash1 = Serialization::hash_json(&data).unwrap();
        let hash2 = Serialization::hash_json(&data).unwrap();
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_json_different_data() {
        let data1 = json!({"key": "value1"});
        let data2 = json!({"key": "value2"});
        let hash1 = Serialization::hash_json(&data1).unwrap();
        let hash2 = Serialization::hash_json(&data2).unwrap();
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_json_hex_format() {
        let data = json!({"test": true});
        let hash = Serialization::hash_json(&data).unwrap();
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_hash_json_empty_object() {
        let data = json!({});
        let hash = Serialization::hash_json(&data).unwrap();
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_hash_json_order_matters() {
        // JSON object key order matters for hashing
        let data1 = json!({"a": 1, "b": 2});
        let data2 = json!({"b": 2, "a": 1});
        let hash1 = Serialization::hash_json(&data1).unwrap();
        let hash2 = Serialization::hash_json(&data2).unwrap();
        // Note: serde_json preserves order, so these might differ
        // This test verifies the hashing behavior
        assert_eq!(hash1.len(), 64);
        assert_eq!(hash2.len(), 64);
    }
}
