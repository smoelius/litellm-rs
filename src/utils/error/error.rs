//! Error handling for the Gateway
//!
//! This module defines all error types used throughout the gateway.

#![allow(missing_docs)]

use crate::core::providers::unified_provider::ProviderError;
use actix_web::{HttpResponse, ResponseError};
use thiserror::Error;

/// Result type alias for the Gateway
pub type Result<T> = std::result::Result<T, GatewayError>;

/// Main error type for the Gateway
#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum GatewayError {
    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// Database errors
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),

    /// Redis errors
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    /// HTTP client errors
    #[error("HTTP client error: {0}")]
    HttpClient(#[from] reqwest::Error),

    /// Serialization errors
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// YAML parsing errors
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Authentication errors
    #[error("Authentication error: {0}")]
    Auth(String),

    /// Authorization errors
    #[error("Authorization error: {0}")]
    Authorization(String),

    /// Provider errors
    #[error("Provider error: {0}")]
    Provider(ProviderError),

    /// Rate limiting errors
    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    /// Validation errors
    #[error("Validation error: {0}")]
    Validation(String),

    /// Cache errors
    #[error("Cache error: {0}")]
    Cache(String),

    /// Circuit breaker errors
    #[error("Circuit breaker error: {0}")]
    CircuitBreaker(String),

    /// Timeout errors
    #[error("Timeout error: {0}")]
    Timeout(String),

    /// Not found errors
    #[error("Not found: {0}")]
    NotFound(String),

    /// Conflict errors
    #[error("Conflict: {0}")]
    Conflict(String),

    /// Bad request errors
    #[error("Bad request: {0}")]
    BadRequest(String),

    /// Internal server errors
    #[error("Internal server error: {0}")]
    Internal(String),

    /// Service unavailable errors
    #[error("Service unavailable: {0}")]
    ProviderUnavailable(String),

    /// JWT errors
    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    /// Crypto errors
    #[error("Crypto error: {0}")]
    Crypto(String),

    /// File storage errors
    #[error("File storage error: {0}")]
    FileStorage(String),

    /// Vector database errors
    #[error("Vector database error: {0}")]
    VectorDb(String),

    /// Monitoring errors
    #[error("Monitoring error: {0}")]
    Monitoring(String),

    /// Integration errors
    #[error("Integration error: {0}")]
    Integration(String),

    /// Network errors
    #[error("Network error: {0}")]
    Network(String),

    /// Parsing errors
    #[error("Parsing error: {0}")]
    Parsing(String),

    /// Alert errors
    #[error("Alert error: {0}")]
    Alert(String),

    /// Not implemented errors
    #[error("Not implemented: {0}")]
    NotImplemented(String),

    /// Unauthorized errors
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    /// Forbidden errors
    #[error("Forbidden: {0}")]
    Forbidden(String),

    /// External service errors
    #[error("External service error: {0}")]
    External(String),

    /// Invalid request errors
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// No providers available
    #[error("No providers available: {0}")]
    NoProvidersAvailable(String),

    /// Provider not found
    #[error("Provider not found: {0}")]
    ProviderNotFound(String),

    /// No providers for model
    #[error("No providers for model: {0}")]
    NoProvidersForModel(String),

    /// No healthy providers
    #[error("No healthy providers: {0}")]
    NoHealthyProviders(String),

    /// S3 storage errors
    #[cfg(feature = "s3")]
    #[error("S3 error: {0}")]
    S3(String),

    /// Vector database client errors
    #[cfg(feature = "vector-db")]
    #[error("Qdrant error: {0}")]
    Qdrant(String),

    /// WebSocket errors
    #[cfg(feature = "websockets")]
    #[error("WebSocket error: {0}")]
    WebSocket(String),

    /// Migration errors
    #[error("Migration error: {0}")]
    Migration(String),

    /// Session errors
    #[error("Session error: {0}")]
    Session(String),

    /// Email service errors
    #[error("Email error: {0}")]
    Email(String),
}

