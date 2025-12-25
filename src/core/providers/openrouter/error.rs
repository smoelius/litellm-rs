//! OpenRouter Error types

use crate::core::types::errors::ProviderErrorTrait;
use thiserror::Error;

/// OpenRouter specific errors
#[derive(Error, Debug)]
pub enum OpenRouterError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Network error
    #[error("Network error: {0}")]
    Network(String),

    /// Parsing error
    #[error("Failed to parse response: {0}")]
    Parsing(String),

    /// Authentication error
    #[error("Authentication failed: {0}")]
    Authentication(String),

    /// Rate limit error
    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    /// Model not supported
    #[error("Model not supported: {0}")]
    UnsupportedModel(String),

    /// Feature not supported
    #[error("Feature not supported: {0}")]
    UnsupportedFeature(String),

    /// Request timeout
    #[error("Request timeout: {0}")]
    Timeout(String),

    /// API error with status code
    #[error("API error (status {status_code}): {message}")]
    ApiError { status_code: u16, message: String },

    /// Invalid request
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Transformation error
    #[error("Transformation error: {0}")]
    Transformation(String),

    /// Model not found
    #[error("Model not found: {0}")]
    ModelNotFound(String),

    /// Other error
    #[error("{0}")]
    Other(String),
}

impl From<serde_json::Error> for OpenRouterError {
    fn from(err: serde_json::Error) -> Self {
        Self::Parsing(err.to_string())
    }
}

impl From<crate::core::types::errors::OpenAIError> for OpenRouterError {
    fn from(err: crate::core::types::errors::OpenAIError) -> Self {
        Self::Transformation(format!("OpenAI transformation error: {}", err))
    }
}

