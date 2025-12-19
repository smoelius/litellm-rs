//! Azure OpenAI Embedding Handler
//!
//! Complete embedding functionality for Azure OpenAI Service

use reqwest::header::HeaderMap;
use serde_json::{Value, json};

use crate::core::types::{
    common::RequestContext,
    requests::EmbeddingRequest,
    responses::{EmbeddingData, EmbeddingResponse},
};

use super::config::AzureConfig;
use super::error::{AzureError, azure_api_error, azure_config_error, azure_header_error};
use super::utils::{AzureEndpointType, AzureUtils};
use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::provider::ProviderConfig;

/// Azure OpenAI embedding handler
#[derive(Debug, Clone)]
pub struct AzureEmbeddingHandler {
    config: AzureConfig,
    client: reqwest::Client,
}

impl AzureEmbeddingHandler {
    /// Create new embedding handler
    pub fn new(config: AzureConfig) -> Result<Self, AzureError> {
        let client = reqwest::Client::builder()
            .timeout(ProviderConfig::timeout(&config))
            .build()
            .map_err(|e| azure_config_error(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { config, client })
    }

    /// Build request headers
    async fn build_headers(&self) -> Result<HeaderMap, AzureError> {
        let mut headers = HeaderMap::new();

        // Add API key
        if let Some(api_key) = self.config.get_effective_api_key().await {
            headers.insert(
                "api-key",
                api_key
                    .parse()
                    .map_err(|e| azure_header_error(format!("Invalid API key: {}", e)))?,
            );
        } else {
            return Err(ProviderError::authentication(
                "azure",
                "No API key available",
            ));
        }

        headers.insert(
            "Content-Type",
            "application/json"
                .parse()
                .map_err(|e| azure_header_error(format!("Invalid content type: {}", e)))?,
        );

        // Add custom headers
        for (key, value) in &self.config.custom_headers {
            let header_name = reqwest::header::HeaderName::from_bytes(key.as_bytes())
                .map_err(|e| azure_header_error(format!("Invalid header name: {}", e)))?;
            let header_value = value
                .parse()
                .map_err(|e| azure_header_error(format!("Invalid header value: {}", e)))?;
            headers.insert(header_name, header_value);
        }

        Ok(headers)
    }

    /// Create embeddings
    pub async fn create_embeddings(
        &self,
        request: EmbeddingRequest,
        _context: RequestContext,
    ) -> Result<EmbeddingResponse, AzureError> {
        // Validate request
        AzureEmbeddingUtils::validate_request(&request)?;

        // Get deployment name (Azure uses deployment names for embeddings too)
        let deployment = self.config.get_effective_deployment_name(&request.model);

        // Get Azure endpoint
        let azure_endpoint = self
            .config
            .get_effective_azure_endpoint()
            .ok_or_else(|| azure_config_error("Azure endpoint not configured"))?;

        // Build URL
        let url = AzureUtils::build_azure_url(
            &azure_endpoint,
            &deployment,
            &self.config.api_version,
            AzureEndpointType::Embeddings,
        );

        // Transform request
        let azure_request = AzureEmbeddingUtils::transform_request(&request)?;

        // Build headers
        let headers = self.build_headers().await?;

        // Execute request
        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(&azure_request)
            .send()
            .await?;

        // Check status
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(azure_api_error(status, error_body));
        }

        // Parse response
        let response_json: Value = response.json().await?;

        // Transform response
        AzureEmbeddingUtils::transform_response(response_json, &request.model)
    }
}

/// Azure embedding utilities
pub struct AzureEmbeddingUtils;

impl AzureEmbeddingUtils {
    /// Validate embedding request
    pub fn validate_request(request: &EmbeddingRequest) -> Result<(), AzureError> {
        // Check if input is empty based on the enum variant
        let is_empty = match &request.input {
            crate::core::types::requests::EmbeddingInput::Text(text) => text.is_empty(),
            crate::core::types::requests::EmbeddingInput::Array(array) => array.is_empty(),
        };

        if is_empty {
            return Err(azure_config_error("Input cannot be empty"));
        }

        if request.model.is_empty() {
            return Err(azure_config_error("Model cannot be empty"));
        }

        // Validate dimensions if specified (only for certain models)
        if let Some(dimensions) = request.dimensions {
            if dimensions == 0 || dimensions > 3072 {
                return Err(azure_config_error("Dimensions must be between 1 and 3072"));
            }
        }

        Ok(())
    }

    /// Transform request to Azure format
    pub fn transform_request(request: &EmbeddingRequest) -> Result<Value, AzureError> {
        let mut body = json!({
            "model": request.model,
        });

        // Handle input based on enum variant
        match &request.input {
            crate::core::types::requests::EmbeddingInput::Text(text) => {
                body["input"] = json!(text);
            }
            crate::core::types::requests::EmbeddingInput::Array(array) => {
                body["input"] = json!(array);
            }
        }

        // Add optional parameters
        if let Some(dimensions) = request.dimensions {
            body["dimensions"] = json!(dimensions);
        }

        if let Some(user) = &request.user {
            body["user"] = json!(user);
        }

        if let Some(encoding_format) = &request.encoding_format {
            body["encoding_format"] = json!(encoding_format);
        }

        Ok(body)
    }

    /// Transform Azure response to standard format
    pub fn transform_response(
        response: Value,
        model: &str,
    ) -> Result<EmbeddingResponse, AzureError> {
        let data = response["data"]
            .as_array()
            .ok_or_else(|| ProviderError::serialization("azure", "Missing data array"))?
            .iter()
            .map(|item| {
                let embedding = item["embedding"]
                    .as_array()
                    .ok_or_else(|| {
                        ProviderError::serialization("azure", "Missing embedding array")
                    })?
                    .iter()
                    .filter_map(|v| v.as_f64().map(|f| f as f32))
                    .collect();

                Ok(EmbeddingData {
                    index: item["index"].as_u64().unwrap_or(0) as u32,
                    embedding,
                    object: "embedding".to_string(),
                })
            })
            .collect::<Result<Vec<_>, AzureError>>()?;

        // Calculate usage
        let usage = response["usage"]
            .as_object()
            .map(|u| crate::core::types::responses::Usage {
                prompt_tokens: u["prompt_tokens"].as_u64().unwrap_or(0) as u32,
                completion_tokens: u["completion_tokens"].as_u64().unwrap_or(0) as u32,
                total_tokens: u["total_tokens"].as_u64().unwrap_or(0) as u32,
                completion_tokens_details: None,
                prompt_tokens_details: None,
            thinking_usage: None,
            });

        Ok(EmbeddingResponse {
            object: "list".to_string(),
            data,
            model: model.to_string(),
            usage,
            // For backward compatibility
            embeddings: None,
        })
    }
}
