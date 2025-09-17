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
