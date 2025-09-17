//! Experimental Pass-through for Anthropic Models

use crate::core::providers::vertex_ai::error::VertexAIError;

/// Experimental pass-through handler
pub struct ExperimentalPassThroughHandler;

impl ExperimentalPassThroughHandler {
    /// Handle pass-through requests
    pub async fn handle_request(
        request: serde_json::Value,
    ) -> Result<serde_json::Value, VertexAIError> {
        // Pass through request with minimal transformation
        Ok(request)
    }
}
