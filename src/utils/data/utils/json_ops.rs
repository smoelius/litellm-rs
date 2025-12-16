use crate::core::providers::unified_provider::ProviderError;
use serde_json::{Map, Value};
use std::collections::HashMap;

pub struct JsonOps;

impl JsonOps {
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
}
