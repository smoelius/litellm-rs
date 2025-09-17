use crate::core::providers::unified_provider::ProviderError;
use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::{Map, Value};
use sha2::Digest;
use std::collections::HashMap;
use uuid::Uuid;

pub struct DataUtils;

impl DataUtils {
    pub fn is_base64_encoded(s: &str) -> bool {
        if s.is_empty() {
            return false;
        }

        if s.len() % 4 != 0 {
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

    pub fn convert_to_dict(data: &Value) -> Result<Map<String, Value>, ProviderError> {
        match data {
            Value::Object(map) => Ok(map.clone()),
            _ => Err(ProviderError::InvalidRequest {
                provider: "unknown",
                message: "Data is not a JSON object".to_string(),
            }),
        }
    }

    pub fn convert_list_to_dict(list: &[Value]) -> Vec<Map<String, Value>> {
        list.iter()
            .filter_map(|item| {
                if let Value::Object(map) = item {
                    Some(map.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn jsonify_tools(tools: &[Value]) -> Result<Vec<Map<String, Value>>, ProviderError> {
        let mut jsonified_tools = Vec::new();

        for tool in tools {
            match tool {
                Value::Object(map) => {
                    jsonified_tools.push(map.clone());
                }
                Value::String(s) => {
                    let parsed: Value =
                        serde_json::from_str(s).map_err(|e| ProviderError::InvalidRequest {
                            provider: "unknown",
                            message: format!("Failed to parse tool JSON string: {}", e),
                        })?;

                    if let Value::Object(map) = parsed {
                        jsonified_tools.push(map);
                    } else {
                        return Err(ProviderError::InvalidRequest {
                            provider: "unknown",
                            message: "Tool JSON string must represent an object".to_string(),
                        });
                    }
                }
                _ => {
                    return Err(ProviderError::InvalidRequest {
                        provider: "unknown",
                        message: "Tool must be an object or JSON string".to_string(),
                    });
                }
            }
        }

        Ok(jsonified_tools)
    }

    pub fn cleanup_none_values(data: &mut Map<String, Value>) {
        data.retain(|_, v| !v.is_null());
    }

    pub fn deep_cleanup_none_values(data: &mut Value) {
        match data {
            Value::Object(map) => {
                map.retain(|_, v| !v.is_null());
                for (_, v) in map.iter_mut() {
                    Self::deep_cleanup_none_values(v);
                }
            }
            Value::Array(arr) => {
                for item in arr.iter_mut() {
                    Self::deep_cleanup_none_values(item);
                }
            }
            _ => {}
        }
    }

    pub fn generate_uuid() -> String {
        Uuid::new_v4().to_string()
    }

    pub fn generate_short_id() -> String {
        Uuid::new_v4().to_string()[..8].to_string()
    }

    pub fn merge_json_objects(base: &mut Value, overlay: &Value) -> Result<(), ProviderError> {
        match (base, overlay) {
            (Value::Object(base_map), Value::Object(overlay_map)) => {
                for (key, value) in overlay_map {
                    if let Some(base_value) = base_map.get_mut(key) {
                        if base_value.is_object() && value.is_object() {
                            Self::merge_json_objects(base_value, value)?;
                        } else {
                            *base_value = value.clone();
                        }
                    } else {
                        base_map.insert(key.clone(), value.clone());
                    }
                }
                Ok(())
            }
            _ => Err(ProviderError::InvalidRequest {
                provider: "unknown",
                message: "Both values must be JSON objects for merging".to_string(),
            }),
        }
    }

    pub fn extract_nested_value<'a>(data: &'a Value, path: &[&str]) -> Option<&'a Value> {
        let mut current = data;

        for segment in path {
            match current {
                Value::Object(map) => {
                    if let Some(next_value) = map.get(*segment) {
                        current = next_value;
                    } else {
                        return None;
                    }
                }
                Value::Array(arr) => {
                    if let Ok(index) = segment.parse::<usize>() {
                        if let Some(next_value) = arr.get(index) {
                            current = next_value;
                        } else {
                            return None;
                        }
                    } else {
                        return None;
                    }
                }
                _ => return None,
            }
        }

        Some(current)
    }

    pub fn set_nested_value(
        data: &mut Value,
        path: &[&str],
        value: Value,
    ) -> Result<(), ProviderError> {
        if path.is_empty() {
            return Err(ProviderError::InvalidRequest {
                provider: "unknown",
                message: "Path cannot be empty".to_string(),
            });
        }

        let mut current = data;
        let last_segment = path[path.len() - 1];

        for segment in &path[..path.len() - 1] {
            match current {
                Value::Object(map) => {
                    if !map.contains_key(*segment) {
                        map.insert(segment.to_string(), Value::Object(Map::new()));
                    }
                    current = map.get_mut(*segment).unwrap();
                }
                _ => {
                    return Err(ProviderError::InvalidRequest {
                        provider: "unknown",
                        message: "Cannot set nested value in non-object".to_string(),
                    });
                }
            }
        }

        if let Value::Object(map) = current {
            map.insert(last_segment.to_string(), value);
            Ok(())
        } else {
            Err(ProviderError::InvalidRequest {
                provider: "unknown",
                message: "Cannot set value in non-object".to_string(),
            })
        }
    }

    pub fn flatten_json(data: &Value, prefix: Option<String>) -> HashMap<String, Value> {
        let mut result = HashMap::new();
        let current_prefix = prefix.unwrap_or_default();

        match data {
            Value::Object(map) => {
                for (key, value) in map {
                    let new_key = if current_prefix.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", current_prefix, key)
                    };

                    match value {
                        Value::Object(_) | Value::Array(_) => {
                            let nested = Self::flatten_json(value, Some(new_key));
                            result.extend(nested);
                        }
                        _ => {
                            result.insert(new_key, value.clone());
                        }
                    }
                }
            }
            Value::Array(arr) => {
                for (index, value) in arr.iter().enumerate() {
                    let new_key = if current_prefix.is_empty() {
                        index.to_string()
                    } else {
                        format!("{}.{}", current_prefix, index)
                    };

                    match value {
                        Value::Object(_) | Value::Array(_) => {
                            let nested = Self::flatten_json(value, Some(new_key));
                            result.extend(nested);
                        }
                        _ => {
                            result.insert(new_key, value.clone());
                        }
                    }
                }
            }
            _ => {
                result.insert(current_prefix, data.clone());
            }
        }

        result
    }

    pub fn validate_json_schema(data: &Value, schema: &Value) -> Result<(), ProviderError> {
        match (data, schema) {
            (_, Value::Object(schema_map)) => {
                if let Some(type_value) = schema_map.get("type") {
                    if let Some(expected_type) = type_value.as_str() {
                        let data_type = match data {
                            Value::Null => "null",
                            Value::Bool(_) => "boolean",
                            Value::Number(_) => "number",
                            Value::String(_) => "string",
                            Value::Array(_) => "array",
                            Value::Object(_) => "object",
                        };

                        if data_type != expected_type {
                            return Err(ProviderError::InvalidRequest {
                                provider: "unknown",
                                message: format!(
                                    "Expected type '{}', got '{}'",
                                    expected_type, data_type
                                ),
                            });
                        }
                    }
                }

                if let (Value::Object(data_map), Some(Value::Object(properties))) =
                    (data, schema_map.get("properties"))
                {
                    for (prop_name, prop_schema) in properties {
                        if let Some(prop_data) = data_map.get(prop_name) {
                            Self::validate_json_schema(prop_data, prop_schema)?;
                        }
                    }

                    if let Some(Value::Array(required)) = schema_map.get("required") {
                        for required_prop in required {
                            if let Some(prop_name) = required_prop.as_str() {
                                if !data_map.contains_key(prop_name) {
                                    return Err(ProviderError::InvalidRequest {
                                        provider: "unknown",
                                        message: format!(
                                            "Required property '{}' is missing",
                                            prop_name
                                        ),
                                    });
                                }
                            }
                        }
                    }
                }

                Ok(())
            }
            _ => Ok(()),
        }
    }

    pub fn sanitize_for_json(input: &str) -> String {
        input
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t")
    }

    pub fn extract_json_from_string(input: &str) -> Option<Value> {
        let trimmed = input.trim();

        if let Some(start) = trimmed.find('{') {
            if let Some(end) = trimmed.rfind('}') {
                let json_str = &trimmed[start..=end];
                return serde_json::from_str(json_str).ok();
            }
        }

        if let Some(start) = trimmed.find('[') {
            if let Some(end) = trimmed.rfind(']') {
                let json_str = &trimmed[start..=end];
                return serde_json::from_str(json_str).ok();
            }
        }

        serde_json::from_str(trimmed).ok()
    }

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

    pub fn truncate_string(input: &str, max_length: usize) -> String {
        if input.len() <= max_length {
            input.to_string()
        } else {
            let mut truncated = input
                .chars()
                .take(max_length.saturating_sub(3))
                .collect::<String>();
            truncated.push_str("...");
            truncated
        }
    }

    pub fn extract_urls_from_text(text: &str) -> Vec<String> {
        let url_pattern = regex::Regex::new(
            r"https?://(?:[-\w.])+(?::[0-9]+)?(?:/(?:[\w/_.])*(?:\?(?:[\w&=%.])*)?(?:#(?:[\w.])*)?)?")
            .unwrap();

        url_pattern
            .find_iter(text)
            .map(|m| m.as_str().to_string())
            .collect()
    }

    pub fn clean_whitespace(text: &str) -> String {
        text.split_whitespace().collect::<Vec<&str>>().join(" ")
    }

    pub fn word_count(text: &str) -> usize {
        text.split_whitespace().count()
    }

    pub fn character_count(text: &str) -> usize {
        text.chars().count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_base64_operations() {
        let original = "Hello, World!";
        let encoded = DataUtils::encode_base64(original);
        assert!(DataUtils::is_base64_encoded(&encoded));

        let decoded = DataUtils::decode_base64(&encoded).unwrap();
        assert_eq!(decoded, original);

        assert!(!DataUtils::is_base64_encoded("not base64!"));
    }

    #[test]
    fn test_json_operations() {
        let data = json!({
            "name": "test",
            "value": 42,
            "nested": {
                "inner": "data"
            }
        });

        let dict = DataUtils::convert_to_dict(&data).unwrap();
        assert!(dict.contains_key("name"));
        assert!(dict.contains_key("nested"));
    }

    #[test]
    fn test_uuid_generation() {
        let uuid1 = DataUtils::generate_uuid();
        let uuid2 = DataUtils::generate_uuid();
        assert_ne!(uuid1, uuid2);
        assert!(Uuid::parse_str(&uuid1).is_ok());

        let short_id = DataUtils::generate_short_id();
        assert_eq!(short_id.len(), 8);
    }

    #[test]
    fn test_json_merging() {
        let mut base = json!({
            "a": 1,
            "b": {
                "c": 2
            }
        });

        let overlay = json!({
            "b": {
                "d": 3
            },
            "e": 4
        });

        DataUtils::merge_json_objects(&mut base, &overlay).unwrap();

        assert_eq!(base["a"], json!(1));
        assert_eq!(base["b"]["c"], json!(2));
        assert_eq!(base["b"]["d"], json!(3));
        assert_eq!(base["e"], json!(4));
    }

    #[test]
    fn test_nested_value_extraction() {
        let data = json!({
            "level1": {
                "level2": {
                    "value": "found"
                }
            },
            "array": [1, 2, {"key": "value"}]
        });

        let value = DataUtils::extract_nested_value(&data, &["level1", "level2", "value"]);
        assert_eq!(value, Some(&json!("found")));

        let array_value = DataUtils::extract_nested_value(&data, &["array", "2", "key"]);
        assert_eq!(array_value, Some(&json!("value")));

        let missing = DataUtils::extract_nested_value(&data, &["missing", "path"]);
        assert_eq!(missing, None);
    }

    #[test]
    fn test_json_flattening() {
        let data = json!({
            "a": 1,
            "b": {
                "c": 2,
                "d": {
                    "e": 3
                }
            },
            "f": [1, 2, 3]
        });

        let flattened = DataUtils::flatten_json(&data, None);
        assert_eq!(flattened.get("a"), Some(&json!(1)));
        assert_eq!(flattened.get("b.c"), Some(&json!(2)));
        assert_eq!(flattened.get("b.d.e"), Some(&json!(3)));
        assert_eq!(flattened.get("f.0"), Some(&json!(1)));
    }

    #[test]
    fn test_json_schema_validation() {
        let data = json!({
            "name": "test",
            "age": 25
        });

        let schema = json!({
            "type": "object",
            "properties": {
                "name": {"type": "string"},
                "age": {"type": "number"}
            },
            "required": ["name"]
        });

        assert!(DataUtils::validate_json_schema(&data, &schema).is_ok());

        let invalid_data = json!({
            "age": "not a number"
        });

        assert!(DataUtils::validate_json_schema(&invalid_data, &schema).is_err());
    }

    #[test]
    fn test_string_utilities() {
        assert_eq!(
            DataUtils::truncate_string("Hello, World!", 10),
            "Hello, ..."
        );
        assert_eq!(DataUtils::truncate_string("Short", 10), "Short");

        assert_eq!(
            DataUtils::clean_whitespace("  Hello   world  "),
            "Hello world"
        );

        assert_eq!(DataUtils::word_count("Hello world test"), 3);
        assert_eq!(DataUtils::character_count("Hello 🌍"), 7);
    }

    #[test]
    fn test_url_extraction() {
        let text = "Check out https://example.com and http://test.org/path?query=1";
        let urls = DataUtils::extract_urls_from_text(text);
        assert_eq!(urls.len(), 2);
        assert!(urls.contains(&"https://example.com".to_string()));
        assert!(urls.contains(&"http://test.org/path?query=1".to_string()));
    }

    #[test]
    fn test_json_extraction_from_string() {
        let text = "Here is some JSON: {\"key\": \"value\"} and more text";
        let extracted = DataUtils::extract_json_from_string(text);
        assert_eq!(extracted, Some(json!({"key": "value"})));

        let no_json = "This has no JSON content";
        let no_extracted = DataUtils::extract_json_from_string(no_json);
        assert_eq!(no_extracted, None);
    }

    #[test]
    fn test_json_utilities() {
        let data = json!({"test": "value"});

        let pretty = DataUtils::pretty_print_json(&data).unwrap();
        assert!(pretty.contains("  "));

        let compact = DataUtils::compact_json(&data).unwrap();
        assert!(!compact.contains("  "));

        let hash = DataUtils::hash_json(&data).unwrap();
        assert_eq!(hash.len(), 64); // SHA-256 hex string length

        let size = DataUtils::json_size_bytes(&data);
        assert!(size > 0);
    }
}
