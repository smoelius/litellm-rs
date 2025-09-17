//! Llama 3 Partner Model Support

use crate::core::providers::vertex_ai::error::VertexAIError;

/// Llama3 transformation handler
pub struct Llama3Handler;

impl Llama3Handler {
    /// Handle Llama3 model requests
    pub async fn handle_request(
        request: serde_json::Value,
    ) -> Result<serde_json::Value, VertexAIError> {
        // Transform for Llama models on Vertex AI
        let transformed = Self::transform_llama_request(request)?;
        Ok(transformed)
    }

    /// Transform request for Llama models
    fn transform_llama_request(
        request: serde_json::Value,
    ) -> Result<serde_json::Value, VertexAIError> {
        // Llama models use chat template format
        Ok(serde_json::json!({
            "instances": [{
                "messages": request.get("messages").unwrap_or(&serde_json::Value::Null)
            }],
            "parameters": {
                "temperature": request.get("temperature").unwrap_or(&serde_json::Value::Number(serde_json::Number::from_f64(0.7).unwrap())),
                "maxOutputTokens": request.get("max_tokens").unwrap_or(&serde_json::Value::Number(2048.into())),
            }
        }))
    }
}
