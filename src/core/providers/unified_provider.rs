//! Unified Provider Error Handling
//!
//! Single error type for all providers - optimized design for simplicity and performance
//!
//! This module provides a unified error handling system for all AI providers.
//!
//! ## Core Components
//!
//! ### `ProviderError` Enum
//! A comprehensive error type that covers all possible failure scenarios across different AI providers.
//!
//! | Variant | Purpose | HTTP Status | Retryable |
//! |------|------|------------|--------|
//! | Authentication | Authentication failed | 401 | No |
//! | RateLimit | Rate limit exceeded | 429 | Yes (after delay) |
//! | ModelNotFound | Model not found | 404 | No |
//! | InvalidRequest | Invalid request | 400 | No |
//! | Network | Network error | 500 | Yes |
//! | Timeout | Timeout | 408 | Yes |
//! | Internal | Internal error | 500 | Yes |
//! | ServiceUnavailable | Service unavailable | 503 | Yes |
//! | QuotaExceeded | Quota exceeded | 402 | No |
//! | NotSupported | Feature not supported | 501 | No |
//! | Other | Other error | 500 | No |
//!
//! ## Usage
//!
//! ```rust,ignore
//! use litellm_rs::ProviderError;
//!
//! // 1. Direct construction
//! let err = ProviderError::Authentication {
//!     provider: "openai",
//!     message: "Invalid API key".to_string()
//! };
//!
//! // 2. Use factory methods (preferred)
//! let err = ProviderError::authentication("openai", "Invalid API key");
//! let err = ProviderError::rate_limit("anthropic", Some(60));
//!
//! // 3. Check error properties
//! if err.is_retryable() {
//!     if let Some(delay) = err.retry_delay() {
//!         println!("Retry after {} seconds", delay);
//!     }
//! }
//! ```
//!
//! ## Migration Guide
//!
//! For migrating from provider-specific error types:
//!
//! ```rust,ignore
//! // Old code
//! // pub enum MyProviderError { ... }
//!
//! // New code - use unified error type
//! use litellm_rs::ProviderError;
//! pub type MyProviderError = ProviderError;
//! ```
//!
//! ## Design Advantages
//!
//! - **Unified Interface**: Single error type for all providers eliminates conversion overhead
//! - **Rich Context**: Structured error information with provider-specific details
//! - **Retry Logic**: Built-in retry determination and delay calculation
//! - **HTTP Mapping**: Automatic HTTP status code mapping for web APIs
//! - **Performance**: Zero-cost abstractions with compile-time optimization

/// Unified provider error type - single error for all providers
/// This eliminates the need for error type conversion and simplifies the architecture
#[derive(Debug, Clone, thiserror::Error)]
pub enum ProviderError {
    #[error("Authentication failed for {provider}: {message}")]
    Authentication {
        provider: &'static str,
        message: String,
    },

    #[error("Rate limit exceeded for {provider}: {message}")]
    RateLimit {
        provider: &'static str,
        message: String,
        retry_after: Option<u64>,
        /// Requests per minute limit
        rpm_limit: Option<u32>,
        /// Tokens per minute limit  
        tpm_limit: Option<u32>,
        /// Current usage level
        current_usage: Option<f64>,
    },

    #[error("Quota exceeded for {provider}: {message}")]
    QuotaExceeded {
        provider: &'static str,
        message: String,
    },

    #[error("Model '{model}' not found for {provider}")]
    ModelNotFound {
        provider: &'static str,
        model: String,
    },

    #[error("Invalid request for {provider}: {message}")]
    InvalidRequest {
        provider: &'static str,
        message: String,
    },

    #[error("Network error for {provider}: {message}")]
    Network {
        provider: &'static str,
        message: String,
    },

    #[error("Provider {provider} is unavailable: {message}")]
    ProviderUnavailable {
        provider: &'static str,
        message: String,
    },

    #[error("Feature '{feature}' not supported by {provider}")]
    NotSupported {
        provider: &'static str,
        feature: String,
    },

    #[error("Feature '{feature}' not implemented for {provider}")]
    NotImplemented {
        provider: &'static str,
        feature: String,
    },

    #[error("Configuration error for {provider}: {message}")]
    Configuration {
        provider: &'static str,
        message: String,
    },

    #[error("Serialization error for {provider}: {message}")]
    Serialization {
        provider: &'static str,
        message: String,
    },

