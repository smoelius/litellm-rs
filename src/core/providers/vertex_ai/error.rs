//! Vertex AI Error types

use thiserror::Error;

/// Vertex AI specific errors
#[derive(Error, Debug)]
pub enum VertexAIError {
    /// Authentication error
    #[error("Authentication failed: {0}")]
    Authentication(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Network error
    #[error("Network error: {0}")]
    Network(String),

    /// API error with status code
    #[error("API error (status {status_code}): {message}")]
    ApiError { status_code: u16, message: String },

    /// Response parsing error
    #[error("Failed to parse response: {0}")]
    ResponseParsing(String),

    /// Unsupported model
    #[error("Unsupported model: {0}")]
    UnsupportedModel(String),

    /// Unsupported feature
    #[error("Unsupported feature: {0}")]
    UnsupportedFeature(String),

    /// Invalid request
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    /// Quota exceeded
    #[error("Quota exceeded for model: {0}")]
    QuotaExceeded(String),

    /// Token limit exceeded
    #[error("Token limit exceeded: {0}")]
    TokenLimitExceeded(String),

    /// Context length exceeded
    #[error("Context length exceeded: max {max}, got {actual}")]
    ContextLengthExceeded { max: usize, actual: usize },

    /// Content filter triggered
    #[error("Content was blocked by safety filters")]
    ContentFiltered,

    /// Service unavailable
    #[error("Vertex AI service is temporarily unavailable")]
    ServiceUnavailable,

    /// Timeout
    #[error("Request timed out after {0} seconds")]
    Timeout(u64),

    /// Feature disabled
    #[error("Feature disabled: {0}")]
    FeatureDisabled(String),

    /// Other error
    #[error("{0}")]
    Other(String),
}

impl From<serde_json::Error> for VertexAIError {
    fn from(err: serde_json::Error) -> Self {
        Self::ResponseParsing(err.to_string())
    }
}

impl VertexAIError {
    /// Check if error is retryable
    pub fn is_retryable_internal(&self) -> bool {
        match self {
            Self::Network(_)
            | Self::RateLimitExceeded
            | Self::ServiceUnavailable
            | Self::Timeout(_) => true,
            Self::ApiError { status_code, .. } if *status_code >= 500 => true,
            _ => false,
        }
    }

    /// Get the HTTP status code if applicable
    pub fn status_code(&self) -> Option<u16> {
        match self {
            Self::ApiError { status_code, .. } => Some(*status_code),
            Self::RateLimitExceeded => Some(429),
            Self::ServiceUnavailable => Some(503),
            _ => None,
        }
    }
}

impl crate::core::types::errors::ProviderErrorTrait for VertexAIError {
    fn error_type(&self) -> &'static str {
        match self {
            Self::Authentication(_) => "authentication",
            Self::Configuration(_) => "configuration",
            Self::Network(_) => "network",
            Self::ApiError { .. } => "api_error",
            Self::ResponseParsing(_) => "parsing",
            Self::UnsupportedModel(_) => "unsupported_model",
            Self::UnsupportedFeature(_) => "unsupported_feature",
            Self::InvalidRequest(_) => "invalid_request",
            Self::RateLimitExceeded => "rate_limit",
            Self::QuotaExceeded(_) => "quota_exceeded",
            Self::TokenLimitExceeded(_) => "token_limit",
            Self::ContextLengthExceeded { .. } => "context_length",
            Self::ContentFiltered => "content_filtered",
            Self::ServiceUnavailable => "service_unavailable",
            Self::Timeout(_) => "timeout",
            Self::FeatureDisabled(_) => "feature_disabled",
            Self::Other(_) => "other",
        }
    }

    fn is_retryable(&self) -> bool {
        self.is_retryable_internal()
    }

    fn retry_delay(&self) -> Option<u64> {
        match self {
            Self::RateLimitExceeded => Some(60), // Wait 60 seconds for rate limit
            Self::ServiceUnavailable => Some(30), // Wait 30 seconds for service issues
            Self::Network(_) | Self::Timeout(_) => Some(5), // Quick retry for network issues
            _ if self.is_retryable_internal() => Some(10), // Default retry delay
            _ => None,
        }
    }

    fn http_status(&self) -> u16 {
        self.status_code().unwrap_or(0)
    }

    fn not_supported(feature: &str) -> Self {
        Self::UnsupportedFeature(feature.to_string())
    }

    fn authentication_failed(reason: &str) -> Self {
        Self::Authentication(reason.to_string())
    }

    fn rate_limited(_retry_after: Option<u64>) -> Self {
        Self::RateLimitExceeded
    }

    fn network_error(details: &str) -> Self {
        Self::Network(details.to_string())
    }

    fn parsing_error(details: &str) -> Self {
        Self::ResponseParsing(details.to_string())
    }

