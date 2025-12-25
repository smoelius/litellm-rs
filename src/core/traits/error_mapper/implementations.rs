//! Provider-specific error mapper implementations
//!
//! This module contains error mappers for specific AI providers like OpenAI and Anthropic,
//! handling their unique error response formats.

use super::trait_def::ErrorMapper;
use super::types::GenericErrorMapper;
use crate::core::types::errors::ProviderErrorTrait;
use serde_json::Value;

/// OpenAI error mapper
///
/// Handles OpenAI-specific error response format
pub struct OpenAIErrorMapper;

impl<E> ErrorMapper<E> for OpenAIErrorMapper
where
    E: ProviderErrorTrait,
{
    fn map_http_error(&self, status_code: u16, response_body: &str) -> E {
        // Try to parse JSON response first
        if let Ok(error_json) = serde_json::from_str::<Value>(response_body) {
            return self.map_json_error(&error_json);
        }

        // If parsing fails, use generic mapping
        GenericErrorMapper.map_http_error(status_code, response_body)
    }

    fn map_json_error(&self, error_response: &Value) -> E {
        let error_obj = error_response.get("error");

        if let Some(error) = error_obj {
            let error_type = error.get("type").and_then(|t| t.as_str()).unwrap_or("");
            let error_code = error.get("code").and_then(|c| c.as_str()).unwrap_or("");
            let message = error
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error");

            match error_type {
                "invalid_request_error" => match error_code {
                    "model_not_found" => E::not_supported("Model not found"),
                    "context_length_exceeded" => E::network_error("Context length exceeded"),
                    "invalid_api_key" => E::authentication_failed("Invalid API key"),
                    _ => E::network_error(message),
                },
                "authentication_error" => E::authentication_failed(message),
                "permission_error" => E::authentication_failed(message),
                "rate_limit_error" => {
                    // Extract retry time
                    let retry_after = error.get("retry_after").and_then(|r| r.as_u64());
                    E::rate_limited(retry_after)
                }
                "api_error" => E::network_error(message),
                "overloaded_error" => E::network_error("OpenAI servers are overloaded"),
                _ => E::network_error(&format!("OpenAI Error: {}", message)),
            }
        } else {
            E::network_error("Invalid error response format")
        }
    }
}

/// Anthropic error mapper
///
/// Handles Anthropic-specific error response format
pub struct AnthropicErrorMapper;

impl<E> ErrorMapper<E> for AnthropicErrorMapper
where
    E: ProviderErrorTrait,
{
    fn map_http_error(&self, status_code: u16, response_body: &str) -> E {
        // Try to parse JSON response first
        if let Ok(error_json) = serde_json::from_str::<Value>(response_body) {
            return self.map_json_error(&error_json);
        }

        // If parsing fails, use generic mapping
        GenericErrorMapper.map_http_error(status_code, response_body)
    }

    fn map_json_error(&self, error_response: &Value) -> E {
        let error_type = error_response
            .get("type")
            .and_then(|t| t.as_str())
            .unwrap_or("");
        let message = error_response
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("Unknown error");

        match error_type {
            "authentication_error" => E::authentication_failed(message),
            "permission_error" => E::authentication_failed(message),
            "not_found_error" => E::not_supported(message),
            "rate_limit_error" => E::rate_limited(None),
            "api_error" => E::network_error(message),
            "overloaded_error" => E::network_error("Anthropic servers are overloaded"),
            "validation_error" => E::network_error(&format!("Validation error: {}", message)),
            _ => E::network_error(&format!("Anthropic Error: {}", message)),
        }
    }
}