    #[error("Timeout for {provider}: {message}")]
    Timeout {
        provider: &'static str,
        message: String,
    },

    // Enhanced error variants based on ultrathink analysis
    /// Context length exceeded with structured limits (VertexAI pattern)
    #[error("Context length exceeded for {provider}: max {max} tokens, got {actual} tokens")]
    ContextLengthExceeded {
        provider: &'static str,
        max: usize,
        actual: usize,
    },

    /// Content filtered by safety systems (VertexAI/OpenAI pattern)
    #[error("Content filtered by {provider} safety systems: {reason}")]
    ContentFiltered {
        provider: &'static str,
        reason: String,
        /// Policy categories that were violated
        policy_violations: Option<Vec<String>>,
        /// Whether this might succeed with prompt modification
        potentially_retryable: Option<bool>,
    },

    /// API error with status code (Universal pattern)
    #[error("API error for {provider} (status {status}): {message}")]
    ApiError {
        provider: &'static str,
        status: u16,
        message: String,
    },

    /// Token limit exceeded (separate from context length)
    #[error("Token limit exceeded for {provider}: {message}")]
    TokenLimitExceeded {
        provider: &'static str,
        message: String,
    },

    /// Feature disabled by provider (VertexAI pattern)
    #[error("Feature disabled for {provider}: {feature}")]
    FeatureDisabled {
        provider: &'static str,
        feature: String,
    },

    /// Azure deployment specific error
    #[error("Azure deployment error for {deployment}: {message}")]
    DeploymentError {
        provider: &'static str,
        deployment: String,
        message: String,
    },

    /// Response parsing error (universal pattern)
    #[error("Failed to parse {provider} response: {message}")]
    ResponseParsing {
        provider: &'static str,
        message: String,
    },

    /// Multi-provider routing error (OpenRouter pattern)
    #[error("Routing error from {provider}: tried {attempted_providers:?}, final error: {message}")]
    RoutingError {
        provider: &'static str,
        attempted_providers: Vec<String>,
        message: String,
    },

    /// Transformation error between provider formats (OpenRouter pattern)
    #[error("Transformation error for {provider}: from {from_format} to {to_format}: {message}")]
    TransformationError {
        provider: &'static str,
        from_format: String,
        to_format: String,
        message: String,
    },

    /// Async operation cancelled (Rust async pattern)
    #[error("Operation cancelled for {provider}: {operation_type}")]
    Cancelled {
        provider: &'static str,
        operation_type: String,
        /// Reason for cancellation
        cancellation_reason: Option<String>,
    },

    /// Streaming operation error (SSE/WebSocket pattern)
    #[error("Streaming error for {provider}: {stream_type} at position {position:?}")]
    Streaming {
        provider: &'static str,
        /// Type of stream (chat, completion, etc.)
        stream_type: String,
        /// Position in stream where error occurred
        position: Option<u64>,
        /// Last valid chunk received
        last_chunk: Option<String>,
        /// Error message
        message: String,
    },

    #[error("{provider} error: {message}")]
    Other {
        provider: &'static str,
        message: String,
    },
}

impl ProviderError {
    /// Create authentication error
    pub fn authentication(provider: &'static str, message: impl Into<String>) -> Self {
        Self::Authentication {
            provider,
            message: message.into(),
        }
    }

    /// Create rate limit error
    pub fn rate_limit(provider: &'static str, retry_after: Option<u64>) -> Self {
        Self::RateLimit {
            provider,
            message: match retry_after {
                Some(seconds) => format!("Rate limit exceeded. Retry after {} seconds", seconds),
                None => "Rate limit exceeded".to_string(),
            },
            retry_after,
            rpm_limit: None,
            tpm_limit: None,
            current_usage: None,
        }
    }

    /// Create enhanced rate limit error with usage details
    pub fn rate_limit_with_limits(
        provider: &'static str,
        retry_after: Option<u64>,
        rpm_limit: Option<u32>,
        tpm_limit: Option<u32>,
        current_usage: Option<f64>,
    ) -> Self {
        let message = match (rpm_limit, tpm_limit) {
            (Some(rpm), Some(tpm)) => {
                format!("Rate limit exceeded: {}RPM, {}TPM limits reached", rpm, tpm)
            }
            (Some(rpm), None) => format!("Rate limit exceeded: {}RPM limit reached", rpm),
            (None, Some(tpm)) => format!("Rate limit exceeded: {}TPM limit reached", tpm),
            (None, None) => "Rate limit exceeded".to_string(),
        };

        Self::RateLimit {
            provider,
            message,
            retry_after,
            rpm_limit,
            tpm_limit,
            current_usage,
        }
    }

