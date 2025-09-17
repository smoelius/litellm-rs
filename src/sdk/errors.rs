//! Error handling

use thiserror::Error;

/// Error
#[derive(Error, Debug)]
pub enum SDKError {
    /// Provider not found
    #[error("Provider not found: {0}")]
    ProviderNotFound(String),

    /// Default
    #[error("No default provider configured")]
    NoDefaultProvider,

    /// Error
    #[error("Provider error: {0}")]
    ProviderError(String),

    /// Configuration
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Error
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Error
    #[error("Authentication error: {0}")]
    AuthError(String),

    /// Error
    #[error("Rate limit exceeded: {0}")]
    RateLimitError(String),

    /// Model
    #[error("Model not found: {0}")]
    ModelNotFound(String),

    /// Feature not supported
    #[error("Feature not supported: {0}")]
    NotSupported(String),

    /// Unsupported provider
    #[error("Unsupported provider: {0}")]
    UnsupportedProvider(String),

    /// Error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// Error
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    /// Request
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Error
    #[error("Internal error: {0}")]
    Internal(String),

    /// Error
    #[error("API error: {0}")]
    ApiError(String),

    /// Error
    #[error("Parse error: {0}")]
    ParseError(String),
}

/// Error
impl From<crate::utils::error::GatewayError> for SDKError {
    fn from(error: crate::utils::error::GatewayError) -> Self {
        match error {
            crate::utils::error::GatewayError::Unauthorized(msg) => SDKError::AuthError(msg),
            crate::utils::error::GatewayError::NotFound(msg) => SDKError::ModelNotFound(msg),
            crate::utils::error::GatewayError::BadRequest(msg) => SDKError::InvalidRequest(msg),
            crate::utils::error::GatewayError::RateLimit(msg) => SDKError::RateLimitError(msg),
            crate::utils::error::GatewayError::ProviderUnavailable(msg) => {
                SDKError::ProviderError(msg)
            }
            crate::utils::error::GatewayError::Internal(msg) => SDKError::Internal(msg),
            crate::utils::error::GatewayError::Network(msg) => SDKError::NetworkError(msg),
            crate::utils::error::GatewayError::Validation(msg) => SDKError::InvalidRequest(msg),
            crate::utils::error::GatewayError::Parsing(msg) => SDKError::Internal(msg),
            // Handle
            _ => SDKError::Internal(error.to_string()),
        }
    }
}

// Temporarily disabled old provider error mapping
/*
impl From<crate::core::providers::ProviderError> for SDKError {
    fn from(error: crate::core::providers::ProviderError) -> Self {
        match error {
            crate::core::providers::ProviderError::Authentication(msg) => SDKError::AuthError(msg),
            crate::core::providers::ProviderError::RateLimit(msg) => SDKError::RateLimitError(msg),
            crate::core::providers::ProviderError::RateLimited(msg) => {
                SDKError::RateLimitError(msg)
            }
            crate::core::providers::ProviderError::ModelNotFound(msg) => {
                SDKError::ModelNotFound(msg)
            }
            crate::core::providers::ProviderError::InvalidRequest(msg) => {
                SDKError::InvalidRequest(msg)
            }
            crate::core::providers::ProviderError::Unavailable(msg) => SDKError::ProviderError(msg),
            crate::core::providers::ProviderError::Network(msg) => SDKError::NetworkError(msg),
            crate::core::providers::ProviderError::Parsing(msg) => SDKError::Internal(msg),
            crate::core::providers::ProviderError::Timeout(msg) => SDKError::NetworkError(msg),
            crate::core::providers::ProviderError::Other(msg) => SDKError::Internal(msg),
            crate::core::providers::ProviderError::Unknown(msg) => SDKError::Internal(msg),
        }
    }
}
*/

/// SDK result type
pub type Result<T> = std::result::Result<T, SDKError>;

impl SDKError {
    /// Error
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            SDKError::NetworkError(_) | SDKError::RateLimitError(_) | SDKError::ProviderError(_)
        )
    }

    /// Error
    pub fn is_auth_error(&self) -> bool {
        matches!(self, SDKError::AuthError(_))
    }

    /// Configuration
    pub fn is_config_error(&self) -> bool {
        matches!(
            self,
            SDKError::ConfigError(_) | SDKError::ProviderNotFound(_) | SDKError::NoDefaultProvider
        )
    }
}
