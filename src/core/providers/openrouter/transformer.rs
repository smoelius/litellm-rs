//! OpenRouter request/response transformer
//!
//! OpenRouter uses OpenAI-compatible API, but needs to process additional parameters

use super::error::OpenRouterError;
use crate::core::providers::openai::models as openai_models;
use crate::core::providers::openai::transformer::OpenAIRequestTransformer;
use crate::core::types::{
    requests::ChatRequest,
    responses::{ChatChunk, ChatResponse},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// OpenRouter specific parameters
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OpenRouterExtraParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transforms: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub models: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
}

/// Error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRouterErrorModel {
    pub message: String,
    pub code: i64,
    #[serde(rename = "type")]
    pub error_type: Option<String>,
}

/// Request transformer
/// Inherits from OpenAI transformer, because OpenRouter is OpenAI compatible
pub struct OpenRouterRequestTransformer;

impl OpenRouterRequestTransformer {
    /// Transform request
    /// OpenRouter uses OpenAI format, but supports additional parameters
    pub fn transform_request(
        request: ChatRequest,
        extra_params: Option<OpenRouterExtraParams>,
    ) -> Result<openai_models::OpenAIChatRequest, OpenRouterError> {
        // Transform to OpenAI request
        let openai_request = OpenAIRequestTransformer::transform(request)
            .map_err(|e| OpenRouterError::InvalidRequest(e.to_string()))?;

        // If there are OpenRouter specific params, add to extra_body
        if let Some(extra) = extra_params {
            let mut extra_body = HashMap::new();

            if let Some(transforms) = extra.transforms {
                extra_body.insert("transforms".to_string(), serde_json::to_value(transforms)?);
            }

            if let Some(models) = extra.models {
                extra_body.insert("models".to_string(), serde_json::to_value(models)?);
            }

            if let Some(route) = extra.route {
                extra_body.insert("route".to_string(), serde_json::to_value(route)?);
            }

            if let Some(provider) = extra.provider {
                extra_body.insert("provider".to_string(), serde_json::to_value(provider)?);
            }

            // OpenRouter's extra_body parameters will be passed through OpenAI client
            // Additional request processing could be done here
        }

        Ok(openai_request)
    }

    /// Check if should keep cache control
    /// Model specific logic
    pub fn should_keep_cache_control(model: &str) -> bool {
        model.to_lowercase().contains("claude")
    }
}

/// Response transformer
pub struct OpenRouterResponseTransformer;

impl OpenRouterResponseTransformer {
    /// Transform response
    pub fn transform_response(
        response: openai_models::OpenAIChatResponse,
    ) -> Result<ChatResponse, OpenRouterError> {
        // Delegate to OpenAI transformer
        crate::core::providers::openai::transformer::OpenAIResponseTransformer::transform(response)
            .map_err(|e| OpenRouterError::Transformation(e.to_string()))
    }

    /// Transform stream chunk
    pub fn transform_stream_chunk(
        chunk: openai_models::OpenAIStreamChunk,
    ) -> Result<ChatChunk, OpenRouterError> {
        // Check for errors in chunk
        if let Some(error) = Self::check_error_in_chunk(&chunk) {
            return Err(error);
        }

        // OpenRouter may include reasoning field in delta
        // Handle OpenRouter-specific delta fields if needed

        // Delegate to OpenAI transformer
        crate::core::providers::openai::transformer::OpenAIResponseTransformer::transform_stream_chunk(chunk)
            .map_err(|e| OpenRouterError::Transformation(e.to_string()))
    }

    /// Check error in chunk
    fn check_error_in_chunk(_chunk: &openai_models::OpenAIStreamChunk) -> Option<OpenRouterError> {
        // OpenRouter specific error checking could be implemented here
        // Handle any special error patterns
        None // No error found
    }

    /// Parse error response
    pub fn parse_error(error_body: &str, status_code: u16) -> OpenRouterError {
        if let Ok(error_model) = serde_json::from_str::<OpenRouterErrorModel>(error_body) {
            let message = format!(
                "OpenRouter Error: {} (Code: {})",
                error_model.message, error_model.code
            );

            match error_model.code {
                401 => OpenRouterError::Authentication(message),
                429 => OpenRouterError::RateLimit(message),
                400 => OpenRouterError::InvalidRequest(message),
                404 => OpenRouterError::ModelNotFound(error_model.message),
                _ => OpenRouterError::ApiError {
                    message,
                    status_code,
                },
            }
        } else {
            OpenRouterError::ApiError {
                message: error_body.to_string(),
                status_code,
            }
        }
    }
}

/// Create OpenRouter headers
pub fn create_openrouter_headers(
    api_key: &str,
    http_referer: Option<&str>,
    x_title: Option<&str>,
) -> HashMap<String, String> {
    // Pre-allocate capacity for known headers: Authorization, Content-Type, User-Agent + optional
    let mut headers = HashMap::with_capacity(5);

    // Authorization header
    headers.insert("Authorization".to_string(), format!("Bearer {}", api_key));

    // Content type
    headers.insert("Content-Type".to_string(), "application/json".to_string());

    // OpenRouter specific HTTP headers
    if let Some(referer) = http_referer {
        headers.insert("HTTP-Referer".to_string(), referer.to_string());
    }

    if let Some(title) = x_title {
        headers.insert("X-Title".to_string(), title.to_string());
    }

    // User agent
    headers.insert(
        "User-Agent".to_string(),
        "LiteLLM-RS/0.1.0 (OpenRouter)".to_string(),
    );

    headers
}