    /// Create quota exceeded error
    pub fn quota_exceeded(provider: &'static str, message: impl Into<String>) -> Self {
        Self::QuotaExceeded {
            provider,
            message: message.into(),
        }
    }

    /// Create simple rate limit error (convenience method)
    pub fn rate_limit_simple(provider: &'static str, message: impl Into<String>) -> Self {
        Self::RateLimit {
            provider,
            message: message.into(),
            retry_after: None,
            rpm_limit: None,
            tpm_limit: None,
            current_usage: None,
        }
    }

    /// Create rate limit error with retry_after only
    pub fn rate_limit_with_retry(
        provider: &'static str,
        message: impl Into<String>,
        retry_after: Option<u64>,
    ) -> Self {
        Self::RateLimit {
            provider,
            message: message.into(),
            retry_after,
            rpm_limit: None,
            tpm_limit: None,
            current_usage: None,
        }
    }

    /// Create model not found error
    pub fn model_not_found(provider: &'static str, model: impl Into<String>) -> Self {
        Self::ModelNotFound {
            provider,
            model: model.into(),
        }
    }

    /// Create invalid request error
    pub fn invalid_request(provider: &'static str, message: impl Into<String>) -> Self {
        Self::InvalidRequest {
            provider,
            message: message.into(),
        }
    }

    /// Create network error
    pub fn network(provider: &'static str, message: impl Into<String>) -> Self {
        Self::Network {
            provider,
            message: message.into(),
        }
    }

    /// Create provider unavailable error
    pub fn provider_unavailable(provider: &'static str, message: impl Into<String>) -> Self {
        Self::ProviderUnavailable {
            provider,
            message: message.into(),
        }
    }

    /// Create not supported error
    pub fn not_supported(provider: &'static str, feature: impl Into<String>) -> Self {
        Self::NotSupported {
            provider,
            feature: feature.into(),
        }
    }

    /// Create not implemented error
    pub fn not_implemented(provider: &'static str, feature: impl Into<String>) -> Self {
        Self::NotImplemented {
            provider,
            feature: feature.into(),
        }
    }

    /// Create configuration error
    pub fn configuration(provider: &'static str, message: impl Into<String>) -> Self {
        Self::Configuration {
            provider,
            message: message.into(),
        }
    }

    /// Create serialization error
    pub fn serialization(provider: &'static str, message: impl Into<String>) -> Self {
        Self::Serialization {
            provider,
            message: message.into(),
        }
    }

    /// Create timeout error
    pub fn timeout(provider: &'static str, message: impl Into<String>) -> Self {
        Self::Timeout {
            provider,
            message: message.into(),
        }
    }

    /// Create initialization error (provider failed to start)
    pub fn initialization(provider: &'static str, message: impl Into<String>) -> Self {
        Self::Network {
            provider,
            message: format!("Initialization failed: {}", message.into()),
        }
    }

    // Enhanced factory methods for new error variants

    /// Create context length exceeded error with structured data
    pub fn context_length_exceeded(provider: &'static str, max: usize, actual: usize) -> Self {
        Self::ContextLengthExceeded {
            provider,
            max,
            actual,
        }
    }

    /// Create API error with status code
    pub fn api_error(provider: &'static str, status: u16, message: impl Into<String>) -> Self {
        Self::ApiError {
            provider,
            status,
            message: message.into(),
        }
    }

    /// Create token limit exceeded error
    pub fn token_limit_exceeded(provider: &'static str, message: impl Into<String>) -> Self {
        Self::TokenLimitExceeded {
            provider,
            message: message.into(),
        }
    }

    /// Create feature disabled error
    pub fn feature_disabled(provider: &'static str, feature: impl Into<String>) -> Self {
        Self::FeatureDisabled {
            provider,
            feature: feature.into(),
        }
    }

    /// Create Azure deployment error
    pub fn deployment_error(deployment: impl Into<String>, message: impl Into<String>) -> Self {
        Self::DeploymentError {
            provider: "azure",
            deployment: deployment.into(),
            message: message.into(),
        }
    }

    /// Create response parsing error
    pub fn response_parsing(provider: &'static str, message: impl Into<String>) -> Self {
        Self::ResponseParsing {
            provider,
            message: message.into(),
        }
    }

