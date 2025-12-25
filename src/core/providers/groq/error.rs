//! Groq-specific error types and error mapping
//!
//! Handles error conversion from Groq API responses to unified provider errors.

use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::error_mapper::trait_def::ErrorMapper;
use crate::core::types::errors::ProviderErrorTrait;
use thiserror::Error;

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
            GroqError::NetworkError(_) => Some(2),    // 2 seconds for network errors
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
            Some(seconds) => GroqError::RateLimitError(format!(
                "Rate limit exceeded, retry after {} seconds",
                seconds
            )),
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
            GroqError::StreamingError(msg) => {
                ProviderError::api_error("groq", 500, format!("Streaming error: {}", msg))
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_groq_error_display() {
        let err = GroqError::ApiError("test error".to_string());
        assert_eq!(err.to_string(), "API error: test error");

        let err = GroqError::AuthenticationError("invalid key".to_string());
        assert_eq!(err.to_string(), "Authentication failed: invalid key");

        let err = GroqError::RateLimitError("limit exceeded".to_string());
        assert_eq!(err.to_string(), "Rate limit exceeded: limit exceeded");
    }

    #[test]
    fn test_groq_error_type() {
        assert_eq!(
            GroqError::ApiError("".to_string()).error_type(),
            "api_error"
        );
        assert_eq!(
            GroqError::AuthenticationError("".to_string()).error_type(),
            "authentication_error"
        );
        assert_eq!(
            GroqError::RateLimitError("".to_string()).error_type(),
            "rate_limit_error"
        );
        assert_eq!(
            GroqError::InvalidRequestError("".to_string()).error_type(),
            "invalid_request_error"
        );
        assert_eq!(
            GroqError::ModelNotFoundError("".to_string()).error_type(),
            "model_not_found_error"
        );
        assert_eq!(
            GroqError::ServiceUnavailableError("".to_string()).error_type(),
            "service_unavailable_error"
        );
        assert_eq!(
            GroqError::StreamingError("".to_string()).error_type(),
            "streaming_error"
        );
        assert_eq!(
            GroqError::ConfigurationError("".to_string()).error_type(),
            "configuration_error"
        );
        assert_eq!(
            GroqError::NetworkError("".to_string()).error_type(),
            "network_error"
        );
        assert_eq!(
            GroqError::UnknownError("".to_string()).error_type(),
            "unknown_error"
        );
    }

    #[test]
    fn test_groq_error_is_retryable() {
        assert!(GroqError::RateLimitError("".to_string()).is_retryable());
        assert!(GroqError::ServiceUnavailableError("".to_string()).is_retryable());
        assert!(GroqError::NetworkError("".to_string()).is_retryable());

        assert!(!GroqError::ApiError("".to_string()).is_retryable());
        assert!(!GroqError::AuthenticationError("".to_string()).is_retryable());
        assert!(!GroqError::InvalidRequestError("".to_string()).is_retryable());
        assert!(!GroqError::ModelNotFoundError("".to_string()).is_retryable());
    }

    #[test]
    fn test_groq_error_retry_delay() {
        assert_eq!(
            GroqError::RateLimitError("".to_string()).retry_delay(),
            Some(60)
        );
        assert_eq!(
            GroqError::ServiceUnavailableError("".to_string()).retry_delay(),
            Some(5)
        );
        assert_eq!(
            GroqError::NetworkError("".to_string()).retry_delay(),
            Some(2)
        );
        assert_eq!(GroqError::ApiError("".to_string()).retry_delay(), None);
    }

    #[test]
    fn test_groq_error_http_status() {
        assert_eq!(
            GroqError::AuthenticationError("".to_string()).http_status(),
            401
        );
        assert_eq!(
            GroqError::RateLimitError("".to_string()).http_status(),
            429
        );
        assert_eq!(
            GroqError::InvalidRequestError("".to_string()).http_status(),
            400
        );
        assert_eq!(
            GroqError::ModelNotFoundError("".to_string()).http_status(),
            404
        );
        assert_eq!(
            GroqError::ServiceUnavailableError("".to_string()).http_status(),
            503
        );
        assert_eq!(GroqError::ApiError("".to_string()).http_status(), 500);
    }

    #[test]
    fn test_groq_error_factory_methods() {
        let err = GroqError::not_supported("vision");
        assert!(matches!(err, GroqError::InvalidRequestError(_)));

        let err = GroqError::authentication_failed("bad key");
        assert!(matches!(err, GroqError::AuthenticationError(_)));

        let err = GroqError::rate_limited(Some(30));
        assert!(matches!(err, GroqError::RateLimitError(_)));

        let err = GroqError::rate_limited(None);
        assert!(matches!(err, GroqError::RateLimitError(_)));

        let err = GroqError::network_error("connection failed");
        assert!(matches!(err, GroqError::NetworkError(_)));

        let err = GroqError::parsing_error("invalid json");
        assert!(matches!(err, GroqError::ApiError(_)));

        let err = GroqError::not_implemented("feature");
        assert!(matches!(err, GroqError::InvalidRequestError(_)));
    }

    #[test]
    fn test_groq_error_to_provider_error() {
        let err: ProviderError = GroqError::AuthenticationError("bad key".to_string()).into();
        assert!(matches!(err, ProviderError::Authentication { .. }));

        let err: ProviderError = GroqError::RateLimitError("limit".to_string()).into();
        assert!(matches!(err, ProviderError::RateLimit { .. }));

        let err: ProviderError = GroqError::ModelNotFoundError("gpt-5".to_string()).into();
        assert!(matches!(err, ProviderError::ModelNotFound { .. }));

        let err: ProviderError = GroqError::ConfigurationError("bad config".to_string()).into();
        assert!(matches!(err, ProviderError::Configuration { .. }));

        let err: ProviderError = GroqError::NetworkError("timeout".to_string()).into();
        assert!(matches!(err, ProviderError::Network { .. }));
    }

    #[test]
    fn test_groq_error_mapper_http_errors() {
        let mapper = GroqErrorMapper;

        let err = mapper.map_http_error(400, "bad request");
        assert!(matches!(err, GroqError::InvalidRequestError(_)));

        let err = mapper.map_http_error(401, "");
        assert!(matches!(err, GroqError::AuthenticationError(_)));

        let err = mapper.map_http_error(403, "");
        assert!(matches!(err, GroqError::AuthenticationError(_)));

        let err = mapper.map_http_error(404, "");
        assert!(matches!(err, GroqError::ModelNotFoundError(_)));

        let err = mapper.map_http_error(429, "");
        assert!(matches!(err, GroqError::RateLimitError(_)));

        let err = mapper.map_http_error(500, "");
        assert!(matches!(err, GroqError::ApiError(_)));

        let err = mapper.map_http_error(502, "");
        assert!(matches!(err, GroqError::ServiceUnavailableError(_)));

        let err = mapper.map_http_error(503, "");
        assert!(matches!(err, GroqError::ServiceUnavailableError(_)));

        let err = mapper.map_http_error(418, "teapot");
        assert!(matches!(err, GroqError::ApiError(_)));
    }

    #[test]
    fn test_groq_error_mapper_empty_body() {
        let mapper = GroqErrorMapper;
        let err = mapper.map_http_error(400, "");
        if let GroqError::InvalidRequestError(msg) = err {
            assert!(msg.contains("HTTP error 400"));
        } else {
            panic!("Expected InvalidRequestError");
        }
    }
}
