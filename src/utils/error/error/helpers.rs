//! Helper functions for creating specific error types

use super::types::GatewayError;

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
