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
