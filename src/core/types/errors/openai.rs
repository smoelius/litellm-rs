//! OpenAI provider error types

use super::litellm::LiteLLMError;
use super::traits::ProviderErrorTrait;

/// OpenAI provider error types
#[derive(Debug, thiserror::Error)]
pub enum OpenAIError {
    #[error("OpenAI API error: {message}")]
    ApiError {
        message: String,
        status_code: Option<u16>,
        error_type: Option<String>,
    },

    #[error("Authentication failed: {0}")]
    Authentication(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    #[error("Model '{model}' not found")]
    ModelNotFound { model: String },

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Parsing error: {0}")]
    Parsing(String),

    #[error("Streaming error: {0}")]
    Streaming(String),

    #[error("Unsupported feature: {0}")]
    UnsupportedFeature(String),

    #[error("Feature not implemented: {0}")]
    NotImplemented(String),

    #[error("Other OpenAI error: {0}")]
    Other(String),
}

impl OpenAIError {
    pub fn not_supported(feature: &str) -> Self {
        Self::UnsupportedFeature(format!("{} is not supported", feature))
    }
}

impl ProviderErrorTrait for OpenAIError {
    fn error_type(&self) -> &'static str {
        match self {
            Self::ApiError { .. } => "api_error",
            Self::Authentication(_) => "authentication_error",
            Self::RateLimit(_) => "rate_limit_error",
            Self::ModelNotFound { .. } => "model_not_found",
            Self::InvalidRequest(_) => "invalid_request_error",
            Self::Network(_) => "network_error",
            Self::Timeout(_) => "timeout_error",
            Self::Parsing(_) => "parsing_error",
            Self::Streaming(_) => "streaming_error",
            Self::UnsupportedFeature(_) => "unsupported_feature",
            Self::NotImplemented(_) => "not_implemented",
            Self::Other(_) => "other_error",
        }
    }

    fn is_retryable(&self) -> bool {
        match self {
            Self::Network(_) | Self::Timeout(_) | Self::Streaming(_) => true,
            Self::ApiError {
                status_code: Some(code),
                ..
            } => matches!(*code, 429 | 500 | 502 | 503 | 504),
            Self::RateLimit(_) => true,
            _ => false,
        }
    }

    fn retry_delay(&self) -> Option<u64> {
        match self {
            Self::RateLimit(_) => Some(60),
            Self::Network(_) | Self::Timeout(_) => Some(1),
            Self::ApiError {
                status_code: Some(429),
                ..
            } => Some(60),
            Self::ApiError {
                status_code: Some(code),
                ..
            } if *code >= 500 => Some(5),
            _ => None,
        }
    }

    fn http_status(&self) -> u16 {
        match self {
            Self::Authentication(_) => 401,
            Self::RateLimit(_) => 429,
            Self::ModelNotFound { .. } => 404,
            Self::InvalidRequest(_) => 400,
            Self::UnsupportedFeature(_) => 405,
            Self::NotImplemented(_) => 501,
            Self::ApiError {
                status_code: Some(code),
                ..
            } => *code,
            Self::Network(_) | Self::Timeout(_) => 503,
            _ => 500,
        }
    }

    fn not_supported(feature: &str) -> Self {
        Self::UnsupportedFeature(format!(
            "Feature '{}' is not supported by OpenAI provider",
            feature
        ))
    }

    fn authentication_failed(reason: &str) -> Self {
        Self::Authentication(reason.to_string())
    }

    fn rate_limited(retry_after: Option<u64>) -> Self {
        let message = if let Some(seconds) = retry_after {
            format!("Rate limit exceeded. Retry after {} seconds", seconds)
        } else {
            "Rate limit exceeded".to_string()
        };
        Self::RateLimit(message)
    }

    fn network_error(details: &str) -> Self {
        Self::Network(details.to_string())
    }

    fn parsing_error(details: &str) -> Self {
        Self::Parsing(details.to_string())
    }

    fn not_implemented(feature: &str) -> Self {
        Self::NotImplemented(format!("Feature '{}' not yet implemented", feature))
    }
}

impl From<reqwest::Error> for OpenAIError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            Self::Timeout(err.to_string())
        } else if err.is_connect() || err.is_request() {
            Self::Network(err.to_string())
        } else {
            Self::Other(err.to_string())
        }
    }
}

impl From<serde_json::Error> for OpenAIError {
    fn from(err: serde_json::Error) -> Self {
        Self::Parsing(err.to_string())
    }
}

impl From<OpenAIError> for LiteLLMError {
    fn from(err: OpenAIError) -> Self {
        Self::provider_error_with_source("openai", err.to_string(), Box::new(err))
    }
}

/// Result type alias
pub type OpenAIResult<T> = Result<T, OpenAIError>;
