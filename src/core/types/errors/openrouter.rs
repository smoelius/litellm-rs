//! OpenRouter provider error types

use super::litellm::LiteLLMError;
use super::openai::OpenAIError;
use super::traits::ProviderErrorTrait;

/// OpenRouter provider error types
#[derive(Debug, thiserror::Error)]
pub enum OpenRouterError {
    #[error("OpenRouter API error: {message}")]
    ApiError {
        message: String,
        status_code: Option<u16>,
    },

    #[error("Authentication failed: {0}")]
    Authentication(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    #[error("Model '{0}' not found")]
    ModelNotFound(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Parsing error: {0}")]
    Parsing(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Transformation error: {0}")]
    Transformation(String),

    #[error("Unsupported feature: {0}")]
    UnsupportedFeature(String),

    #[error("Feature not implemented: {0}")]
    NotImplemented(String),
}

impl From<serde_json::Error> for OpenRouterError {
    fn from(err: serde_json::Error) -> Self {
        Self::Parsing(err.to_string())
    }
}

impl From<OpenAIError> for OpenRouterError {
    fn from(err: OpenAIError) -> Self {
        Self::Transformation(err.to_string())
    }
}

impl ProviderErrorTrait for OpenRouterError {
    fn error_type(&self) -> &'static str {
        match self {
            Self::ApiError { .. } => "api_error",
            Self::Authentication(_) => "authentication_error",
            Self::RateLimit(_) => "rate_limit_error",
            Self::ModelNotFound(_) => "model_not_found",
            Self::InvalidRequest(_) => "invalid_request_error",
            Self::Network(_) => "network_error",
            Self::Timeout(_) => "timeout_error",
            Self::Parsing(_) => "parsing_error",
            Self::Configuration(_) => "configuration_error",
            Self::Transformation(_) => "transformation_error",
            Self::UnsupportedFeature(_) => "unsupported_feature",
            Self::NotImplemented(_) => "not_implemented",
        }
    }

    fn is_retryable(&self) -> bool {
        match self {
            Self::Network(_) | Self::Timeout(_) => true,
            Self::RateLimit(_) => true,
            Self::ApiError {
                status_code: Some(code),
                ..
            } => matches!(*code, 429 | 500 | 502 | 503 | 504),
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
            Self::ModelNotFound(_) => 404,
            Self::InvalidRequest(_) => 400,
            Self::Configuration(_) => 400,
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
            "Feature '{}' is not supported by OpenRouter",
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

impl From<OpenRouterError> for LiteLLMError {
    fn from(err: OpenRouterError) -> Self {
        Self::provider_error_with_source("openrouter", err.to_string(), Box::new(err))
    }
}

/// Result type alias
pub type OpenRouterResult<T> = Result<T, OpenRouterError>;