    /// Create routing error
    pub fn routing_error(
        provider: &'static str,
        attempted_providers: Vec<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::RoutingError {
            provider,
            attempted_providers,
            message: message.into(),
        }
    }

    /// Create transformation error
    pub fn transformation_error(
        provider: &'static str,
        from_format: impl Into<String>,
        to_format: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::TransformationError {
            provider,
            from_format: from_format.into(),
            to_format: to_format.into(),
            message: message.into(),
        }
    }

    /// Create content filtered error
    pub fn content_filtered(
        provider: &'static str,
        reason: impl Into<String>,
        policy_violations: Option<Vec<String>>,
        potentially_retryable: Option<bool>,
    ) -> Self {
        Self::ContentFiltered {
            provider,
            reason: reason.into(),
            policy_violations,
            potentially_retryable,
        }
    }

    /// Create cancellation error
    pub fn cancelled(
        provider: &'static str,
        operation_type: impl Into<String>,
        cancellation_reason: Option<String>,
    ) -> Self {
        Self::Cancelled {
            provider,
            operation_type: operation_type.into(),
            cancellation_reason,
        }
    }

    /// Create streaming error
    pub fn streaming_error(
        provider: &'static str,
        stream_type: impl Into<String>,
        position: Option<u64>,
        last_chunk: Option<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::Streaming {
            provider,
            stream_type: stream_type.into(),
            position,
            last_chunk,
            message: message.into(),
        }
    }

    /// Create other/generic error
    pub fn other(provider: &'static str, message: impl Into<String>) -> Self {
        Self::Other {
            provider,
            message: message.into(),
        }
    }