impl ResponseError for GatewayError {
    fn error_response(&self) -> HttpResponse {
        let (status_code, error_code, message) = match self {
            GatewayError::Config(_) => (
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                "CONFIG_ERROR",
                self.to_string(),
            ),
            GatewayError::Database(_) => (
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                "DATABASE_ERROR",
                "Database operation failed".to_string(),
            ),
            GatewayError::Redis(_) => (
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                "CACHE_ERROR",
                "Cache operation failed".to_string(),
            ),
            GatewayError::Auth(_) => (
                actix_web::http::StatusCode::UNAUTHORIZED,
                "AUTH_ERROR",
                self.to_string(),
            ),
            GatewayError::Authorization(_) => (
                actix_web::http::StatusCode::FORBIDDEN,
                "AUTHORIZATION_ERROR",
                self.to_string(),
            ),
            GatewayError::Provider(provider_error) => match provider_error {
                ProviderError::RateLimit { .. } => (
                    actix_web::http::StatusCode::TOO_MANY_REQUESTS,
                    "PROVIDER_RATE_LIMIT",
                    provider_error.to_string(),
                ),
                ProviderError::QuotaExceeded { .. } => (
                    actix_web::http::StatusCode::PAYMENT_REQUIRED,
                    "PROVIDER_QUOTA_EXCEEDED",
                    provider_error.to_string(),
                ),
                ProviderError::ModelNotFound { .. } => (
                    actix_web::http::StatusCode::NOT_FOUND,
                    "MODEL_NOT_FOUND",
                    provider_error.to_string(),
                ),
                ProviderError::InvalidRequest { .. } => (
                    actix_web::http::StatusCode::BAD_REQUEST,
                    "INVALID_REQUEST",
                    provider_error.to_string(),
                ),
                ProviderError::Timeout { .. } => (
                    actix_web::http::StatusCode::GATEWAY_TIMEOUT,
                    "PROVIDER_TIMEOUT",
                    provider_error.to_string(),
                ),
                ProviderError::ProviderUnavailable { .. } => (
                    actix_web::http::StatusCode::SERVICE_UNAVAILABLE,
                    "PROVIDER_UNAVAILABLE",
                    provider_error.to_string(),
                ),
                ProviderError::Authentication { .. } => (
                    actix_web::http::StatusCode::UNAUTHORIZED,
                    "PROVIDER_AUTH_ERROR",
                    provider_error.to_string(),
                ),
                _ => (
                    actix_web::http::StatusCode::BAD_GATEWAY,
                    "PROVIDER_ERROR",
                    provider_error.to_string(),
                ),
            },
            GatewayError::RateLimit(_) => (
                actix_web::http::StatusCode::TOO_MANY_REQUESTS,
                "RATE_LIMIT_EXCEEDED",
                self.to_string(),
            ),
            GatewayError::Validation(_) => (
                actix_web::http::StatusCode::BAD_REQUEST,
                "VALIDATION_ERROR",
                self.to_string(),
            ),
            GatewayError::NotFound(_) => (
                actix_web::http::StatusCode::NOT_FOUND,
                "NOT_FOUND",
                self.to_string(),
            ),
            GatewayError::Conflict(_) => (
                actix_web::http::StatusCode::CONFLICT,
                "CONFLICT",
                self.to_string(),
            ),
            GatewayError::BadRequest(_) => (
                actix_web::http::StatusCode::BAD_REQUEST,
                "BAD_REQUEST",
                self.to_string(),
            ),
            GatewayError::Timeout(_) => (
                actix_web::http::StatusCode::REQUEST_TIMEOUT,
                "TIMEOUT",
                self.to_string(),
            ),
            GatewayError::ProviderUnavailable(_) => (
                actix_web::http::StatusCode::SERVICE_UNAVAILABLE,
                "SERVICE_UNAVAILABLE",
                self.to_string(),
            ),
            GatewayError::CircuitBreaker(_) => (
                actix_web::http::StatusCode::SERVICE_UNAVAILABLE,
                "CIRCUIT_BREAKER_OPEN",
                self.to_string(),
            ),
            GatewayError::Network(_) => (
                actix_web::http::StatusCode::BAD_GATEWAY,
                "NETWORK_ERROR",
                self.to_string(),
            ),
            GatewayError::Parsing(_) => (
                actix_web::http::StatusCode::BAD_REQUEST,
                "PARSING_ERROR",
                self.to_string(),
            ),
            GatewayError::Alert(_) => (
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                "ALERT_ERROR",
                self.to_string(),
            ),
            GatewayError::NotImplemented(_) => (
                actix_web::http::StatusCode::NOT_IMPLEMENTED,
                "NOT_IMPLEMENTED",
                self.to_string(),
            ),
            GatewayError::Unauthorized(_) => (
                actix_web::http::StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED",
                self.to_string(),
            ),
            GatewayError::Forbidden(_) => (
                actix_web::http::StatusCode::FORBIDDEN,
                "FORBIDDEN",
                self.to_string(),
            ),
            GatewayError::External(_) => (
                actix_web::http::StatusCode::BAD_GATEWAY,
                "EXTERNAL_ERROR",
                self.to_string(),
            ),
            GatewayError::InvalidRequest(_) => (
                actix_web::http::StatusCode::BAD_REQUEST,
                "INVALID_REQUEST",
                self.to_string(),
            ),
            GatewayError::NoProvidersAvailable(_) => (
                actix_web::http::StatusCode::SERVICE_UNAVAILABLE,
                "NO_PROVIDERS_AVAILABLE",
                self.to_string(),
            ),
            GatewayError::ProviderNotFound(_) => (
                actix_web::http::StatusCode::NOT_FOUND,
                "PROVIDER_NOT_FOUND",
                self.to_string(),
            ),
            GatewayError::NoProvidersForModel(_) => (
                actix_web::http::StatusCode::BAD_REQUEST,
                "NO_PROVIDERS_FOR_MODEL",
                self.to_string(),
            ),
            GatewayError::NoHealthyProviders(_) => (
                actix_web::http::StatusCode::SERVICE_UNAVAILABLE,
                "NO_HEALTHY_PROVIDERS",
                self.to_string(),
            ),
            _ => (
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "An internal error occurred".to_string(),
            ),
        };

        let error_response = ErrorResponse {
            error: ErrorDetail {
                code: error_code.to_string(),
                message,
                timestamp: chrono::Utc::now().timestamp(),
                request_id: None, // This should be set by middleware
            },
        };

        HttpResponse::build(status_code).json(error_response)
    }
}

