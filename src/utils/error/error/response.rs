//! HTTP response handling for errors

use super::types::GatewayError;
use crate::core::providers::unified_provider::ProviderError;
use actix_web::{HttpResponse, ResponseError};

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
