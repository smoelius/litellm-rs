//! DeepInfra Rerank Transformation
//!
//! Transforms between Cohere's rerank format and DeepInfra's rerank format

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankRequest {
    pub query: Option<String>,
    pub queries: Option<Vec<String>>,
    pub documents: Vec<Value>,
    pub top_n: Option<usize>,
    pub return_documents: Option<bool>,
    pub max_chunks_per_doc: Option<usize>,
    pub max_tokens_per_doc: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankResult {
    pub index: usize,
    pub relevance_score: f64,
    pub document: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankResponse {
    pub id: String,
    pub results: Vec<RerankResult>,
    pub meta: RerankMeta,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<RerankUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankMeta {
    pub tokens: RerankTokens,
    pub billed_units: RerankBilledUnits,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankTokens {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankBilledUnits {
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankUsage {
    pub prompt_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepInfraRerankResponse {
    pub scores: Vec<f64>,
    pub input_tokens: u32,
    pub request_id: Option<String>,
    pub inference_status: Option<InferenceStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceStatus {
    pub status: String,
    pub runtime_ms: u64,
    pub cost: f64,
    pub tokens_generated: u32,
    pub tokens_input: u32,
}

pub struct DeepInfraRerankTransformation;

impl DeepInfraRerankTransformation {
    pub fn new() -> Self {
        Self
    }

    /// Get the complete URL for DeepInfra rerank endpoint
    pub fn get_complete_url(&self, api_base: Option<&str>, model: &str) -> Result<String, String> {
        let base = api_base.ok_or_else(|| {
            "DeepInfra API base is required. Set via DEEPINFRA_API_BASE env var.".to_string()
        })?;

        // Remove 'openai' from the base if present
        let api_base_clean = if base.contains("openai") {
            base.replace("openai", "")
        } else {
            base.to_string()
        };

        // Remove trailing slashes for consistency, then add one
        let api_base_clean = api_base_clean.trim_end_matches('/');

        // Compose the full endpoint
        Ok(format!("{}/inference/{}", api_base_clean, model))
    }

    /// Create authorization headers
    pub fn create_headers(&self, api_key: Option<&str>) -> Result<HashMap<String, String>, String> {
        let api_key = api_key.ok_or_else(|| {
            "DeepInfra API key is required. Set via DEEPINFRA_API_KEY env var.".to_string()
        })?;

        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), format!("Bearer {}", api_key));
        headers.insert("Accept".to_string(), "application/json".to_string());
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        Ok(headers)
    }

    /// Map Cohere rerank parameters to DeepInfra format
    pub fn map_cohere_rerank_params(&self, request: &RerankRequest) -> Value {
        let mut params = json!({});

        // DeepInfra requires queries to be same length as documents
        if let Some(query) = &request.query {
            let queries = vec![query.clone(); request.documents.len()];
            params["queries"] = json!(queries);
        } else if let Some(queries) = &request.queries {
            params["queries"] = json!(queries);
        }

        // Add documents
        params["documents"] = json!(request.documents);

        // Add optional parameters
        if let Some(top_n) = request.top_n {
            params["top_n"] = json!(top_n);
        }

        if let Some(return_docs) = request.return_documents {
            params["return_documents"] = json!(return_docs);
        }

        params
    }

    /// Transform request for DeepInfra
    pub fn transform_rerank_request(&self, request: Value) -> Result<Value, String> {
        // Parse the request
        let rerank_request: RerankRequest = serde_json::from_value(request)
            .map_err(|e| format!("Failed to parse rerank request: {}", e))?;

        // Map to DeepInfra format
        let transformed = self.map_cohere_rerank_params(&rerank_request);

        Ok(transformed)
    }

    /// Transform DeepInfra response to standard format
    pub fn transform_rerank_response(&self, response: Value) -> Result<RerankResponse, String> {
        // Try to parse as DeepInfra response
        let deepinfra_response: DeepInfraRerankResponse = serde_json::from_value(response.clone())
            .map_err(|e| format!("Failed to parse DeepInfra response: {}", e))?;

        // Create results from scores
        let results: Vec<RerankResult> = deepinfra_response
            .scores
            .into_iter()
            .enumerate()
            .map(|(index, score)| RerankResult {
                index,
                relevance_score: score,
                document: None, // DeepInfra doesn't return documents in response
            })
            .collect();

        // Create metadata
        let tokens = RerankTokens {
            input_tokens: deepinfra_response.input_tokens,
            output_tokens: 0, // DeepInfra doesn't provide output tokens for rerank
        };

        let billed_units = RerankBilledUnits {
            total_tokens: deepinfra_response.input_tokens,
        };

        let meta = RerankMeta {
            tokens,
            billed_units,
        };

        // Create usage if we have inference status
        let usage = deepinfra_response
            .inference_status
            .map(|status| RerankUsage {
                prompt_tokens: status.tokens_input,
                total_tokens: status.tokens_input,
            });

        // Create final response
        let rerank_response = RerankResponse {
            id: deepinfra_response
                .request_id
                .unwrap_or_else(|| Uuid::new_v4().to_string()),
            results,
            meta,
            usage,
        };

        Ok(rerank_response)
    }

    /// Get supported rerank parameters
    pub fn get_supported_cohere_rerank_params(&self) -> Vec<&'static str> {
        vec!["query", "documents", "queries", "top_n", "return_documents"]
    }

    /// Parse error response from DeepInfra
    pub fn parse_error(&self, error_response: Value) -> String {
        // Try to extract a more specific error message
        if let Some(obj) = error_response.as_object() {
            // Check for {"detail": {"error": "..."}}
            if let Some(detail) = obj.get("detail") {
                if let Some(detail_obj) = detail.as_object() {
                    if let Some(error) = detail_obj.get("error") {
                        if let Some(error_str) = error.as_str() {
                            return error_str.to_string();
                        }
                    }
                } else if let Some(detail_str) = detail.as_str() {
                    return detail_str.to_string();
                }
            }

            // Check for {"error": "..."}
            if let Some(error) = obj.get("error") {
                if let Some(error_str) = error.as_str() {
                    return error_str.to_string();
                }
            }
        }

        // Fallback to stringifying the whole response
        serde_json::to_string(&error_response).unwrap_or_else(|_| "Unknown error".to_string())
    }
}

impl Default for DeepInfraRerankTransformation {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_complete_url() {
        let transformation = DeepInfraRerankTransformation::new();

        let url = transformation
            .get_complete_url(Some("https://api.deepinfra.com/v1/openai"), "model-name")
            .unwrap();

        assert_eq!(url, "https://api.deepinfra.com/v1/inference/model-name");
    }

    #[test]
    fn test_map_cohere_rerank_params() {
        let transformation = DeepInfraRerankTransformation::new();

        let request = RerankRequest {
            query: Some("test query".to_string()),
            queries: None,
            documents: vec![json!({"text": "doc1"}), json!({"text": "doc2"})],
            top_n: Some(5),
            return_documents: Some(true),
            max_chunks_per_doc: None,
            max_tokens_per_doc: None,
        };

        let result = transformation.map_cohere_rerank_params(&request);

        assert!(result["queries"].is_array());
        assert_eq!(result["queries"].as_array().unwrap().len(), 2);
        assert_eq!(result["top_n"], json!(5));
        assert_eq!(result["return_documents"], json!(true));
    }

    #[test]
    fn test_transform_rerank_response() {
        let transformation = DeepInfraRerankTransformation::new();

        let deepinfra_response = json!({
            "scores": [0.9, 0.7, 0.5],
            "input_tokens": 100,
            "request_id": "test-id",
            "inference_status": {
                "status": "success",
                "runtime_ms": 150,
                "cost": 0.001,
                "tokens_generated": 0,
                "tokens_input": 100
            }
        });

        let result = transformation
            .transform_rerank_response(deepinfra_response)
            .unwrap();

        assert_eq!(result.id, "test-id");
        assert_eq!(result.results.len(), 3);
        assert_eq!(result.results[0].relevance_score, 0.9);
        assert_eq!(result.results[0].index, 0);
        assert_eq!(result.meta.tokens.input_tokens, 100);
        assert!(result.usage.is_some());
    }
}