/// Standard error response format
#[derive(serde::Serialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

/// Error detail structure
#[derive(serde::Serialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    pub timestamp: i64,
    pub request_id: Option<String>,
}

/// Helper functions for creating specific errors
#[allow(dead_code)]
impl GatewayError {
    pub fn auth<S: Into<String>>(message: S) -> Self {
        Self::Auth(message.into())
    }

    pub fn authorization<S: Into<String>>(message: S) -> Self {
        Self::Authorization(message.into())
    }

    pub fn bad_request<S: Into<String>>(message: S) -> Self {
        Self::BadRequest(message.into())
    }

    pub fn not_found<S: Into<String>>(message: S) -> Self {
        Self::NotFound(message.into())
    }

    pub fn conflict<S: Into<String>>(message: S) -> Self {
        Self::Conflict(message.into())
    }

    pub fn internal<S: Into<String>>(message: S) -> Self {
        Self::Internal(message.into())
    }

    pub fn validation<S: Into<String>>(message: S) -> Self {
        Self::Validation(message.into())
    }

    pub fn rate_limit<S: Into<String>>(message: S) -> Self {
        Self::RateLimit(message.into())
    }

    pub fn timeout<S: Into<String>>(message: S) -> Self {
        Self::Timeout(message.into())
    }

    pub fn service_unavailable<S: Into<String>>(message: S) -> Self {
        Self::ProviderUnavailable(message.into())
    }