    /// Get the provider name that caused this error
    pub fn provider(&self) -> &'static str {
        match self {
            Self::Authentication { provider, .. }
            | Self::RateLimit { provider, .. }
            | Self::QuotaExceeded { provider, .. }
            | Self::ModelNotFound { provider, .. }
            | Self::InvalidRequest { provider, .. }
            | Self::Network { provider, .. }
            | Self::ProviderUnavailable { provider, .. }
            | Self::NotSupported { provider, .. }
            | Self::NotImplemented { provider, .. }
            | Self::Configuration { provider, .. }
            | Self::Serialization { provider, .. }
            | Self::Timeout { provider, .. }
            | Self::ContextLengthExceeded { provider, .. }
            | Self::ContentFiltered { provider, .. }
            | Self::ApiError { provider, .. }
            | Self::TokenLimitExceeded { provider, .. }
            | Self::FeatureDisabled { provider, .. }
            | Self::DeploymentError { provider, .. }
            | Self::ResponseParsing { provider, .. }
            | Self::RoutingError { provider, .. }
            | Self::TransformationError { provider, .. }
            | Self::Cancelled { provider, .. }
            | Self::Streaming { provider, .. }
            | Self::Other { provider, .. } => provider,
        }
    }

    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Network { .. }
            | Self::Timeout { .. }
            | Self::RateLimit { .. }
            | Self::ProviderUnavailable { .. } => true,

            // API errors depend on status code
            Self::ApiError { status, .. } => matches!(*status, 429 | 500..=599),

            // Deployment errors might be retryable depending on the issue
            Self::DeploymentError { .. } => true,

            // Streaming errors are typically retryable
            Self::Streaming { .. } => true,

            // Content filtered might be retryable with prompt changes
            Self::ContentFiltered { potentially_retryable, .. } => {
                potentially_retryable.unwrap_or(false)
            },

            // All other errors are not retryable
            Self::Authentication { .. }
            | Self::QuotaExceeded { .. }
            | Self::ModelNotFound { .. }
            | Self::InvalidRequest { .. }
            | Self::NotSupported { .. }
            | Self::NotImplemented { .. }
            | Self::Configuration { .. }
            | Self::Serialization { .. }
            | Self::ContextLengthExceeded { .. }
            | Self::TokenLimitExceeded { .. }
            | Self::FeatureDisabled { .. }
            | Self::ResponseParsing { .. }
            | Self::RoutingError { .. }
            | Self::TransformationError { .. }
            | Self::Cancelled { .. } // User cancelled, don't retry
            | Self::Other { .. } => false,
        }
    }

    /// Get retry delay in seconds
    pub fn retry_delay(&self) -> Option<u64> {
        match self {
            Self::RateLimit { retry_after, .. } => *retry_after,
            Self::Network { .. } | Self::Timeout { .. } => Some(1),
            Self::ProviderUnavailable { .. } => Some(5),

            // API errors with 429 (rate limit) or 5xx get retry delays
            Self::ApiError { status, .. } => match *status {
                429 => Some(60),      // Rate limit, wait longer
                500..=599 => Some(3), // Server errors, shorter delay
                _ => None,
            },

            // Deployment errors get a retry delay
            Self::DeploymentError { .. } => Some(5),

            // Streaming errors get a shorter retry delay
            Self::Streaming { .. } => Some(2),

            // Content filtered - conditional retry
            Self::ContentFiltered {
                potentially_retryable,
                ..
            } => {
                if potentially_retryable.unwrap_or(false) {
                    Some(10) // Allow time for prompt modification
                } else {
                    None
                }
            }

            // All other errors have no retry delay
            Self::Authentication { .. }
            | Self::QuotaExceeded { .. }
            | Self::ModelNotFound { .. }
            | Self::InvalidRequest { .. }
            | Self::NotSupported { .. }
            | Self::NotImplemented { .. }
            | Self::Configuration { .. }
            | Self::Serialization { .. }
            | Self::ContextLengthExceeded { .. }
            | Self::TokenLimitExceeded { .. }
            | Self::FeatureDisabled { .. }
            | Self::ResponseParsing { .. }
            | Self::RoutingError { .. }
            | Self::TransformationError { .. }
            | Self::Cancelled { .. }
            | Self::Other { .. } => None,
        }
    }

    /// Get HTTP status code for this error
    pub fn http_status(&self) -> u16 {
        match self {
            Self::Authentication { .. } => 401,
            Self::RateLimit { .. } => 429,
            Self::QuotaExceeded { .. } => 402, // Payment Required
            Self::ModelNotFound { .. } => 404,
            Self::InvalidRequest { .. } => 400,
            Self::Configuration { .. } => 400,
            Self::NotSupported { .. } => 405,
            Self::NotImplemented { .. } => 501,
            Self::Network { .. } | Self::Timeout { .. } | Self::ProviderUnavailable { .. } => 503,
            Self::Serialization { .. } => 500,

            // Enhanced error variants with appropriate HTTP status codes
            Self::ContextLengthExceeded { .. } => 413, // Payload Too Large
            Self::ContentFiltered { .. } => 400,       // Bad Request (content policy violation)
            Self::ApiError { status, .. } => *status,  // Use the actual API status
            Self::TokenLimitExceeded { .. } => 413,    // Payload Too Large
            Self::FeatureDisabled { .. } => 403,       // Forbidden (feature not available)
            Self::DeploymentError { .. } => 404,       // Not Found (deployment not found)
            Self::ResponseParsing { .. } => 502,       // Bad Gateway (upstream response invalid)
            Self::RoutingError { .. } => 503, // Service Unavailable (no providers available)
            Self::TransformationError { .. } => 500, // Internal Server Error (conversion failed)
            Self::Cancelled { .. } => 499,    // Client Closed Request
            Self::Streaming { .. } => 500,    // Internal Server Error (streaming failed)

            Self::Other { .. } => 500,
        }
    }
}

// Convert from common error types
impl From<reqwest::Error> for ProviderError {
    fn from(err: reqwest::Error) -> Self {
        let provider = "unknown"; // Will be overridden by provider-specific constructors

        if err.is_timeout() {
            Self::timeout(provider, err.to_string())
        } else {
            Self::network(provider, err.to_string())
        }
    }
}

impl From<serde_json::Error> for ProviderError {
    fn from(err: serde_json::Error) -> Self {
        Self::serialization("unknown", err.to_string())
    }
}

