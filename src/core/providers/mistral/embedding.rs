//! Embedding functionality for Mistral provider

use serde_json::{Value, json};
use tracing::{debug, info};

use crate::core::providers::mistral::{MistralConfig, MistralError};

/// Mistral embedding handler
#[derive(Debug, Clone)]
pub struct MistralEmbeddingHandler {
    config: MistralConfig,
}

impl MistralEmbeddingHandler {
    /// Create a new embedding handler
    pub fn new(config: MistralConfig) -> Result<Self, MistralError> {
        Ok(Self { config })
    }

    /// Transform an embedding request to Mistral format
    pub fn transform_request(
        &self,
        request: crate::core::types::requests::EmbeddingRequest,
    ) -> Result<Value, MistralError> {
        let transformed = json!({
            "model": "mistral-embed", // Always use mistral-embed for embeddings
            "input": request.input,
            "encoding_format": request.encoding_format.unwrap_or_else(|| "float".to_string()),
        });

        debug!("Transformed Mistral embedding request");
        Ok(transformed)
    }

    /// Transform a Mistral embedding response to standard format
    pub fn transform_response(
        &self,
        response: Value,
    ) -> Result<crate::core::types::responses::EmbeddingResponse, MistralError> {
        use crate::core::types::responses::{EmbeddingData, Usage};

        let object = response
            .get("object")
            .and_then(|v| v.as_str())
            .unwrap_or("list")
            .to_string();

        let model = response
            .get("model")
            .and_then(|v| v.as_str())
            .unwrap_or("mistral-embed")
            .to_string();

        // Parse embeddings data
        let data: Vec<EmbeddingData> = response
            .get("data")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| {
                        let index = item.get("index")?.as_i64()? as u32;
                        let embedding = item
                            .get("embedding")?
                            .as_array()?
                            .iter()
                            .filter_map(|v| v.as_f64().map(|f| f as f32))
                            .collect();

                        Some(EmbeddingData {
                            object: "embedding".to_string(),
                            index,
                            embedding,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Parse usage
        let usage = response.get("usage").map(|u| {
            Usage {
                prompt_tokens: u.get("prompt_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                completion_tokens: 0, // Not applicable for embeddings
                total_tokens: u.get("total_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                prompt_tokens_details: None,
                completion_tokens_details: None,
            }
        });

        info!("Mistral embedding response transformed successfully");

        Ok(crate::core::types::responses::EmbeddingResponse {
            object,
            data: data.clone(),
            model,
            usage,
            embeddings: Some(data),
        })
    }
}
