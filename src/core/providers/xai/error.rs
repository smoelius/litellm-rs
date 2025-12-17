//! xAI-specific error types and error mapping

use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::error_mapper::trait_def::ErrorMapper;
use crate::core::types::errors::ProviderErrorTrait;
use thiserror::Error;

/// xAI-specific error types
#[derive(Debug, Error)]
pub enum XAIError {
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

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Reasoning token limit exceeded: {0}")]
    ReasoningTokenLimitError(String),

    #[error("Web search error: {0}")]
    WebSearchError(String),
}

impl ProviderErrorTrait for XAIError {
    fn error_type(&self) -> &'static str {
        match self {
            XAIError::ApiError(_) => "api_error",
            XAIError::AuthenticationError(_) => "authentication_error",
            XAIError::RateLimitError(_) => "rate_limit_error",
            XAIError::InvalidRequestError(_) => "invalid_request_error",
            XAIError::ModelNotFoundError(_) => "model_not_found_error",
            XAIError::ServiceUnavailableError(_) => "service_unavailable_error",
            XAIError::ConfigurationError(_) => "configuration_error",
            XAIError::NetworkError(_) => "network_error",
            XAIError::ReasoningTokenLimitError(_) => "reasoning_token_limit_error",
            XAIError::WebSearchError(_) => "web_search_error",
        }
    }

    fn is_retryable(&self) -> bool {
        matches!(
            self,
            XAIError::RateLimitError(_)
                | XAIError::ServiceUnavailableError(_)
                | XAIError::NetworkError(_)
        )
    }

    fn retry_delay(&self) -> Option<u64> {
        match self {
            XAIError::RateLimitError(_) => Some(60), // Default 60 seconds for rate limit
            XAIError::ServiceUnavailableError(_) => Some(5), // 5 seconds for service unavailable
            XAIError::NetworkError(_) => Some(2),    // 2 seconds for network errors
            _ => None,
        }
    }

    fn http_status(&self) -> u16 {
        match self {
            XAIError::AuthenticationError(_) => 401,
            XAIError::RateLimitError(_) => 429,
            XAIError::InvalidRequestError(_) => 400,
            XAIError::ModelNotFoundError(_) => 404,
            XAIError::ServiceUnavailableError(_) => 503,
            XAIError::ApiError(_) => 500,
            _ => 500,
        }
    }

    fn not_supported(feature: &str) -> Self {
        XAIError::InvalidRequestError(format!("Feature not supported: {}", feature))
    }

    fn authentication_failed(reason: &str) -> Self {
        XAIError::AuthenticationError(reason.to_string())
    }

    fn rate_limited(retry_after: Option<u64>) -> Self {
        match retry_after {
            Some(seconds) => XAIError::RateLimitError(format!(
                "Rate limit exceeded, retry after {} seconds",
                seconds
            )),
            None => XAIError::RateLimitError("Rate limit exceeded".to_string()),
        }
    }

    fn network_error(details: &str) -> Self {
        XAIError::NetworkError(details.to_string())
    }

    fn parsing_error(details: &str) -> Self {
        XAIError::ApiError(format!("Response parsing error: {}", details))
    }

    fn not_implemented(feature: &str) -> Self {
        XAIError::InvalidRequestError(format!("Feature not implemented: {}", feature))
    }
}

impl From<XAIError> for ProviderError {
    fn from(error: XAIError) -> Self {
        match error {
            XAIError::ApiError(msg) => ProviderError::api_error("xai", 500, msg),
            XAIError::AuthenticationError(msg) => ProviderError::authentication("xai", msg),
            XAIError::RateLimitError(_) => ProviderError::rate_limit("xai", None),
            XAIError::InvalidRequestError(msg) => ProviderError::invalid_request("xai", msg),
            XAIError::ModelNotFoundError(msg) => ProviderError::model_not_found("xai", msg),
            XAIError::ServiceUnavailableError(msg) => ProviderError::api_error("xai", 503, msg),
            XAIError::ConfigurationError(msg) => ProviderError::configuration("xai", msg),
            XAIError::NetworkError(msg) => ProviderError::network("xai", msg),
            XAIError::ReasoningTokenLimitError(msg) => {
                ProviderError::token_limit_exceeded("xai", msg)
            }
            XAIError::WebSearchError(msg) => ProviderError::api_error("xai", 500, msg),
        }
    }
}

/// Error mapper for xAI provider
pub struct XAIErrorMapper;

impl ErrorMapper<XAIError> for XAIErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> XAIError {
        let message = if response_body.is_empty() {
            format!("HTTP error {}", status_code)
        } else {
            response_body.to_string()
        };

        match status_code {
            400 => XAIError::InvalidRequestError(message),
            401 => XAIError::AuthenticationError("Invalid API key".to_string()),
            403 => XAIError::AuthenticationError("Access forbidden".to_string()),
            404 => XAIError::ModelNotFoundError("Model not found".to_string()),
            429 => XAIError::RateLimitError("Rate limit exceeded".to_string()),
            500 => XAIError::ApiError("Internal server error".to_string()),
            502 => XAIError::ServiceUnavailableError("Bad gateway".to_string()),
            503 => XAIError::ServiceUnavailableError("Service unavailable".to_string()),
            _ => XAIError::ApiError(message),
        }
    }
}