// Convert from provider-specific errors for unified handling
impl From<crate::core::types::errors::OpenAIError> for ProviderError {
    fn from(err: crate::core::types::errors::OpenAIError) -> Self {
        use crate::core::types::errors::OpenAIError;
        match err {
            OpenAIError::Authentication(msg) => Self::authentication("openai", msg),
            OpenAIError::RateLimit(_msg) => Self::rate_limit("openai", Some(60)),
            OpenAIError::InvalidRequest(msg) => Self::invalid_request("openai", msg),
            OpenAIError::Network(msg) => Self::network("openai", msg),
            OpenAIError::Timeout(msg) => Self::timeout("openai", msg),
            OpenAIError::Parsing(msg) => Self::serialization("openai", msg),
            OpenAIError::Streaming(msg) => Self::network("openai", msg),
            OpenAIError::UnsupportedFeature(feature) => Self::not_implemented("openai", feature),
            OpenAIError::NotImplemented(feature) => Self::not_implemented("openai", feature),
            OpenAIError::ModelNotFound { model } => Self::model_not_found("openai", model),
            OpenAIError::ApiError {
                message,
                status_code,
                ..
            } => Self::api_error("openai", status_code.unwrap_or(500), message),
            OpenAIError::Other(msg) => Self::api_error("openai", 500, msg),
        }
    }
}

// AzureError is now a type alias for ProviderError, no conversion needed

// Add more error type conversions for better interoperability
impl From<Box<dyn std::error::Error + Send + Sync>> for ProviderError {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Self::network("unknown", format!("{}", err))
    }
}

impl From<String> for ProviderError {
    fn from(err: String) -> Self {
        Self::network("unknown", err)
    }
}

// Provider-specific error conversions for unified error handling
// Note: MoonshotError and MistralError are now type aliases for ProviderError, so no From impl needed

impl From<crate::core::providers::meta_llama::common_utils::LlamaError> for ProviderError {
    fn from(err: crate::core::providers::meta_llama::common_utils::LlamaError) -> Self {
        use crate::core::providers::meta_llama::common_utils::LlamaError;
        match err {
            LlamaError::Authentication(msg) => Self::authentication("meta", msg),
            LlamaError::RateLimit(_msg) => Self::rate_limit("meta", None),
            LlamaError::ApiRequest(msg) => Self::api_error("meta", 400, msg),
            LlamaError::InvalidRequest(msg) => Self::invalid_request("meta", msg),
            LlamaError::Network(msg) => Self::network("meta", msg),
            LlamaError::Serialization(msg) => Self::serialization("meta", msg),
            LlamaError::ModelNotFound(msg) => Self::model_not_found("meta", msg),
            LlamaError::Timeout(msg) => Self::timeout("meta", msg),
            LlamaError::Configuration(msg) => Self::invalid_request("meta", msg),
            LlamaError::Other(msg) => Self::api_error("meta", 500, msg),
        }
    }
}

impl From<crate::core::providers::openrouter::OpenRouterError> for ProviderError {
    fn from(err: crate::core::providers::openrouter::OpenRouterError) -> Self {
        use crate::core::providers::openrouter::OpenRouterError;
        match err {
            OpenRouterError::Authentication(msg) => Self::authentication("openrouter", msg),
            OpenRouterError::RateLimit(_msg) => Self::rate_limit("openrouter", None),
            OpenRouterError::ModelNotFound(model) => Self::model_not_found("openrouter", model),
            OpenRouterError::UnsupportedModel(model) => Self::model_not_found("openrouter", model),
            OpenRouterError::InvalidRequest(msg) => Self::invalid_request("openrouter", msg),
            OpenRouterError::Network(msg) => Self::network("openrouter", msg),
            OpenRouterError::Parsing(msg) => Self::serialization("openrouter", msg),
            OpenRouterError::Timeout(msg) => Self::timeout("openrouter", msg),
            OpenRouterError::Configuration(msg) => Self::invalid_request("openrouter", msg),
            OpenRouterError::UnsupportedFeature(feature) => {
                Self::not_implemented("openrouter", feature)
            }
            OpenRouterError::Transformation(msg) => Self::serialization("openrouter", msg),
            OpenRouterError::ApiError {
                status_code,
                message,
            } => Self::api_error("openrouter", status_code, message),
            OpenRouterError::Other(msg) => Self::api_error("openrouter", 500, msg),
        }
    }
}

impl From<crate::core::providers::deepinfra::DeepInfraError> for ProviderError {
    fn from(err: crate::core::providers::deepinfra::DeepInfraError) -> Self {
        use crate::core::providers::deepinfra::DeepInfraError;
        match err {
            DeepInfraError::Authentication(msg) => Self::authentication("deepinfra", msg),
            DeepInfraError::RateLimit(_msg) => Self::rate_limit("deepinfra", None),
            DeepInfraError::ModelNotFound(model) => Self::model_not_found("deepinfra", model),
            DeepInfraError::Configuration(msg) => Self::invalid_request("deepinfra", msg),
            DeepInfraError::Network(msg) => Self::network("deepinfra", msg),
            DeepInfraError::Serialization(msg) => Self::serialization("deepinfra", msg),
            DeepInfraError::Validation(msg) => Self::invalid_request("deepinfra", msg),
            DeepInfraError::NotImplemented(feature) => Self::not_implemented("deepinfra", feature),
            DeepInfraError::Api { status, message } => {
                Self::api_error("deepinfra", status, message)
            }
        }
    }
}

