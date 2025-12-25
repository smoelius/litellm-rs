//! DeepInfra Chat Transformation
//!
//! OpenAI-compatible transformations for DeepInfra's chat API

use serde_json::{Value, json};
use std::collections::HashMap;

pub struct DeepInfraChatTransformation;

impl DeepInfraChatTransformation {
    pub fn new() -> Self {
        Self
    }

    /// Get supported OpenAI parameters for DeepInfra
    pub fn get_supported_openai_params(&self, _model: &str) -> Vec<&'static str> {
        vec![
            "stream",
            "frequency_penalty",
            "function_call",
            "functions",
            "logit_bias",
            "max_tokens",
            "max_completion_tokens",
            "n",
            "presence_penalty",
            "stop",
            "temperature",
            "top_p",
            "response_format",
            "tools",
            "tool_choice",
        ]
    }

    /// Map OpenAI parameters to DeepInfra format
    pub fn map_openai_params(
        &self,
        params: HashMap<String, Value>,
        model: &str,
    ) -> Result<HashMap<String, Value>, String> {
        let mut mapped = HashMap::new();
        let supported_params = self.get_supported_openai_params(model);

        for (key, value) in params {
            match key.as_str() {
                // Handle temperature edge case for Mistral model
                "temperature" => {
                    let temp_value = if model.contains("mistralai/Mistral-7B-Instruct-v0.1")
                        && value.as_f64() == Some(0.0)
                    {
                        // This model doesn't support temperature == 0
                        json!(0.001) // Close to 0
                    } else {
                        value
                    };
                    mapped.insert("temperature".to_string(), temp_value);
                }

                // Handle tool_choice restrictions
                "tool_choice" => {
                    if let Some(choice_str) = value.as_str() {
                        if choice_str != "auto" && choice_str != "none" {
                            // DeepInfra only supports "auto" and "none"
                            // Drop unsupported values silently
                            continue;
                        }
                    }
                    mapped.insert(key, value);
                }

                // Convert max_completion_tokens to max_tokens
                "max_completion_tokens" => {
                    mapped.insert("max_tokens".to_string(), value);
                }

                // Pass through supported parameters
                _ => {
                    if supported_params.contains(&key.as_str()) {
                        mapped.insert(key, value);
                    }
                }
            }
        }

        Ok(mapped)
    }

    /// Transform request for DeepInfra
    pub fn transform_request(&self, mut request: Value, model: &str) -> Result<Value, String> {
        if let Some(obj) = request.as_object_mut() {
            // Extract parameters for mapping
            let mut params_to_map = HashMap::new();

            // Collect all parameters except messages and model
            for (key, value) in obj.iter() {
                if key != "messages" && key != "model" {
                    params_to_map.insert(key.clone(), value.clone());
                }
            }

            // Map parameters
            let mapped_params = self.map_openai_params(params_to_map, model)?;

            // Clear object and rebuild with mapped parameters
            let messages = obj.get("messages").cloned();
            let model_value = obj.get("model").cloned();

            obj.clear();

            // Re-add messages and model first
            if let Some(m) = messages {
                obj.insert("messages".to_string(), m);
            }
            if let Some(m) = model_value {
                obj.insert("model".to_string(), m);
            }

            // Add mapped parameters
            for (key, value) in mapped_params {
                obj.insert(key, value);
            }
        }

        Ok(request)
    }

    /// Transform response from DeepInfra
    pub fn transform_response(&self, response: Value) -> Result<Value, String> {
        // DeepInfra responses are OpenAI-compatible
        Ok(response)
    }

    /// Get the complete API URL
    pub fn get_complete_url(&self, api_base: Option<&str>) -> String {
        api_base
            .unwrap_or("https://api.deepinfra.com/v1/openai")
            .to_string()
    }

    /// Validate that required fields are present
    pub fn validate_request(&self, request: &Value) -> Result<(), String> {
        let obj = request
            .as_object()
            .ok_or_else(|| "Request must be a JSON object".to_string())?;

        // Check for required fields
        if !obj.contains_key("messages") {
            return Err("Missing required field: messages".to_string());
        }

        if !obj.contains_key("model") {
            return Err("Missing required field: model".to_string());
        }

        Ok(())
    }
}

impl Default for DeepInfraChatTransformation {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deepinfra_chat_transformation_new() {
        let transformation = DeepInfraChatTransformation::new();
        let params = transformation.get_supported_openai_params("any-model");
        assert!(params.contains(&"temperature"));
        assert!(params.contains(&"max_tokens"));
        assert!(params.contains(&"stream"));
    }

    #[test]
    fn test_deepinfra_chat_transformation_default() {
        let transformation = DeepInfraChatTransformation::default();
        let params = transformation.get_supported_openai_params("any-model");
        assert!(!params.is_empty());
    }

    #[test]
    fn test_get_supported_openai_params() {
        let transformation = DeepInfraChatTransformation::new();
        let params = transformation.get_supported_openai_params("meta-llama/Llama-2-70b");

        assert!(params.contains(&"stream"));
        assert!(params.contains(&"frequency_penalty"));
        assert!(params.contains(&"function_call"));
        assert!(params.contains(&"functions"));
        assert!(params.contains(&"logit_bias"));
        assert!(params.contains(&"max_tokens"));
        assert!(params.contains(&"max_completion_tokens"));
        assert!(params.contains(&"n"));
        assert!(params.contains(&"presence_penalty"));
        assert!(params.contains(&"stop"));
        assert!(params.contains(&"temperature"));
        assert!(params.contains(&"top_p"));
        assert!(params.contains(&"response_format"));
        assert!(params.contains(&"tools"));
        assert!(params.contains(&"tool_choice"));
    }

