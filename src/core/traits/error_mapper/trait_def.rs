//! Error mapper trait definition
//!
//! This module defines the core ErrorMapper trait that provides a unified interface
//! for converting HTTP status codes, JSON error responses, and other error conditions
//! into structured error types.

use crate::core::types::errors::ProviderErrorTrait;
use serde_json::Value;

/// Trait for mapping various error conditions to provider-specific error types
///
/// This trait provides a unified interface for converting HTTP status codes,
/// JSON error responses, and other error conditions into structured error types.
///
/// # Design Goals
///
/// - Provide consistent error handling across all providers
/// - Support provider-specific error response formats
/// - Enable structured error information extraction
/// - Support retry logic and error categorization
///
/// # Implementation Guide
///
/// Implement this trait for each provider to handle their specific error formats:
///
/// ```rust,ignore
/// use litellm_rs::core::traits::error_mapper::ErrorMapper;
/// use litellm_rs::core::types::errors::ProviderErrorTrait;
///
/// struct MyProviderErrorMapper;
///
/// impl<E: ProviderErrorTrait> ErrorMapper<E> for MyProviderErrorMapper {
///     fn map_http_error(&self, status: u16, body: &str) -> E {
///         match status {
///             401 => E::authentication_failed("Invalid API key"),
///             429 => E::rate_limited(Some(60)),
///             _ => E::network_error(&format!("HTTP {}: {}", status, body)),
///         }
///     }
/// }
/// ```
pub trait ErrorMapper<E>: Send + Sync + 'static
where
    E: ProviderErrorTrait,
{
    /// Map HTTP status code and response body to provider error
    ///
    /// # Parameters
    /// * `status_code` - HTTP status code from the response
    /// * `response_body` - Raw response body as string
    ///
    /// # Returns
    /// Provider-specific error type
    ///
    /// # Common Mappings
    /// * `400` - Invalid request parameters
    /// * `401` - Authentication failure
    /// * `429` - Rate limit exceeded
    /// * `404` - Resource/model not found
    /// * `5xx` - Server-side errors
    fn map_http_error(&self, status_code: u16, response_body: &str) -> E;

    /// Map JSON error response to provider error
    ///
    /// # Parameters
    /// * `error_response` - Parsed JSON error response
    ///
    /// # Returns
    /// Provider-specific error type
    ///
    /// Default implementation handles common JSON error formats.
    /// Override for provider-specific error response structures.
    fn map_json_error(&self, error_response: &Value) -> E {
        let error_msg = error_response
            .get("error")
            .and_then(|e| e.get("message"))
            .and_then(|m| m.as_str())
            .unwrap_or("Unknown error");

        let error_code = error_response
            .get("error")
            .and_then(|e| e.get("code"))
            .and_then(|c| c.as_str())
            .unwrap_or("unknown");

        match error_code {
            "invalid_api_key" | "authentication_failed" => {
                E::authentication_failed("Invalid API key")
            }
            "insufficient_quota" | "quota_exceeded" => E::rate_limited(None),
            "model_not_found" => E::not_supported("Model not found"),
            "invalid_request_error" => E::network_error(error_msg),
            _ => E::network_error(&format!("API Error: {}", error_msg)),
        }
    }

    /// Map network-level errors to provider error
    ///
    /// # Parameters
    /// * `error` - Network or connection error
    ///
    /// # Returns
    /// Provider-specific error type
    ///
    /// Default implementation wraps the error as a network error.
    fn map_network_error(&self, error: &dyn std::error::Error) -> E {
        E::network_error(&error.to_string())
    }

    /// Map parsing/serialization errors to provider error
    ///
    /// # Parameters
    /// * `error` - Parsing or serialization error
    ///
    /// # Returns
    /// Provider-specific error type
    ///
    /// Default implementation maps to parsing error.
    fn map_parsing_error(&self, error: &dyn std::error::Error) -> E {
        E::parsing_error(&error.to_string())
    }

    /// Map timeout errors to provider error
    ///
    /// # Parameters
    /// * `timeout_duration` - Duration after which the request timed out
    ///
    /// # Returns
    /// Provider-specific error type
    ///
    /// Default implementation wraps timeout as network error.
    fn map_timeout_error(&self, timeout_duration: std::time::Duration) -> E {
        E::network_error(&format!("Request timeout after {:?}", timeout_duration))
    }
}