impl From<crate::core::cost::types::CostError> for ProviderError {
    fn from(err: crate::core::cost::types::CostError) -> Self {
        use crate::core::cost::types::CostError;
        match err {
            CostError::ModelNotSupported { model, provider } => Self::model_not_found(
                "cost",
                format!("Model {} not supported for provider {}", model, provider),
            ),
            CostError::ProviderNotSupported { provider } => Self::not_implemented(
                "cost",
                format!("Provider {} does not support cost calculation", provider),
            ),
            CostError::MissingPricing { model } => {
                Self::invalid_request("cost", format!("Missing pricing for model: {}", model))
            }
            CostError::InvalidUsage { message } => Self::invalid_request("cost", message),
            CostError::CalculationError { message } => Self::api_error("cost", 500, message),
            CostError::ConfigError { message } => Self::invalid_request("cost", message),
        }
    }
}

impl From<crate::core::providers::vertex_ai::VertexAIError> for ProviderError {
    fn from(err: crate::core::providers::vertex_ai::VertexAIError) -> Self {
        use crate::core::providers::vertex_ai::VertexAIError;
        match err {
            VertexAIError::Authentication(msg) => Self::authentication("vertex_ai", msg),
            VertexAIError::Configuration(msg) => Self::invalid_request("vertex_ai", msg),
            VertexAIError::Network(msg) => Self::network("vertex_ai", msg),
            VertexAIError::ResponseParsing(msg) => Self::serialization("vertex_ai", msg),
            VertexAIError::UnsupportedModel(model) => Self::model_not_found("vertex_ai", model),
            VertexAIError::UnsupportedFeature(feature) => {
                Self::not_implemented("vertex_ai", feature)
            }
            VertexAIError::InvalidRequest(msg) => Self::invalid_request("vertex_ai", msg),
            VertexAIError::RateLimitExceeded => Self::rate_limit("vertex_ai", None),
            VertexAIError::QuotaExceeded(model) => {
                Self::quota_exceeded("vertex_ai", format!("Quota exceeded for model: {}", model))
            }
            VertexAIError::TokenLimitExceeded(msg) => Self::invalid_request("vertex_ai", msg),
            VertexAIError::ContextLengthExceeded { max, actual } => Self::invalid_request(
                "vertex_ai",
                format!("Context length exceeded: max {}, got {}", max, actual),
            ),
            VertexAIError::ContentFiltered => Self::content_filtered(
                "vertex_ai",
                "Content was blocked by safety filters".to_string(),
                None,
                None,
            ),
            VertexAIError::ServiceUnavailable => Self::provider_unavailable(
                "vertex_ai",
                "Service temporarily unavailable".to_string(),
            ),
            VertexAIError::Timeout(seconds) => Self::timeout(
                "vertex_ai",
                format!("Request timed out after {} seconds", seconds),
            ),
            VertexAIError::FeatureDisabled(feature) => Self::not_implemented("vertex_ai", feature),
            VertexAIError::ApiError {
                status_code,
                message,
            } => Self::api_error("vertex_ai", status_code, message),
            VertexAIError::Other(msg) => Self::api_error("vertex_ai", 500, msg),
        }
    }
}

impl From<crate::core::providers::v0::V0Error> for ProviderError {
    fn from(err: crate::core::providers::v0::V0Error) -> Self {
        use crate::core::providers::v0::V0Error;
        match err {
            V0Error::AuthenticationFailed => {
                Self::authentication("v0", "Authentication failed".to_string())
            }
            V0Error::RateLimitExceeded => Self::rate_limit("v0", None),
            V0Error::ModelNotFound(model) => Self::model_not_found("v0", model),
            V0Error::InvalidRequest(msg) => Self::invalid_request("v0", msg),
            V0Error::HttpError(e) => Self::network("v0", e.to_string()),
            V0Error::JsonError(e) => Self::serialization("v0", e.to_string()),
            V0Error::ApiError(msg) => Self::api_error("v0", 500, msg),
        }
    }
}