    #[test]
    fn test_map_openai_params_temperature_fix() {
        let transformation = DeepInfraChatTransformation::new();

        let mut params = HashMap::new();
        params.insert("temperature".to_string(), json!(0.0));
        params.insert("max_tokens".to_string(), json!(100));

        let mapped = transformation
            .map_openai_params(params, "mistralai/Mistral-7B-Instruct-v0.1")
            .unwrap();

        // Temperature should be adjusted for this model
        assert_eq!(mapped.get("temperature").unwrap().as_f64().unwrap(), 0.001);
        assert_eq!(mapped.get("max_tokens").unwrap().as_i64().unwrap(), 100);
    }

    #[test]
    fn test_map_openai_params_temperature_non_mistral() {
        let transformation = DeepInfraChatTransformation::new();

        let mut params = HashMap::new();
        params.insert("temperature".to_string(), json!(0.0));

        let mapped = transformation
            .map_openai_params(params, "meta-llama/Llama-2-70b")
            .unwrap();

        // Temperature should NOT be adjusted for other models
        assert_eq!(mapped.get("temperature").unwrap().as_f64().unwrap(), 0.0);
    }

    #[test]
    fn test_map_openai_params_tool_choice() {
        let transformation = DeepInfraChatTransformation::new();

        let mut params = HashMap::new();
        params.insert("tool_choice".to_string(), json!("auto"));

        let mapped = transformation
            .map_openai_params(params.clone(), "any-model")
            .unwrap();
        assert!(mapped.contains_key("tool_choice"));

        // Test unsupported tool_choice value
        let mut params2 = HashMap::new();
        params2.insert("tool_choice".to_string(), json!("specific_function"));

        let mapped2 = transformation
            .map_openai_params(params2, "any-model")
            .unwrap();
        assert!(!mapped2.contains_key("tool_choice")); // Should be dropped
    }

    #[test]
    fn test_map_openai_params_tool_choice_none() {
        let transformation = DeepInfraChatTransformation::new();

        let mut params = HashMap::new();
        params.insert("tool_choice".to_string(), json!("none"));

        let mapped = transformation
            .map_openai_params(params, "any-model")
            .unwrap();
        assert!(mapped.contains_key("tool_choice"));
        assert_eq!(mapped.get("tool_choice").unwrap(), "none");
    }

    #[test]
    fn test_max_completion_tokens_mapping() {
        let transformation = DeepInfraChatTransformation::new();

        let mut params = HashMap::new();
        params.insert("max_completion_tokens".to_string(), json!(500));

        let mapped = transformation
            .map_openai_params(params, "any-model")
            .unwrap();

        assert!(!mapped.contains_key("max_completion_tokens"));
        assert_eq!(mapped.get("max_tokens").unwrap().as_i64().unwrap(), 500);
    }

    #[test]
    fn test_map_openai_params_unsupported_params_dropped() {
        let transformation = DeepInfraChatTransformation::new();

        let mut params = HashMap::new();
        params.insert("unsupported_param".to_string(), json!("value"));
        params.insert("temperature".to_string(), json!(0.5));

        let mapped = transformation
            .map_openai_params(params, "any-model")
            .unwrap();

        assert!(!mapped.contains_key("unsupported_param"));
        assert!(mapped.contains_key("temperature"));
    }

    #[test]
    fn test_transform_request_basic() {
        let transformation = DeepInfraChatTransformation::new();
        let request = json!({
            "model": "meta-llama/Llama-2-70b",
            "messages": [{"role": "user", "content": "Hello"}],
            "temperature": 0.5
        });

        let result = transformation.transform_request(request, "meta-llama/Llama-2-70b");
        assert!(result.is_ok());
        let value = result.unwrap();
        assert!(value["messages"].is_array());
        assert_eq!(value["model"], "meta-llama/Llama-2-70b");
    }

    #[test]
    fn test_transform_response() {
        let transformation = DeepInfraChatTransformation::new();
        let response = json!({
            "id": "chatcmpl-123",
            "choices": [{"message": {"content": "Hello!"}}],
            "usage": {"prompt_tokens": 10, "completion_tokens": 5}
        });

        let result = transformation.transform_response(response.clone());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), response);
    }

    #[test]
    fn test_get_complete_url_default() {
        let transformation = DeepInfraChatTransformation::new();
        let url = transformation.get_complete_url(None);
        assert_eq!(url, "https://api.deepinfra.com/v1/openai");
    }

    #[test]
    fn test_get_complete_url_custom() {
        let transformation = DeepInfraChatTransformation::new();
        let url = transformation.get_complete_url(Some("https://custom.api.com/v1"));
        assert_eq!(url, "https://custom.api.com/v1");
    }

    #[test]
    fn test_validate_request_valid() {
        let transformation = DeepInfraChatTransformation::new();
        let request = json!({
            "model": "meta-llama/Llama-2-70b",
            "messages": [{"role": "user", "content": "Hello"}]
        });

        let result = transformation.validate_request(&request);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_request_missing_messages() {
        let transformation = DeepInfraChatTransformation::new();
        let request = json!({
            "model": "meta-llama/Llama-2-70b"
        });

        let result = transformation.validate_request(&request);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("messages"));
    }

    #[test]
    fn test_validate_request_missing_model() {
        let transformation = DeepInfraChatTransformation::new();
        let request = json!({
            "messages": [{"role": "user", "content": "Hello"}]
        });

        let result = transformation.validate_request(&request);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("model"));
    }

    #[test]
    fn test_validate_request_not_object() {
        let transformation = DeepInfraChatTransformation::new();
        let request = json!("invalid");

        let result = transformation.validate_request(&request);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("JSON object"));
    }
}