    pub fn server<S: Into<String>>(message: S) -> Self {
        Self::Internal(message.into())
    }

    pub fn network<S: Into<String>>(message: S) -> Self {
        Self::Network(message.into())
    }

    pub fn external_service<S: Into<String>>(message: S) -> Self {
        Self::Internal(message.into())
    }

    pub fn invalid_request<S: Into<String>>(message: S) -> Self {
        Self::BadRequest(message.into())
    }

    pub fn parsing<S: Into<String>>(message: S) -> Self {
        Self::Parsing(message.into())
    }

    pub fn alert<S: Into<String>>(message: S) -> Self {
        Self::Alert(message.into())
    }

    pub fn not_implemented<S: Into<String>>(message: S) -> Self {
        Self::NotImplemented(message.into())
    }

    pub fn unauthorized<S: Into<String>>(message: S) -> Self {
        Self::Unauthorized(message.into())
    }

    pub fn forbidden<S: Into<String>>(message: S) -> Self {
        Self::Forbidden(message.into())
    }

    pub fn external<S: Into<String>>(message: S) -> Self {
        Self::External(message.into())
    }

    pub fn invalid_request_error<S: Into<String>>(message: S) -> Self {
        Self::InvalidRequest(message.into())
    }

    pub fn no_providers_available<S: Into<String>>(message: S) -> Self {
        Self::NoProvidersAvailable(message.into())
    }

    pub fn provider_not_found<S: Into<String>>(message: S) -> Self {
        Self::ProviderNotFound(message.into())
    }

    pub fn no_providers_for_model<S: Into<String>>(message: S) -> Self {
        Self::NoProvidersForModel(message.into())
    }

    pub fn no_healthy_providers<S: Into<String>>(message: S) -> Self {
        Self::NoHealthyProviders(message.into())
    }
}

#[allow(dead_code)]
impl GatewayError {
    pub fn api_error<S: Into<String>>(_status_code: u16, message: S, _provider: S) -> Self {
        // ApiError doesn't exist in unified ProviderError, map to Internal in GatewayError
        Self::Internal(message.into())
    }

    pub fn unavailable<S: Into<String>>(message: S) -> Self {
        Self::ProviderUnavailable(message.into())
    }
}

