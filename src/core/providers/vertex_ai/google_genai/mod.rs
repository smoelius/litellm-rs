//! Google Generative AI Module
//!
//! Integration with Google AI Studio and Generative AI API

use super::error::VertexAIError;

/// Google AI transformation utilities
pub struct GoogleGenAITransformation;

impl GoogleGenAITransformation {
    /// Transform Vertex AI request to Google AI Studio format
    pub fn transform_to_genai_format(
        vertex_request: serde_json::Value,
    ) -> Result<serde_json::Value, VertexAIError> {
        // Google AI Studio uses slightly different format than Vertex AI
        let mut genai_request = vertex_request.clone();

        // Remove Vertex-specific fields
        if let Some(obj) = genai_request.as_object_mut() {
            obj.remove("instances");
            obj.remove("parameters");
        }

        Ok(genai_request)
    }

    /// Transform Google AI Studio response to Vertex AI format
    pub fn transform_from_genai_format(
        genai_response: serde_json::Value,
    ) -> Result<serde_json::Value, VertexAIError> {
        // Add Vertex AI response wrapper
        Ok(serde_json::json!({
            "predictions": [genai_response],
            "metadata": {}
        }))
    }
}
