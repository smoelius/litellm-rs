mod base64_ops;
mod json_ops;
mod serialization;
mod string_ops;
mod uuid_ops;

#[cfg(test)]
mod tests;

use crate::core::providers::unified_provider::ProviderError;
use base64_ops::Base64Ops;
use json_ops::JsonOps;
use serde_json::{Map, Value};
use serialization::Serialization;
use std::collections::HashMap;
use string_ops::StringOps;
use uuid_ops::UuidOps;

pub struct DataUtils;

impl DataUtils {
    // Base64 operations
    pub fn is_base64_encoded(s: &str) -> bool {
        Base64Ops::is_base64_encoded(s)
    }

    pub fn get_base64_string(s: &str) -> String {
        Base64Ops::get_base64_string(s)
    }

    pub fn decode_base64(s: &str) -> Result<String, ProviderError> {
        Base64Ops::decode_base64(s)
    }

    pub fn encode_base64(s: &str) -> String {
        Base64Ops::encode_base64(s)
    }

    // JSON operations
    pub fn convert_to_dict(data: &Value) -> Result<Map<String, Value>, ProviderError> {
        JsonOps::convert_to_dict(data)
    }

    pub fn convert_list_to_dict(list: &[Value]) -> Vec<Map<String, Value>> {
        JsonOps::convert_list_to_dict(list)
    }

    pub fn jsonify_tools(tools: &[Value]) -> Result<Vec<Map<String, Value>>, ProviderError> {
        JsonOps::jsonify_tools(tools)
    }

    pub fn cleanup_none_values(data: &mut Map<String, Value>) {
        JsonOps::cleanup_none_values(data)
    }

    pub fn deep_cleanup_none_values(data: &mut Value) {
        JsonOps::deep_cleanup_none_values(data)
    }

    pub fn merge_json_objects(base: &mut Value, overlay: &Value) -> Result<(), ProviderError> {
        JsonOps::merge_json_objects(base, overlay)
    }

    pub fn extract_nested_value<'a>(data: &'a Value, path: &[&str]) -> Option<&'a Value> {
        JsonOps::extract_nested_value(data, path)
    }

    pub fn set_nested_value(
        data: &mut Value,
        path: &[&str],
        value: Value,
    ) -> Result<(), ProviderError> {
        JsonOps::set_nested_value(data, path, value)
    }

    pub fn flatten_json(data: &Value, prefix: Option<String>) -> HashMap<String, Value> {
        JsonOps::flatten_json(data, prefix)
    }

    pub fn validate_json_schema(data: &Value, schema: &Value) -> Result<(), ProviderError> {
        JsonOps::validate_json_schema(data, schema)
    }

    // UUID operations
    pub fn generate_uuid() -> String {
        UuidOps::generate_uuid()
    }

    pub fn generate_short_id() -> String {
        UuidOps::generate_short_id()
    }

    // String operations
    pub fn sanitize_for_json(input: &str) -> String {
        StringOps::sanitize_for_json(input)
    }

    pub fn extract_json_from_string(input: &str) -> Option<Value> {
        StringOps::extract_json_from_string(input)
    }

    pub fn truncate_string(input: &str, max_length: usize) -> String {
        StringOps::truncate_string(input, max_length)
    }

    pub fn extract_urls_from_text(text: &str) -> Vec<String> {
        StringOps::extract_urls_from_text(text)
    }

    pub fn clean_whitespace(text: &str) -> String {
        StringOps::clean_whitespace(text)
    }

    pub fn word_count(text: &str) -> usize {
        StringOps::word_count(text)
    }

    pub fn character_count(text: &str) -> usize {
        StringOps::character_count(text)
    }

    // Serialization operations
    pub fn deep_clone_json(data: &Value) -> Value {
        Serialization::deep_clone_json(data)
    }

    pub fn json_size_bytes(data: &Value) -> usize {
        Serialization::json_size_bytes(data)
    }

    pub fn pretty_print_json(data: &Value) -> Result<String, ProviderError> {
        Serialization::pretty_print_json(data)
    }

    pub fn compact_json(data: &Value) -> Result<String, ProviderError> {
        Serialization::compact_json(data)
    }

    pub fn hash_json(data: &Value) -> Result<String, ProviderError> {
        Serialization::hash_json(data)
    }
}