impl ProviderErrorTrait for OpenRouterError {
    fn error_type(&self) -> &'static str {
        match self {
            Self::Configuration(_) => "configuration",
            Self::Network(_) => "network",
            Self::Parsing(_) => "parsing",
            Self::Authentication(_) => "authentication",
            Self::RateLimit(_) => "rate_limit",
            Self::UnsupportedModel(_) => "unsupported_model",
            Self::UnsupportedFeature(_) => "unsupported_feature",
            Self::Timeout(_) => "timeout",
            Self::ApiError { .. } => "api_error",
            Self::InvalidRequest(_) => "invalid_request",
            Self::Transformation(_) => "transformation",
            Self::ModelNotFound(_) => "model_not_found",
            Self::Other(_) => "other",
        }
    }

    fn is_retryable(&self) -> bool {
        match self {
            Self::Network(_) | Self::Timeout(_) => true,
            Self::RateLimit(_) => true,
            Self::ApiError { status_code, .. } if *status_code >= 500 => true,
            _ => false,
        }
    }

    fn retry_delay(&self) -> Option<u64> {
        match self {
            Self::RateLimit(_) => Some(60), // Wait 60 seconds for rate limit
            Self::Timeout(_) => Some(5),    // Quick retry for timeout
            Self::Network(_) => Some(10),   // 10 second delay for network issues
            _ if self.is_retryable() => Some(15), // Default retry delay
            _ => None,
        }
    }

    fn http_status(&self) -> u16 {
        match self {
            Self::ApiError { status_code, .. } => *status_code,
            Self::Authentication(_) => 401,
            Self::RateLimit(_) => 429,
            Self::Configuration(_) => 400,
            Self::InvalidRequest(_) => 400,
            Self::UnsupportedModel(_) | Self::UnsupportedFeature(_) => 404,
            _ => 500,
        }
    }

    fn not_supported(feature: &str) -> Self {
        Self::UnsupportedFeature(feature.to_string())
    }

    fn authentication_failed(reason: &str) -> Self {
        Self::Authentication(reason.to_string())
    }

    fn rate_limited(_retry_after: Option<u64>) -> Self {
        Self::RateLimit("Rate limit exceeded".to_string())
    }

    fn network_error(details: &str) -> Self {
        Self::Network(details.to_string())
    }

    fn parsing_error(details: &str) -> Self {
        Self::Parsing(details.to_string())
    }

    fn not_implemented(feature: &str) -> Self {
        Self::UnsupportedFeature(format!("Feature not implemented: {}", feature))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openrouter_error_display() {
        let err = OpenRouterError::Configuration("missing api key".to_string());
        assert_eq!(err.to_string(), "Configuration error: missing api key");

        let err = OpenRouterError::Network("connection failed".to_string());
        assert_eq!(err.to_string(), "Network error: connection failed");

        let err = OpenRouterError::Authentication("invalid key".to_string());
        assert_eq!(err.to_string(), "Authentication failed: invalid key");

        let err = OpenRouterError::ApiError {
            status_code: 500,
            message: "server error".to_string(),
        };
        assert_eq!(err.to_string(), "API error (status 500): server error");
    }

    #[test]
    fn test_openrouter_error_type() {
        assert_eq!(OpenRouterError::Configuration("".to_string()).error_type(), "configuration");
        assert_eq!(OpenRouterError::Network("".to_string()).error_type(), "network");
        assert_eq!(OpenRouterError::Parsing("".to_string()).error_type(), "parsing");
        assert_eq!(OpenRouterError::Authentication("".to_string()).error_type(), "authentication");
        assert_eq!(OpenRouterError::RateLimit("".to_string()).error_type(), "rate_limit");
        assert_eq!(OpenRouterError::UnsupportedModel("".to_string()).error_type(), "unsupported_model");
        assert_eq!(OpenRouterError::UnsupportedFeature("".to_string()).error_type(), "unsupported_feature");
        assert_eq!(OpenRouterError::Timeout("".to_string()).error_type(), "timeout");
        assert_eq!(OpenRouterError::ApiError { status_code: 500, message: "".to_string() }.error_type(), "api_error");
        assert_eq!(OpenRouterError::InvalidRequest("".to_string()).error_type(), "invalid_request");
        assert_eq!(OpenRouterError::Transformation("".to_string()).error_type(), "transformation");
        assert_eq!(OpenRouterError::ModelNotFound("".to_string()).error_type(), "model_not_found");
        assert_eq!(OpenRouterError::Other("".to_string()).error_type(), "other");
    }

    #[test]
    fn test_openrouter_error_is_retryable() {
        assert!(OpenRouterError::Network("".to_string()).is_retryable());
        assert!(OpenRouterError::Timeout("".to_string()).is_retryable());
        assert!(OpenRouterError::RateLimit("".to_string()).is_retryable());
        assert!(OpenRouterError::ApiError { status_code: 500, message: "".to_string() }.is_retryable());
        assert!(OpenRouterError::ApiError { status_code: 503, message: "".to_string() }.is_retryable());

        assert!(!OpenRouterError::Authentication("".to_string()).is_retryable());
        assert!(!OpenRouterError::Configuration("".to_string()).is_retryable());
        assert!(!OpenRouterError::InvalidRequest("".to_string()).is_retryable());
        assert!(!OpenRouterError::ApiError { status_code: 400, message: "".to_string() }.is_retryable());
    }

    #[test]
    fn test_openrouter_error_retry_delay() {
        assert_eq!(OpenRouterError::RateLimit("".to_string()).retry_delay(), Some(60));
        assert_eq!(OpenRouterError::Timeout("".to_string()).retry_delay(), Some(5));
        assert_eq!(OpenRouterError::Network("".to_string()).retry_delay(), Some(10));
        assert_eq!(OpenRouterError::ApiError { status_code: 500, message: "".to_string() }.retry_delay(), Some(15));
        assert_eq!(OpenRouterError::Authentication("".to_string()).retry_delay(), None);
    }

    #[test]
    fn test_openrouter_error_http_status() {
        assert_eq!(OpenRouterError::ApiError { status_code: 500, message: "".to_string() }.http_status(), 500);
        assert_eq!(OpenRouterError::Authentication("".to_string()).http_status(), 401);
        assert_eq!(OpenRouterError::RateLimit("".to_string()).http_status(), 429);
        assert_eq!(OpenRouterError::Configuration("".to_string()).http_status(), 400);
        assert_eq!(OpenRouterError::InvalidRequest("".to_string()).http_status(), 400);
        assert_eq!(OpenRouterError::UnsupportedModel("".to_string()).http_status(), 404);
        assert_eq!(OpenRouterError::UnsupportedFeature("".to_string()).http_status(), 404);
        assert_eq!(OpenRouterError::Other("".to_string()).http_status(), 500);
    }

    #[test]
    fn test_openrouter_error_factory_methods() {
        let err = OpenRouterError::not_supported("vision");
        assert!(matches!(err, OpenRouterError::UnsupportedFeature(_)));

        let err = OpenRouterError::authentication_failed("bad key");
        assert!(matches!(err, OpenRouterError::Authentication(_)));

        let err = OpenRouterError::rate_limited(Some(60));
        assert!(matches!(err, OpenRouterError::RateLimit(_)));

        let err = OpenRouterError::network_error("timeout");
        assert!(matches!(err, OpenRouterError::Network(_)));

        let err = OpenRouterError::parsing_error("invalid json");
        assert!(matches!(err, OpenRouterError::Parsing(_)));

        let err = OpenRouterError::not_implemented("feature");
        assert!(matches!(err, OpenRouterError::UnsupportedFeature(_)));
    }

    #[test]
    fn test_openrouter_error_from_serde_error() {
        let json_err = serde_json::from_str::<String>("invalid").unwrap_err();
        let err: OpenRouterError = json_err.into();
        assert!(matches!(err, OpenRouterError::Parsing(_)));
    }
}