// Add conversion from core::providers::unified_provider::ProviderError
impl From<crate::core::providers::unified_provider::ProviderError> for GatewayError {
    fn from(err: crate::core::providers::unified_provider::ProviderError) -> Self {
        match err {
            crate::core::providers::unified_provider::ProviderError::Authentication {
                message,
                ..
            } => GatewayError::Auth(message),
            crate::core::providers::unified_provider::ProviderError::RateLimit {
                message, ..
            } => GatewayError::RateLimit(message),
            crate::core::providers::unified_provider::ProviderError::ModelNotFound {
                model,
                ..
            } => GatewayError::NotFound(format!("Model not found: {}", model)),
            crate::core::providers::unified_provider::ProviderError::InvalidRequest {
                message,
                ..
            } => GatewayError::BadRequest(message),
            crate::core::providers::unified_provider::ProviderError::Network {
                message, ..
            } => GatewayError::network(message),
            crate::core::providers::unified_provider::ProviderError::ProviderUnavailable {
                message,
                ..
            } => GatewayError::ProviderUnavailable(message),
            crate::core::providers::unified_provider::ProviderError::NotSupported {
                feature,
                provider,
            } => GatewayError::NotImplemented(format!(
                "Feature '{}' not supported by {}",
                feature, provider
            )),
            crate::core::providers::unified_provider::ProviderError::NotImplemented {
                feature,
                provider,
            } => GatewayError::NotImplemented(format!(
                "Feature '{}' not implemented for {}",
                feature, provider
            )),
            crate::core::providers::unified_provider::ProviderError::Configuration {
                message,
                ..
            } => GatewayError::Config(message),
            crate::core::providers::unified_provider::ProviderError::Serialization {
                message,
                ..
            } => GatewayError::parsing(message),
            crate::core::providers::unified_provider::ProviderError::Timeout {
                message, ..
            } => GatewayError::Timeout(message),
            crate::core::providers::unified_provider::ProviderError::QuotaExceeded {
                message,
                ..
            } => GatewayError::BadRequest(format!("Quota exceeded: {}", message)),
            crate::core::providers::unified_provider::ProviderError::Other { message, .. } => {
                GatewayError::Internal(message)
            }

            // Enhanced error variants mapping
            crate::core::providers::unified_provider::ProviderError::ContextLengthExceeded {
                max,
                actual,
                provider,
            } => GatewayError::BadRequest(format!(
                "Context length exceeded for {}: max {} tokens, got {} tokens",
                provider, max, actual
            )),
            crate::core::providers::unified_provider::ProviderError::ContentFiltered {
                reason,
                provider,
                ..
            } => GatewayError::BadRequest(format!(
                "Content filtered by {} safety systems: {}",
                provider, reason
            )),
            crate::core::providers::unified_provider::ProviderError::ApiError {
                status,
                message,
                provider,
            } => match status {
                401 => GatewayError::Auth(format!("{}: {}", provider, message)),
                404 => GatewayError::NotFound(format!("{}: {}", provider, message)),
                429 => GatewayError::RateLimit(format!("{}: {}", provider, message)),
                400..=499 => GatewayError::BadRequest(format!("{}: {}", provider, message)),
                _ => GatewayError::Internal(format!("{}: {}", provider, message)),
            },
            crate::core::providers::unified_provider::ProviderError::TokenLimitExceeded {
                message,
                provider,
            } => GatewayError::BadRequest(format!(
                "Token limit exceeded for {}: {}",
                provider, message
            )),
            crate::core::providers::unified_provider::ProviderError::FeatureDisabled {
                feature,
                provider,
            } => GatewayError::NotImplemented(format!(
                "Feature '{}' disabled for {}",
                feature, provider
            )),
            crate::core::providers::unified_provider::ProviderError::DeploymentError {
                deployment,
                message,
                provider,
            } => GatewayError::NotFound(format!(
                "Azure deployment '{}' error for {}: {}",
                deployment, provider, message
            )),
            crate::core::providers::unified_provider::ProviderError::ResponseParsing {
                message,
                provider,
            } => GatewayError::parsing(format!(
                "Failed to parse {} response: {}",
                provider, message
            )),
            crate::core::providers::unified_provider::ProviderError::RoutingError {
                attempted_providers,
                message,
                provider,
            } => GatewayError::ProviderUnavailable(format!(
                "Routing error from {}: tried {:?}, final error: {}",
                provider, attempted_providers, message
            )),
            crate::core::providers::unified_provider::ProviderError::TransformationError {
                from_format,
                to_format,
                message,
                provider,
            } => GatewayError::Internal(format!(
                "Transformation error for {}: from {} to {}: {}",
                provider, from_format, to_format, message
            )),
            ProviderError::Cancelled {
                provider,
                operation_type,
                ..
            } => GatewayError::BadRequest(format!(
                "Operation {} was cancelled for provider {}",
                operation_type, provider
            )),
            ProviderError::Streaming {
                provider,
                stream_type,
                ..
            } => GatewayError::Internal(format!(
                "Streaming error for provider {} on stream type {}",
                provider, stream_type
            )),
        }
    }
}

// AnthropicProviderError is now replaced by unified ProviderError
// The conversion is handled by the unified ProviderError From implementations

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = GatewayError::auth("Invalid token");
        assert!(matches!(error, GatewayError::Auth(_)));

        let error = GatewayError::bad_request("Missing parameter");
        assert!(matches!(error, GatewayError::BadRequest(_)));
    }

    #[test]
    fn test_provider_error_creation() {
        let error = ProviderError::other("openai", "Bad request");
        assert!(matches!(error, ProviderError::Other { .. }));

        let error = ProviderError::rate_limit("openai", Some(60));
        assert!(matches!(error, ProviderError::RateLimit { .. }));
    }
}
