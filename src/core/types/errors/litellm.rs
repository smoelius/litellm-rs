//! Main LiteLLM error types

use super::config::ConfigError;
use super::routing::RoutingError;

/// Top-level error type for the LiteLLM gateway
#[derive(Debug, thiserror::Error)]
pub enum LiteLLMError {
    /// Provider-specific error
    #[error("Provider error ({provider}): {message}")]
    Provider {
        provider: String,
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Request routing error
    #[error("Routing error: {0}")]
    Routing(RoutingError),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(ConfigError),

    /// Authentication failure
    #[error("Authentication error: {0}")]
    Authentication(String),

    /// Authorization/permission denied
    #[error("Authorization error: {0}")]
    Authorization(String),

    /// Request validation error
    #[error("Validation error: {field}: {message}")]
    Validation { field: String, message: String },

    /// Rate limit exceeded
    #[error("Rate limit exceeded: {message}")]
    RateLimit {
        message: String,
        retry_after: Option<u64>,
    },

    /// Network connectivity error
    #[error("Network error: {0}")]
    Network(String),

    /// Operation timeout
    #[error("Operation timed out: {operation}")]
    Timeout { operation: String },

    /// Response parsing error
    #[error("Parsing error: {0}")]
    Parsing(String),

    /// JSON serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Cache operation error
    #[error("Cache error: {0}")]
    Cache(String),

    /// Internal system error
    #[error("Internal error: {0}")]
    Internal(String),

    /// Service unavailable
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    /// Resource not found
    #[error("Not found: {resource}")]
    NotFound { resource: String },

    /// Unsupported operation
    #[error("Unsupported operation: {operation}")]
    UnsupportedOperation { operation: String },
}

impl From<RoutingError> for LiteLLMError {
    fn from(err: RoutingError) -> Self {
        LiteLLMError::Routing(err)
    }
}

impl From<ConfigError> for LiteLLMError {
    fn from(err: ConfigError) -> Self {
        LiteLLMError::Configuration(err)
    }
}

impl LiteLLMError {
    pub fn provider_error(provider: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Provider {
            provider: provider.into(),
            message: message.into(),
            source: None,
        }
    }

    pub fn provider_error_with_source(
        provider: impl Into<String>,
        message: impl Into<String>,
        source: Box<dyn std::error::Error + Send + Sync>,
    ) -> Self {
        Self::Provider {
            provider: provider.into(),
            message: message.into(),
            source: Some(source),
        }
    }

    pub fn authentication(message: impl Into<String>) -> Self {
        Self::Authentication(message.into())
    }

    pub fn authorization(message: impl Into<String>) -> Self {
        Self::Authorization(message.into())
    }

    pub fn validation(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Validation {
            field: field.into(),
            message: message.into(),
        }
    }

    pub fn rate_limit(message: impl Into<String>, retry_after: Option<u64>) -> Self {
        Self::RateLimit {
            message: message.into(),
            retry_after,
        }
    }

    pub fn network(message: impl Into<String>) -> Self {
        Self::Network(message.into())
    }

    pub fn timeout(operation: impl Into<String>) -> Self {
        Self::Timeout {
            operation: operation.into(),
        }
    }

    pub fn parsing(message: impl Into<String>) -> Self {
        Self::Parsing(message.into())
    }

    pub fn cache(message: impl Into<String>) -> Self {
        Self::Cache(message.into())
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal(message.into())
    }

    pub fn service_unavailable(message: impl Into<String>) -> Self {
        Self::ServiceUnavailable(message.into())
    }

    pub fn not_found(resource: impl Into<String>) -> Self {
        Self::NotFound {
            resource: resource.into(),
        }
    }

    pub fn unsupported_operation(operation: impl Into<String>) -> Self {
        Self::UnsupportedOperation {
            operation: operation.into(),
        }
    }
}

/// HTTP status code mapping
impl LiteLLMError {
    pub fn to_http_status(&self) -> u16 {
        match self {
            Self::Authentication(_) => 401,
            Self::Authorization(_) => 403,
            Self::NotFound { .. } => 404,
            Self::UnsupportedOperation { .. } => 405,
            Self::RateLimit { .. } => 429,
            Self::Validation { .. } => 400,
            Self::Configuration(_) => 400,
            Self::Network(_) | Self::ServiceUnavailable(_) => 503,
            Self::Timeout { .. } => 504,
            Self::Provider { .. }
            | Self::Routing(_)
            | Self::Internal(_)
            | Self::Parsing(_)
            | Self::Serialization(_)
            | Self::Cache(_) => 500,
        }
    }

    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Network(_)
                | Self::Timeout { .. }
                | Self::ServiceUnavailable(_)
                | Self::RateLimit { .. }
                | Self::Provider { .. }
                | Self::Internal(_)
        )
    }

    /// Get retry delay
    pub fn retry_delay(&self) -> Option<u64> {
        match self {
            Self::RateLimit { retry_after, .. } => *retry_after,
            Self::Network(_) | Self::Timeout { .. } => Some(1),
            Self::ServiceUnavailable(_) => Some(5),
            Self::Internal(_) => Some(1),
            _ => None,
        }
    }
}

/// Result type alias
pub type LiteLLMResult<T> = Result<T, LiteLLMError>;
