//! Error types and traits for unified error handling across providers
//!
//! This module provides a hierarchical error system with provider-agnostic interfaces.
//!
//! ## Architecture Overview
//!
//! The error system is organized into multiple layers:
//!
//! ## 1. Trait Layer (this file)
//! - `ProviderErrorTrait`: Common interface for all provider errors
//! - Provides unified methods:
//!   - is_retryable(): Whether the error can be retried
//!   - retry_delay(): Recommended retry delay duration
//!   - http_status(): HTTP status code mapping
//!   - Factory methods: not_supported(), authentication_failed() etc
//!
//! ## 2. Unified Implementation Layer
//! - `ProviderError`: Single concrete error type for all providers
//! - Common variants:
//!   - Authentication: Authentication failed
//!   - RateLimit: Rate limit exceeded with retry information
//!   - ModelNotFound: Requested model does not exist
//!   - InvalidRequest: Malformed or invalid request
//!   - Network: Network connectivity issues
//!   - Timeout: Request timeout exceeded
//!   - ApiError: Provider-specific API errors
//!   - ServiceUnavailable: Service temporarily unavailable
//!   - QuotaExceeded: Usage quota exceeded
//!   - NotSupported: Feature not supported by provider
//!   - ContentFiltered: Content blocked by safety filters
//!
//! ## Usage
//! ```rust,ignore
//! // All providers use unified ProviderError
//! use litellm_rs::ProviderError;
//!
//! // Create errors using factory methods
//! let err = ProviderError::authentication("openai", "Invalid API key");
//! let err = ProviderError::rate_limit("anthropic", Some(60));
//!
//! // Check error properties
//! if err.is_retryable() {
//!     if let Some(delay) = err.retry_delay() {
//!         println!("Retry after {} seconds", delay);
//!     }
//! }
//! ```
//!
//! ## Design Principles
//! 1. **Unified Interface**: Single error type eliminates conversion overhead
//! 2. **Extensible**: Define interfaces through traits for future expansion
//! 3. **Zero-cost abstraction**: Use static dispatch, no runtime overhead
//! 4. **Rich Context**: Structured error information with provider-specific details

/// Common trait for all provider error types
///
/// Provides a unified interface for error handling across different AI providers.
pub trait ProviderErrorTrait: std::error::Error + Send + Sync + 'static {
    /// Get the error type as a string identifier
    fn error_type(&self) -> &'static str;

    /// Check if this error can be retried
    fn is_retryable(&self) -> bool;

    /// Get the recommended retry delay in seconds
    fn retry_delay(&self) -> Option<u64>;

    /// Get the appropriate HTTP status code for this error
    fn http_status(&self) -> u16;

    /// Create a "feature not supported" error
    fn not_supported(feature: &str) -> Self;

    /// Create an authentication failure error
    fn authentication_failed(reason: &str) -> Self;

    /// Create a rate limit error with optional retry delay
    fn rate_limited(retry_after: Option<u64>) -> Self;

    /// Create a network error
    fn network_error(details: &str) -> Self;

    /// Create a parsing/serialization error
    fn parsing_error(details: &str) -> Self;

    /// Create a "feature not implemented" error
    fn not_implemented(feature: &str) -> Self;
}

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

    /// Configuration
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

/// Error
#[derive(Debug, thiserror::Error)]
pub enum RoutingError {
    #[error("No healthy providers available")]
    NoHealthyProviders,

    #[error("No suitable provider found for request")]
    NoSuitableProvider,

    #[error("All providers failed")]
    AllProvidersFailed,

    #[error("Provider '{provider}' not found")]
    ProviderNotFound { provider: String },

    #[error("Invalid routing strategy: {strategy}")]
    InvalidStrategy { strategy: String },

    #[error("Route selection failed: {reason}")]
    SelectionFailed { reason: String },

    #[error("Circuit breaker is open for provider '{provider}'")]
    CircuitBreakerOpen { provider: String },

    #[error("Load balancing failed: {reason}")]
    LoadBalancingFailed { reason: String },
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

/// Configuration
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Missing required field: {field}")]
    MissingField { field: String },

    #[error("Invalid value for field '{field}': {value}")]
    InvalidValue { field: String, value: String },

    #[error("Configuration file not found: {path}")]
    FileNotFound { path: String },

    #[error("Failed to read configuration file: {path}")]
    ReadError { path: String },

    #[error("Failed to parse configuration: {reason}")]
    ParseError { reason: String },

    #[error("Unsupported configuration format")]
    UnsupportedFormat,

    #[error("Configuration validation failed: {reason}")]
    ValidationFailed { reason: String },

    #[error("Environment variable error: {var}")]
    EnvVarError { var: String },
}

/// Error
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
    /// Create
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
            } => {
                matches!(*code, 429 | 500 | 502 | 503 | 504)
            }
            Self::RateLimit(_) => true,
            _ => false,
        }
    }

    fn retry_delay(&self) -> Option<u64> {
        match self {
            Self::RateLimit(_) => Some(60),                 // 1 minute
            Self::Network(_) | Self::Timeout(_) => Some(1), // 1 second
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

/// Error
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

/// Error
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

    /// Error
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Network(_) | Self::Timeout { .. } | Self::ServiceUnavailable(_) => true,
            Self::RateLimit { .. } => true,
            Self::Provider { .. } => true, // Error
            Self::Internal(_) => true,     // Error
            _ => false,
        }
    }

    /// Get
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

impl From<OpenAIError> for LiteLLMError {
    fn from(err: OpenAIError) -> Self {
        Self::provider_error_with_source("openai", err.to_string(), Box::new(err))
    }
}

/// Error
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
            } => {
                matches!(*code, 429 | 500 | 502 | 503 | 504)
            }
            _ => false,
        }
    }

    fn retry_delay(&self) -> Option<u64> {
        match self {
            Self::RateLimit(_) => Some(60),                 // 1 minute
            Self::Network(_) | Self::Timeout(_) => Some(1), // 1 second
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
pub type LiteLLMResult<T> = Result<T, LiteLLMError>;
pub type RoutingResult<T> = Result<T, RoutingError>;
pub type ConfigResult<T> = Result<T, ConfigError>;
pub type OpenAIResult<T> = Result<T, OpenAIError>;
pub type OpenRouterResult<T> = Result<T, OpenRouterError>;
