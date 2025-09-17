//! Vertex AI Embeddings Module
//!
//! Legacy embedding models and compatibility

use super::error::VertexAIError;
use serde::{Deserialize, Serialize};

/// Legacy embedding types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyEmbeddingTypes {
    pub model: String,
    pub dimensions: usize,
}

/// Legacy embedding handler
pub struct VertexEmbeddingHandler;

impl VertexEmbeddingHandler {
    /// Handle legacy embedding requests
    pub async fn handle_legacy_embedding(
        model: &str,
        _text: &str,
    ) -> Result<Vec<f32>, VertexAIError> {
        // Handle legacy models like textembedding-gecko
        match model {
            "textembedding-gecko" | "textembedding-gecko@003" => {
                // TODO: Implement gecko embedding
                Ok(vec![0.0; 768])
            }
            "textembedding-gecko-multilingual" => {
                // TODO: Implement multilingual gecko
                Ok(vec![0.0; 768])
            }
            _ => Err(VertexAIError::UnsupportedModel(model.to_string())),
        }
    }
}
