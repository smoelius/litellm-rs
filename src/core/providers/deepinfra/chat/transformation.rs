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
}
