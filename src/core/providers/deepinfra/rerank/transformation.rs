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
    fn test_deepinfra_rerank_transformation_new() {
        let transformation = DeepInfraRerankTransformation::new();
        let params = transformation.get_supported_cohere_rerank_params();
        assert!(params.contains(&"query"));
        assert!(params.contains(&"documents"));
    }

    #[test]
    fn test_deepinfra_rerank_transformation_default() {
        let transformation = DeepInfraRerankTransformation::default();
        let params = transformation.get_supported_cohere_rerank_params();
        assert!(!params.is_empty());
    }

    #[test]
    fn test_get_supported_cohere_rerank_params() {
        let transformation = DeepInfraRerankTransformation::new();
        let params = transformation.get_supported_cohere_rerank_params();

        assert!(params.contains(&"query"));
        assert!(params.contains(&"documents"));
        assert!(params.contains(&"queries"));
        assert!(params.contains(&"top_n"));
        assert!(params.contains(&"return_documents"));
        assert_eq!(params.len(), 5);
    }

    #[test]
    fn test_get_complete_url() {
        let transformation = DeepInfraRerankTransformation::new();

        let url = transformation
            .get_complete_url(Some("https://api.deepinfra.com/v1/openai"), "model-name")
            .unwrap();

        assert_eq!(url, "https://api.deepinfra.com/v1/inference/model-name");
    }

    #[test]
    fn test_get_complete_url_no_openai() {
        let transformation = DeepInfraRerankTransformation::new();

        let url = transformation
            .get_complete_url(Some("https://api.deepinfra.com/v1/"), "bge-reranker")
            .unwrap();

        assert_eq!(url, "https://api.deepinfra.com/v1/inference/bge-reranker");
    }

    #[test]
    fn test_get_complete_url_missing_base() {
        let transformation = DeepInfraRerankTransformation::new();
        let result = transformation.get_complete_url(None, "model-name");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("API base is required"));
    }

    #[test]
    fn test_create_headers() {
        let transformation = DeepInfraRerankTransformation::new();
        let headers = transformation.create_headers(Some("test-api-key")).unwrap();

        assert_eq!(headers.get("Authorization").unwrap(), "Bearer test-api-key");
        assert_eq!(headers.get("Accept").unwrap(), "application/json");
        assert_eq!(headers.get("Content-Type").unwrap(), "application/json");
    }

    #[test]
    fn test_create_headers_missing_key() {
        let transformation = DeepInfraRerankTransformation::new();
        let result = transformation.create_headers(None);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("API key is required"));
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
    fn test_map_cohere_rerank_params_with_queries() {
        let transformation = DeepInfraRerankTransformation::new();

        let request = RerankRequest {
            query: None,
            queries: Some(vec!["query1".to_string(), "query2".to_string()]),
            documents: vec![json!("doc1"), json!("doc2")],
            top_n: None,
            return_documents: None,
            max_chunks_per_doc: None,
            max_tokens_per_doc: None,
        };

        let result = transformation.map_cohere_rerank_params(&request);

        assert!(result["queries"].is_array());
        assert_eq!(result["queries"].as_array().unwrap().len(), 2);
        assert_eq!(result["queries"][0], "query1");
        assert!(result.get("top_n").is_none());
    }

    #[test]
    fn test_transform_rerank_request() {
        let transformation = DeepInfraRerankTransformation::new();

        let request = json!({
            "query": "test query",
            "documents": ["doc1", "doc2"],
            "top_n": 3
        });

        let result = transformation.transform_rerank_request(request);
        assert!(result.is_ok());
        let transformed = result.unwrap();
        assert!(transformed["queries"].is_array());
        assert!(transformed["documents"].is_array());
    }

    #[test]
    fn test_transform_rerank_request_invalid() {
        let transformation = DeepInfraRerankTransformation::new();

        let request = json!("invalid");

        let result = transformation.transform_rerank_request(request);
        assert!(result.is_err());
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
        assert_eq!(result.results[1].relevance_score, 0.7);
        assert_eq!(result.results[1].index, 1);
        assert_eq!(result.results[2].relevance_score, 0.5);
        assert_eq!(result.results[2].index, 2);
        assert_eq!(result.meta.tokens.input_tokens, 100);
        assert_eq!(result.meta.tokens.output_tokens, 0);
        assert_eq!(result.meta.billed_units.total_tokens, 100);
        assert!(result.usage.is_some());
        let usage = result.usage.unwrap();
        assert_eq!(usage.prompt_tokens, 100);
        assert_eq!(usage.total_tokens, 100);
    }

    #[test]
    fn test_transform_rerank_response_without_inference_status() {
        let transformation = DeepInfraRerankTransformation::new();

        let deepinfra_response = json!({
            "scores": [0.8, 0.6],
            "input_tokens": 50
        });

        let result = transformation
            .transform_rerank_response(deepinfra_response)
            .unwrap();

        assert!(!result.id.is_empty()); // UUID generated
        assert_eq!(result.results.len(), 2);
        assert!(result.usage.is_none());
    }

    #[test]
    fn test_transform_rerank_response_invalid() {
        let transformation = DeepInfraRerankTransformation::new();

        let invalid_response = json!({
            "invalid": "response"
        });

        let result = transformation.transform_rerank_response(invalid_response);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_error_with_detail_error() {
        let transformation = DeepInfraRerankTransformation::new();

        let error_response = json!({
            "detail": {
                "error": "Model not found"
            }
        });

        let msg = transformation.parse_error(error_response);
        assert_eq!(msg, "Model not found");
    }

    #[test]
    fn test_parse_error_with_detail_string() {
        let transformation = DeepInfraRerankTransformation::new();

        let error_response = json!({
            "detail": "Bad request"
        });

        let msg = transformation.parse_error(error_response);
        assert_eq!(msg, "Bad request");
    }

    #[test]
    fn test_parse_error_with_error_field() {
        let transformation = DeepInfraRerankTransformation::new();

        let error_response = json!({
            "error": "Rate limit exceeded"
        });

        let msg = transformation.parse_error(error_response);
        assert_eq!(msg, "Rate limit exceeded");
    }

    #[test]
    fn test_parse_error_fallback() {
        let transformation = DeepInfraRerankTransformation::new();

        let error_response = json!({
            "status": "error",
            "code": 500
        });

        let msg = transformation.parse_error(error_response);
        assert!(msg.contains("status"));
        assert!(msg.contains("error"));
    }

    // Struct serialization tests
    #[test]
    fn test_rerank_request_serialization() {
        let request = RerankRequest {
            query: Some("test".to_string()),
            queries: None,
            documents: vec![json!("doc1")],
            top_n: Some(5),
            return_documents: Some(true),
            max_chunks_per_doc: Some(10),
            max_tokens_per_doc: Some(512),
        };

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["query"], "test");
        assert_eq!(json["top_n"], 5);
        assert_eq!(json["return_documents"], true);
        assert_eq!(json["max_chunks_per_doc"], 10);
        assert_eq!(json["max_tokens_per_doc"], 512);
    }

    #[test]
    fn test_rerank_result_serialization() {
        let result = RerankResult {
            index: 0,
            relevance_score: 0.95,
            document: Some(json!({"text": "test document"})),
        };

        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["index"], 0);
        assert_eq!(json["relevance_score"], 0.95);
        assert!(json["document"].is_object());
    }

    #[test]
    fn test_rerank_response_serialization() {
        let response = RerankResponse {
            id: "test-id".to_string(),
            results: vec![RerankResult {
                index: 0,
                relevance_score: 0.9,
                document: None,
            }],
            meta: RerankMeta {
                tokens: RerankTokens {
                    input_tokens: 100,
                    output_tokens: 0,
                },
                billed_units: RerankBilledUnits {
                    total_tokens: 100,
                },
            },
            usage: None,
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["id"], "test-id");
        assert!(json["results"].is_array());
        assert!(json["meta"].is_object());
        // usage should be skipped when None
        assert!(json.get("usage").is_none());
    }

    #[test]
    fn test_inference_status_deserialization() {
        let json_str = r#"{
            "status": "success",
            "runtime_ms": 150,
            "cost": 0.001,
            "tokens_generated": 10,
            "tokens_input": 100
        }"#;

        let status: InferenceStatus = serde_json::from_str(json_str).unwrap();
        assert_eq!(status.status, "success");
        assert_eq!(status.runtime_ms, 150);
        assert_eq!(status.cost, 0.001);
        assert_eq!(status.tokens_generated, 10);
        assert_eq!(status.tokens_input, 100);
    }
}
