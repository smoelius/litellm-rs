//! Anthropic Partner Model Support

pub mod experimental_pass_through;

use crate::core::providers::vertex_ai::error::VertexAIError;

/// Anthropic transformation handler
pub struct AnthropicHandler;

impl AnthropicHandler {
    /// Handle Anthropic model requests
    pub async fn handle_request(
        request: serde_json::Value,
    ) -> Result<serde_json::Value, VertexAIError> {
        // Transform for Claude models on Vertex AI
        let transformed = Self::transform_claude_request(request)?;
        Ok(transformed)
    }

    /// Transform request for Claude models
    fn transform_claude_request(
        request: serde_json::Value,
    ) -> Result<serde_json::Value, VertexAIError> {
        // Claude via Vertex AI uses specific format
        Ok(serde_json::json!({
            "anthropic_version": "vertex-2023-10-16",
            "messages": request.get("messages").unwrap_or(&serde_json::Value::Null),
            "max_tokens": request.get("max_tokens").unwrap_or(&serde_json::Value::Number(4096.into())),
        }))
    }
}
