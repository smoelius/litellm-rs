//! Groq-specific error types and error mapping
//!
//! Handles error conversion from Groq API responses to unified provider errors.

use thiserror::Error;
use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::ErrorMapper;
use crate::core::types::errors::ProviderErrorTrait;

/// Groq-specific error types
#[derive(Debug, Error)]
pub enum GroqError {
    #[error("API error: {0}")]
    ApiError(String),

    #[error("Authentication failed: {0}")]
    AuthenticationError(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimitError(String),

    #[error("Invalid request: {0}")]
    InvalidRequestError(String),

    #[error("Model not found: {0}")]
    ModelNotFoundError(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailableError(String),

    #[error("Streaming error: {0}")]
    StreamingError(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Unknown error: {0}")]
    UnknownError(String),
}

impl ProviderErrorTrait for GroqError {
    fn error_type(&self) -> &'static str {
        match self {
            GroqError::ApiError(_) => "api_error",
            GroqError::AuthenticationError(_) => "authentication_error",
            GroqError::RateLimitError(_) => "rate_limit_error",
            GroqError::InvalidRequestError(_) => "invalid_request_error",
            GroqError::ModelNotFoundError(_) => "model_not_found_error",
            GroqError::ServiceUnavailableError(_) => "service_unavailable_error",
            GroqError::StreamingError(_) => "streaming_error",
            GroqError::ConfigurationError(_) => "configuration_error",
            GroqError::NetworkError(_) => "network_error",
            GroqError::UnknownError(_) => "unknown_error",
        }
    }

    fn is_retryable(&self) -> bool {
        matches!(
            self,
            GroqError::RateLimitError(_)
                | GroqError::ServiceUnavailableError(_)
                | GroqError::NetworkError(_)
        )
    }

    fn retry_delay(&self) -> Option<u64> {
        match self {
            GroqError::RateLimitError(_) => Some(60), // Default 60 seconds for rate limit
            GroqError::ServiceUnavailableError(_) => Some(5), // 5 seconds for service unavailable
            GroqError::NetworkError(_) => Some(2), // 2 seconds for network errors
            _ => None,
        }
    }

    fn http_status(&self) -> u16 {
        match self {
            GroqError::AuthenticationError(_) => 401,
            GroqError::RateLimitError(_) => 429,
            GroqError::InvalidRequestError(_) => 400,
            GroqError::ModelNotFoundError(_) => 404,
            GroqError::ServiceUnavailableError(_) => 503,
            GroqError::ApiError(_) => 500,
            _ => 500,
        }
    }

    fn not_supported(feature: &str) -> Self {
        GroqError::InvalidRequestError(format!("Feature not supported: {}", feature))
    }

    fn authentication_failed(reason: &str) -> Self {
        GroqError::AuthenticationError(reason.to_string())
    }

    fn rate_limited(retry_after: Option<u64>) -> Self {
        match retry_after {
            Some(seconds) => GroqError::RateLimitError(format!("Rate limit exceeded, retry after {} seconds", seconds)),
            None => GroqError::RateLimitError("Rate limit exceeded".to_string()),
        }
    }

    fn network_error(details: &str) -> Self {
        GroqError::NetworkError(details.to_string())
    }

    fn parsing_error(details: &str) -> Self {
        GroqError::ApiError(format!("Response parsing error: {}", details))
    }

    fn not_implemented(feature: &str) -> Self {
        GroqError::InvalidRequestError(format!("Feature not implemented: {}", feature))
    }
}

impl From<GroqError> for ProviderError {
    fn from(error: GroqError) -> Self {
        match error {
            GroqError::ApiError(msg) => ProviderError::api_error("groq", 500, msg),
            GroqError::AuthenticationError(msg) => ProviderError::authentication("groq", msg),
            GroqError::RateLimitError(_) => ProviderError::rate_limit("groq", None),
            GroqError::InvalidRequestError(msg) => ProviderError::invalid_request("groq", msg),
            GroqError::ModelNotFoundError(msg) => ProviderError::model_not_found("groq", msg),
            GroqError::ServiceUnavailableError(msg) => ProviderError::api_error("groq", 503, msg),
            GroqError::StreamingError(msg) => ProviderError::api_error("groq", 500, format!("Streaming error: {}", msg)),
            GroqError::ConfigurationError(msg) => ProviderError::configuration("groq", msg),
            GroqError::NetworkError(msg) => ProviderError::network("groq", msg),
            GroqError::UnknownError(msg) => ProviderError::api_error("groq", 500, msg),
        }
    }
}

/// Error mapper for Groq provider
pub struct GroqErrorMapper;

impl ErrorMapper<GroqError> for GroqErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> GroqError {
        let message = if response_body.is_empty() {
            format!("HTTP error {}", status_code)
        } else {
            response_body.to_string()
        };

        match status_code {
            400 => GroqError::InvalidRequestError(message),
            401 => GroqError::AuthenticationError("Invalid API key".to_string()),
            403 => GroqError::AuthenticationError("Access forbidden".to_string()),
            404 => GroqError::ModelNotFoundError("Model not found".to_string()),
            429 => GroqError::RateLimitError("Rate limit exceeded".to_string()),
            500 => GroqError::ApiError("Internal server error".to_string()),
            502 => GroqError::ServiceUnavailableError("Bad gateway".to_string()),
            503 => GroqError::ServiceUnavailableError("Service unavailable".to_string()),
            _ => GroqError::ApiError(message),
        }
    }
}