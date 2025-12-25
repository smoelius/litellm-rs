//! Cloudflare-specific error types and error mapping

use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::error_mapper::trait_def::ErrorMapper;
use crate::core::types::errors::ProviderErrorTrait;
use thiserror::Error;

/// Cloudflare-specific error types
#[derive(Debug, Error)]
pub enum CloudflareError {
    #[error("API error: {0}")]
    ApiError(String),

    #[error("Authentication failed: {0}")]
    AuthenticationError(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimitError(String),

    #[error("Invalid request: {0}")]
    InvalidRequestError(String),

    #[error("Model not found: {0}")]
    ModelNotFoundError(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailableError(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Quota exceeded: {0}")]
    QuotaExceededError(String),

    #[error("Workers AI error: {0}")]
    WorkersAIError(String),
}

impl ProviderErrorTrait for CloudflareError {
    fn error_type(&self) -> &'static str {
        match self {
            CloudflareError::ApiError(_) => "api_error",
            CloudflareError::AuthenticationError(_) => "authentication_error",
            CloudflareError::RateLimitError(_) => "rate_limit_error",
            CloudflareError::InvalidRequestError(_) => "invalid_request_error",
            CloudflareError::ModelNotFoundError(_) => "model_not_found_error",
            CloudflareError::ServiceUnavailableError(_) => "service_unavailable_error",
            CloudflareError::ConfigurationError(_) => "configuration_error",
            CloudflareError::NetworkError(_) => "network_error",
            CloudflareError::QuotaExceededError(_) => "quota_exceeded_error",
            CloudflareError::WorkersAIError(_) => "workers_ai_error",
        }
    }

    fn is_retryable(&self) -> bool {
        matches!(
            self,
            CloudflareError::RateLimitError(_)
                | CloudflareError::ServiceUnavailableError(_)
                | CloudflareError::NetworkError(_)
        )
    }

    fn retry_delay(&self) -> Option<u64> {
        match self {
            CloudflareError::RateLimitError(_) => Some(60), // Default 60 seconds for rate limit
            CloudflareError::ServiceUnavailableError(_) => Some(5), // 5 seconds for service unavailable
            CloudflareError::NetworkError(_) => Some(2),            // 2 seconds for network errors
            _ => None,
        }
    }

    fn http_status(&self) -> u16 {
        match self {
            CloudflareError::AuthenticationError(_) => 401,
            CloudflareError::RateLimitError(_) => 429,
            CloudflareError::InvalidRequestError(_) => 400,
            CloudflareError::ModelNotFoundError(_) => 404,
            CloudflareError::ServiceUnavailableError(_) => 503,
            CloudflareError::QuotaExceededError(_) => 429,
            CloudflareError::ApiError(_) => 500,
            _ => 500,
        }
    }

    fn not_supported(feature: &str) -> Self {
        CloudflareError::InvalidRequestError(format!("Feature not supported: {}", feature))
    }

    fn authentication_failed(reason: &str) -> Self {
        CloudflareError::AuthenticationError(reason.to_string())
    }

    fn rate_limited(retry_after: Option<u64>) -> Self {
        match retry_after {
            Some(seconds) => CloudflareError::RateLimitError(format!(
                "Rate limit exceeded, retry after {} seconds",
                seconds
            )),
            None => CloudflareError::RateLimitError("Rate limit exceeded".to_string()),
        }
    }

    fn network_error(details: &str) -> Self {
        CloudflareError::NetworkError(details.to_string())
    }

    fn parsing_error(details: &str) -> Self {
        CloudflareError::ApiError(format!("Response parsing error: {}", details))
    }

    fn not_implemented(feature: &str) -> Self {
        CloudflareError::InvalidRequestError(format!("Feature not implemented: {}", feature))
    }
}

impl From<CloudflareError> for ProviderError {
    fn from(error: CloudflareError) -> Self {
        match error {
            CloudflareError::ApiError(msg) => ProviderError::api_error("cloudflare", 500, msg),
            CloudflareError::AuthenticationError(msg) => {
                ProviderError::authentication("cloudflare", msg)
            }
            CloudflareError::RateLimitError(_) => ProviderError::rate_limit("cloudflare", None),
            CloudflareError::InvalidRequestError(msg) => {
                ProviderError::invalid_request("cloudflare", msg)
            }
            CloudflareError::ModelNotFoundError(msg) => {
                ProviderError::model_not_found("cloudflare", msg)
            }
            CloudflareError::ServiceUnavailableError(msg) => {
                ProviderError::api_error("cloudflare", 503, msg)
            }
            CloudflareError::ConfigurationError(msg) => {
                ProviderError::configuration("cloudflare", msg)
            }
            CloudflareError::NetworkError(msg) => ProviderError::network("cloudflare", msg),
            CloudflareError::QuotaExceededError(msg) => {
                ProviderError::token_limit_exceeded("cloudflare", msg)
            }
            CloudflareError::WorkersAIError(msg) => {
                ProviderError::api_error("cloudflare", 500, msg)
            }
        }
    }
}

/// Error mapper for Cloudflare provider
pub struct CloudflareErrorMapper;

impl ErrorMapper<CloudflareError> for CloudflareErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> CloudflareError {
        let message = if response_body.is_empty() {
            format!("HTTP error {}", status_code)
        } else {
            response_body.to_string()
        };

        match status_code {
            400 => CloudflareError::InvalidRequestError(message),
            401 => CloudflareError::AuthenticationError("Invalid API token".to_string()),
            403 => CloudflareError::AuthenticationError("Access forbidden".to_string()),
            404 => CloudflareError::ModelNotFoundError("Model not found".to_string()),
            429 => CloudflareError::RateLimitError("Rate limit exceeded".to_string()),
            500 => CloudflareError::ApiError("Internal server error".to_string()),
            502 => CloudflareError::ServiceUnavailableError("Bad gateway".to_string()),
            503 => CloudflareError::ServiceUnavailableError("Service unavailable".to_string()),
            _ => CloudflareError::ApiError(message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cloudflare_error_display() {
        let err = CloudflareError::ApiError("test error".to_string());
        assert_eq!(err.to_string(), "API error: test error");

        let err = CloudflareError::AuthenticationError("invalid token".to_string());
        assert_eq!(err.to_string(), "Authentication failed: invalid token");

        let err = CloudflareError::WorkersAIError("workers error".to_string());
        assert_eq!(err.to_string(), "Workers AI error: workers error");
    }

    #[test]
    fn test_cloudflare_error_type() {
        assert_eq!(CloudflareError::ApiError("".to_string()).error_type(), "api_error");
        assert_eq!(CloudflareError::AuthenticationError("".to_string()).error_type(), "authentication_error");
        assert_eq!(CloudflareError::RateLimitError("".to_string()).error_type(), "rate_limit_error");
        assert_eq!(CloudflareError::InvalidRequestError("".to_string()).error_type(), "invalid_request_error");
        assert_eq!(CloudflareError::ModelNotFoundError("".to_string()).error_type(), "model_not_found_error");
        assert_eq!(CloudflareError::ServiceUnavailableError("".to_string()).error_type(), "service_unavailable_error");
        assert_eq!(CloudflareError::ConfigurationError("".to_string()).error_type(), "configuration_error");
        assert_eq!(CloudflareError::NetworkError("".to_string()).error_type(), "network_error");
        assert_eq!(CloudflareError::QuotaExceededError("".to_string()).error_type(), "quota_exceeded_error");
        assert_eq!(CloudflareError::WorkersAIError("".to_string()).error_type(), "workers_ai_error");
    }

    #[test]
    fn test_cloudflare_error_is_retryable() {
        assert!(CloudflareError::RateLimitError("".to_string()).is_retryable());
        assert!(CloudflareError::ServiceUnavailableError("".to_string()).is_retryable());
        assert!(CloudflareError::NetworkError("".to_string()).is_retryable());

        assert!(!CloudflareError::ApiError("".to_string()).is_retryable());
        assert!(!CloudflareError::AuthenticationError("".to_string()).is_retryable());
        assert!(!CloudflareError::InvalidRequestError("".to_string()).is_retryable());
    }

    #[test]
    fn test_cloudflare_error_retry_delay() {
        assert_eq!(CloudflareError::RateLimitError("".to_string()).retry_delay(), Some(60));
        assert_eq!(CloudflareError::ServiceUnavailableError("".to_string()).retry_delay(), Some(5));
        assert_eq!(CloudflareError::NetworkError("".to_string()).retry_delay(), Some(2));
        assert_eq!(CloudflareError::ApiError("".to_string()).retry_delay(), None);
    }

    #[test]
    fn test_cloudflare_error_http_status() {
        assert_eq!(CloudflareError::AuthenticationError("".to_string()).http_status(), 401);
        assert_eq!(CloudflareError::RateLimitError("".to_string()).http_status(), 429);
        assert_eq!(CloudflareError::InvalidRequestError("".to_string()).http_status(), 400);
        assert_eq!(CloudflareError::ModelNotFoundError("".to_string()).http_status(), 404);
        assert_eq!(CloudflareError::ServiceUnavailableError("".to_string()).http_status(), 503);
        assert_eq!(CloudflareError::QuotaExceededError("".to_string()).http_status(), 429);
        assert_eq!(CloudflareError::ApiError("".to_string()).http_status(), 500);
    }

    #[test]
    fn test_cloudflare_error_factory_methods() {
        let err = CloudflareError::not_supported("vision");
        assert!(matches!(err, CloudflareError::InvalidRequestError(_)));

        let err = CloudflareError::authentication_failed("bad token");
        assert!(matches!(err, CloudflareError::AuthenticationError(_)));

        let err = CloudflareError::rate_limited(Some(30));
        assert!(matches!(err, CloudflareError::RateLimitError(_)));

        let err = CloudflareError::rate_limited(None);
        assert!(matches!(err, CloudflareError::RateLimitError(_)));

        let err = CloudflareError::network_error("timeout");
        assert!(matches!(err, CloudflareError::NetworkError(_)));

        let err = CloudflareError::parsing_error("invalid json");
        assert!(matches!(err, CloudflareError::ApiError(_)));

        let err = CloudflareError::not_implemented("feature");
        assert!(matches!(err, CloudflareError::InvalidRequestError(_)));
    }

    #[test]
    fn test_cloudflare_error_to_provider_error() {
        let err: ProviderError = CloudflareError::AuthenticationError("bad token".to_string()).into();
        assert!(matches!(err, ProviderError::Authentication { .. }));

        let err: ProviderError = CloudflareError::RateLimitError("limit".to_string()).into();
        assert!(matches!(err, ProviderError::RateLimit { .. }));

        let err: ProviderError = CloudflareError::ModelNotFoundError("model".to_string()).into();
        assert!(matches!(err, ProviderError::ModelNotFound { .. }));

        let err: ProviderError = CloudflareError::ConfigurationError("config".to_string()).into();
        assert!(matches!(err, ProviderError::Configuration { .. }));

        let err: ProviderError = CloudflareError::NetworkError("network".to_string()).into();
        assert!(matches!(err, ProviderError::Network { .. }));
    }

    #[test]
    fn test_cloudflare_error_mapper() {
        let mapper = CloudflareErrorMapper;

        let err = mapper.map_http_error(400, "bad request");
        assert!(matches!(err, CloudflareError::InvalidRequestError(_)));

        let err = mapper.map_http_error(401, "");
        assert!(matches!(err, CloudflareError::AuthenticationError(_)));

        let err = mapper.map_http_error(403, "");
        assert!(matches!(err, CloudflareError::AuthenticationError(_)));

        let err = mapper.map_http_error(404, "");
        assert!(matches!(err, CloudflareError::ModelNotFoundError(_)));

        let err = mapper.map_http_error(429, "");
        assert!(matches!(err, CloudflareError::RateLimitError(_)));

        let err = mapper.map_http_error(500, "");
        assert!(matches!(err, CloudflareError::ApiError(_)));

        let err = mapper.map_http_error(502, "");
        assert!(matches!(err, CloudflareError::ServiceUnavailableError(_)));

        let err = mapper.map_http_error(503, "");
        assert!(matches!(err, CloudflareError::ServiceUnavailableError(_)));

        let err = mapper.map_http_error(418, "teapot");
        assert!(matches!(err, CloudflareError::ApiError(_)));
    }
}