    fn not_implemented(feature: &str) -> Self {
        Self::UnsupportedFeature(format!("Feature not implemented: {}", feature))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::errors::ProviderErrorTrait;

    #[test]
    fn test_vertex_error_display() {
        let err = VertexAIError::Authentication("bad key".to_string());
        assert_eq!(err.to_string(), "Authentication failed: bad key");

        let err = VertexAIError::Configuration("missing project".to_string());
        assert_eq!(err.to_string(), "Configuration error: missing project");

        let err = VertexAIError::Network("connection refused".to_string());
        assert_eq!(err.to_string(), "Network error: connection refused");

        let err = VertexAIError::ApiError {
            status_code: 500,
            message: "server error".to_string(),
        };
        assert_eq!(err.to_string(), "API error (status 500): server error");

        let err = VertexAIError::RateLimitExceeded;
        assert_eq!(err.to_string(), "Rate limit exceeded");

        let err = VertexAIError::ServiceUnavailable;
        assert_eq!(err.to_string(), "Vertex AI service is temporarily unavailable");

        let err = VertexAIError::Timeout(30);
        assert_eq!(err.to_string(), "Request timed out after 30 seconds");

        let err = VertexAIError::ContextLengthExceeded { max: 4096, actual: 5000 };
        assert_eq!(err.to_string(), "Context length exceeded: max 4096, got 5000");
    }

    #[test]
    fn test_vertex_error_type() {
        assert_eq!(VertexAIError::Authentication("".to_string()).error_type(), "authentication");
        assert_eq!(VertexAIError::Configuration("".to_string()).error_type(), "configuration");
        assert_eq!(VertexAIError::Network("".to_string()).error_type(), "network");
        assert_eq!(VertexAIError::ApiError { status_code: 500, message: "".to_string() }.error_type(), "api_error");
        assert_eq!(VertexAIError::ResponseParsing("".to_string()).error_type(), "parsing");
        assert_eq!(VertexAIError::UnsupportedModel("".to_string()).error_type(), "unsupported_model");
        assert_eq!(VertexAIError::UnsupportedFeature("".to_string()).error_type(), "unsupported_feature");
        assert_eq!(VertexAIError::InvalidRequest("".to_string()).error_type(), "invalid_request");
        assert_eq!(VertexAIError::RateLimitExceeded.error_type(), "rate_limit");
        assert_eq!(VertexAIError::QuotaExceeded("".to_string()).error_type(), "quota_exceeded");
        assert_eq!(VertexAIError::TokenLimitExceeded("".to_string()).error_type(), "token_limit");
        assert_eq!(VertexAIError::ContextLengthExceeded { max: 0, actual: 0 }.error_type(), "context_length");
        assert_eq!(VertexAIError::ContentFiltered.error_type(), "content_filtered");
        assert_eq!(VertexAIError::ServiceUnavailable.error_type(), "service_unavailable");
        assert_eq!(VertexAIError::Timeout(0).error_type(), "timeout");
        assert_eq!(VertexAIError::FeatureDisabled("".to_string()).error_type(), "feature_disabled");
        assert_eq!(VertexAIError::Other("".to_string()).error_type(), "other");
    }

    #[test]
    fn test_vertex_error_is_retryable() {
        assert!(VertexAIError::Network("".to_string()).is_retryable());
        assert!(VertexAIError::RateLimitExceeded.is_retryable());
        assert!(VertexAIError::ServiceUnavailable.is_retryable());
        assert!(VertexAIError::Timeout(30).is_retryable());
        assert!(VertexAIError::ApiError { status_code: 500, message: "".to_string() }.is_retryable());
        assert!(VertexAIError::ApiError { status_code: 503, message: "".to_string() }.is_retryable());

        assert!(!VertexAIError::Authentication("".to_string()).is_retryable());
        assert!(!VertexAIError::Configuration("".to_string()).is_retryable());
        assert!(!VertexAIError::InvalidRequest("".to_string()).is_retryable());
        assert!(!VertexAIError::ApiError { status_code: 400, message: "".to_string() }.is_retryable());
    }

    #[test]
    fn test_vertex_error_retry_delay() {
        assert_eq!(VertexAIError::RateLimitExceeded.retry_delay(), Some(60));
        assert_eq!(VertexAIError::ServiceUnavailable.retry_delay(), Some(30));
        assert_eq!(VertexAIError::Network("".to_string()).retry_delay(), Some(5));
        assert_eq!(VertexAIError::Timeout(30).retry_delay(), Some(5));
        assert_eq!(VertexAIError::ApiError { status_code: 500, message: "".to_string() }.retry_delay(), Some(10));
        assert_eq!(VertexAIError::Authentication("".to_string()).retry_delay(), None);
    }

    #[test]
    fn test_vertex_error_status_code() {
        assert_eq!(VertexAIError::ApiError { status_code: 500, message: "".to_string() }.status_code(), Some(500));
        assert_eq!(VertexAIError::RateLimitExceeded.status_code(), Some(429));
        assert_eq!(VertexAIError::ServiceUnavailable.status_code(), Some(503));
        assert_eq!(VertexAIError::Authentication("".to_string()).status_code(), None);
    }

    #[test]
    fn test_vertex_error_http_status() {
        assert_eq!(VertexAIError::ApiError { status_code: 500, message: "".to_string() }.http_status(), 500);
        assert_eq!(VertexAIError::RateLimitExceeded.http_status(), 429);
        assert_eq!(VertexAIError::ServiceUnavailable.http_status(), 503);
        assert_eq!(VertexAIError::Authentication("".to_string()).http_status(), 0);
    }

    #[test]
    fn test_vertex_error_factory_methods() {
        let err = VertexAIError::not_supported("vision");
        assert!(matches!(err, VertexAIError::UnsupportedFeature(_)));

        let err = VertexAIError::authentication_failed("bad key");
        assert!(matches!(err, VertexAIError::Authentication(_)));

        let err = VertexAIError::rate_limited(Some(60));
        assert!(matches!(err, VertexAIError::RateLimitExceeded));

        let err = VertexAIError::network_error("timeout");
        assert!(matches!(err, VertexAIError::Network(_)));

        let err = VertexAIError::parsing_error("invalid json");
        assert!(matches!(err, VertexAIError::ResponseParsing(_)));

        let err = VertexAIError::not_implemented("feature");
        assert!(matches!(err, VertexAIError::UnsupportedFeature(_)));
    }

    #[test]
    fn test_vertex_error_from_serde_error() {
        let json_err = serde_json::from_str::<String>("invalid").unwrap_err();
        let err: VertexAIError = json_err.into();
        assert!(matches!(err, VertexAIError::ResponseParsing(_)));
    }
}