// DeepSeek now uses ProviderError directly - no conversion needed

// Azure AI provider uses ProviderError directly - no conversion needed

// Anthropic provider now uses ProviderError directly - no conversion needed

/// Type alias for backward compatibility
pub type UnifiedProviderError = ProviderError;

// Convenience methods for backward compatibility
impl ProviderError {
    /// Create authentication error (legacy method)
    pub fn authentication_legacy(msg: impl Into<String>) -> Self {
        Self::authentication("unknown", msg)
    }

    /// Create rate limit error (legacy method)
    pub fn rate_limit_legacy(msg: impl Into<String>) -> Self {
        Self::RateLimit {
            provider: "unknown",
            message: msg.into(),
            retry_after: None,
            rpm_limit: None,
            tpm_limit: None,
            current_usage: None,
        }
    }

    /// Create model not found error (legacy method)
    pub fn model_not_found_legacy(msg: impl Into<String>) -> Self {
        Self::ModelNotFound {
            provider: "unknown",
            model: msg.into(),
        }
    }

    /// Create network error (legacy method)
    pub fn network_legacy(msg: impl Into<String>) -> Self {
        Self::network("unknown", msg)
    }

    /// Create generic error (legacy method)
    pub fn generic(err: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::network("unknown", err.to_string())
    }
}

// Implement ProviderErrorTrait for ProviderError
use crate::core::types::errors::ProviderErrorTrait;

impl ProviderErrorTrait for ProviderError {
    fn error_type(&self) -> &'static str {
        match self {
            Self::Authentication { .. } => "authentication",
            Self::RateLimit { .. } => "rate_limit",
            Self::QuotaExceeded { .. } => "quota_exceeded",
            Self::ModelNotFound { .. } => "model_not_found",
            Self::InvalidRequest { .. } => "invalid_request",
            Self::Network { .. } => "network",
            Self::ProviderUnavailable { .. } => "provider_unavailable",
            Self::NotSupported { .. } => "not_supported",
            Self::NotImplemented { .. } => "not_implemented",
            Self::Configuration { .. } => "configuration",
            Self::Serialization { .. } => "serialization",
            Self::Timeout { .. } => "timeout",

            // Enhanced error variants
            Self::ContextLengthExceeded { .. } => "context_length_exceeded",
            Self::ContentFiltered { .. } => "content_filtered",
            Self::ApiError { .. } => "api_error",
            Self::TokenLimitExceeded { .. } => "token_limit_exceeded",
            Self::FeatureDisabled { .. } => "feature_disabled",
            Self::DeploymentError { .. } => "deployment_error",
            Self::ResponseParsing { .. } => "response_parsing",
            Self::RoutingError { .. } => "routing_error",
            Self::TransformationError { .. } => "transformation_error",
            Self::Cancelled { .. } => "cancelled",
            Self::Streaming { .. } => "streaming",

            Self::Other { .. } => "other",
        }
    }

    fn is_retryable(&self) -> bool {
        // Delegate to the main implementation
        ProviderError::is_retryable(self)
    }

    fn retry_delay(&self) -> Option<u64> {
        // Delegate to the main implementation
        ProviderError::retry_delay(self)
    }

    fn http_status(&self) -> u16 {
        // Delegate to the main implementation
        ProviderError::http_status(self)
    }

    fn not_supported(feature: &str) -> Self {
        Self::NotSupported {
            provider: "unknown",
            feature: feature.to_string(),
        }
    }

    fn authentication_failed(reason: &str) -> Self {
        Self::Authentication {
            provider: "unknown",
            message: reason.to_string(),
        }
    }

    fn rate_limited(retry_after: Option<u64>) -> Self {
        Self::RateLimit {
            provider: "unknown",
            message: "Rate limit exceeded".to_string(),
            retry_after,
            rpm_limit: None,
            tpm_limit: None,
            current_usage: None,
        }
    }

    fn network_error(details: &str) -> Self {
        Self::Network {
            provider: "unknown",
            message: details.to_string(),
        }
    }

    fn parsing_error(details: &str) -> Self {
        Self::Serialization {
            provider: "unknown",
            message: details.to_string(),
        }
    }

    fn not_implemented(feature: &str) -> Self {
        Self::NotImplemented {
            provider: "unknown",
            feature: feature.to_string(),
        }
    }
}
