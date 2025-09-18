//! Error handling
//!
//! Error handling

use crate::core::types::errors::ProviderErrorTrait;
use serde_json::Value;

/// Error
///
/// Error
///
/// # Design Goals
///
/// Handle
/// Error
/// Error
/// Error
///
/// # Implementation Guide
///
/// Handle
///
/// ```rust
/// struct OpenAIErrorMapper;
///
/// impl ErrorMapper<OpenAIError> for OpenAIErrorMapper {
///     fn map_http_error(&self, status: u16, body: &str) -> OpenAIError {
///         match status {
///             401 => OpenAIError::Authentication("Invalid API key".to_string()),
///             429 => OpenAIError::RateLimit("Rate limit exceeded".to_string()),
///             _ => OpenAIError::Unknown(format!("HTTP {}: {}", status, body)),
///         }
///     }
/// }
/// ```
pub trait ErrorMapper<E>: Send + Sync + 'static
where
    E: ProviderErrorTrait,
{
    /// Response
    ///
    /// # parameter
    /// * `status_code` - HTTP status code
    /// Response
    ///
    /// # Returns
    /// Error
    ///
    /// # Common Mappings
    /// Error
    /// Error
    /// Error
    /// Error
    /// Error
    fn map_http_error(&self, status_code: u16, response_body: &str) -> E;

    /// Response
    ///
    /// # parameter
    /// Response
    ///
    /// # Returns
    /// Error
    ///
    /// Default
    /// Error
    fn map_json_error(&self, error_response: &Value) -> E {
        let error_msg = error_response
            .get("error")
            .and_then(|e| e.get("message"))
            .and_then(|m| m.as_str())
            .unwrap_or("Unknown error");

        let error_code = error_response
            .get("error")
            .and_then(|e| e.get("code"))
            .and_then(|c| c.as_str())
            .unwrap_or("unknown");

        match error_code {
            "invalid_api_key" | "authentication_failed" => {
                E::authentication_failed("Invalid API key")
            }
            "insufficient_quota" | "quota_exceeded" => E::rate_limited(None),
            "model_not_found" => E::not_supported("Model not found"),
            "invalid_request_error" => E::network_error(error_msg),
            _ => E::network_error(&format!("API Error: {}", error_msg)),
        }
    }

    /// Error
    ///
    /// # parameter
    /// Error
    ///
    /// # Returns
    /// Error
    ///
    /// Default
    /// Error
    fn map_network_error(&self, error: &dyn std::error::Error) -> E {
        E::network_error(&error.to_string())
    }

    /// Error
    ///
    /// # parameter
    /// Error
    ///
    /// # Returns
    /// Error
    ///
    /// Default
    /// Maps to parsing error
    fn map_parsing_error(&self, error: &dyn std::error::Error) -> E {
        E::parsing_error(&error.to_string())
    }

    /// Error
    ///
    /// # parameter
    /// * `timeout_duration` - Timeout duration
    ///
    /// # Returns
    /// Error
    ///
    /// Default
    /// Error
    fn map_timeout_error(&self, timeout_duration: std::time::Duration) -> E {
        E::network_error(&format!("Request timeout after {:?}", timeout_duration))
    }
}

/// Error
///
/// Error
/// Error
pub struct GenericErrorMapper;

impl<E> ErrorMapper<E> for GenericErrorMapper
where
    E: ProviderErrorTrait,
{
    fn map_http_error(&self, status_code: u16, response_body: &str) -> E {
        match status_code {
            400 => E::network_error("Bad Request: Invalid parameters"),
            401 => E::authentication_failed("Authentication failed: Invalid credentials"),
            403 => E::authentication_failed("Permission denied: Insufficient permissions"),
            404 => E::not_supported("Resource not found"),
            408 => E::network_error("Request timeout"),
            429 => E::rate_limited(None),
            500 => E::network_error("Internal server error"),
            502 => E::network_error("Bad gateway: Upstream server error"),
            503 => E::network_error("Service unavailable: Server overloaded"),
            504 => E::network_error("Gateway timeout: Upstream timeout"),
            _ => E::network_error(&format!(
                "HTTP Error {}: {}",
                status_code,
                if response_body.is_empty() {
                    "No details provided"
                } else {
                    response_body
                }
            )),
        }
    }
}

/// OpenAI error mapper
///
/// Handles OpenAI-specific error response format
pub struct OpenAIErrorMapper;

impl<E> ErrorMapper<E> for OpenAIErrorMapper
where
    E: ProviderErrorTrait,
{
    fn map_http_error(&self, status_code: u16, response_body: &str) -> E {
        // Try to parse JSON response first
        if let Ok(error_json) = serde_json::from_str::<Value>(response_body) {
            return self.map_json_error(&error_json);
        }

        // If parsing fails, use generic mapping
        GenericErrorMapper.map_http_error(status_code, response_body)
    }

    fn map_json_error(&self, error_response: &Value) -> E {
        let error_obj = error_response.get("error");

        if let Some(error) = error_obj {
            let error_type = error.get("type").and_then(|t| t.as_str()).unwrap_or("");
            let error_code = error.get("code").and_then(|c| c.as_str()).unwrap_or("");
            let message = error
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error");

            match error_type {
                "invalid_request_error" => match error_code {
                    "model_not_found" => E::not_supported("Model not found"),
                    "context_length_exceeded" => E::network_error("Context length exceeded"),
                    "invalid_api_key" => E::authentication_failed("Invalid API key"),
                    _ => E::network_error(message),
                },
                "authentication_error" => E::authentication_failed(message),
                "permission_error" => E::authentication_failed(message),
                "rate_limit_error" => {
                    // Extract retry time
                    let retry_after = error.get("retry_after").and_then(|r| r.as_u64());
                    E::rate_limited(retry_after)
                }
                "api_error" => E::network_error(message),
                "overloaded_error" => E::network_error("OpenAI servers are overloaded"),
                _ => E::network_error(&format!("OpenAI Error: {}", message)),
            }
        } else {
            E::network_error("Invalid error response format")
        }
    }
}

/// Anthropic error mapper
///
/// Handles Anthropic-specific error response format
pub struct AnthropicErrorMapper;

impl<E> ErrorMapper<E> for AnthropicErrorMapper
where
    E: ProviderErrorTrait,
{
    fn map_http_error(&self, status_code: u16, response_body: &str) -> E {
        // Try to parse JSON response first
        if let Ok(error_json) = serde_json::from_str::<Value>(response_body) {
            return self.map_json_error(&error_json);
        }

        // If parsing fails, use generic mapping
        GenericErrorMapper.map_http_error(status_code, response_body)
    }

    fn map_json_error(&self, error_response: &Value) -> E {
        let error_type = error_response
            .get("type")
            .and_then(|t| t.as_str())
            .unwrap_or("");
        let message = error_response
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("Unknown error");

        match error_type {
            "authentication_error" => E::authentication_failed(message),
            "permission_error" => E::authentication_failed(message),
            "not_found_error" => E::not_supported(message),
            "rate_limit_error" => E::rate_limited(None),
            "api_error" => E::network_error(message),
            "overloaded_error" => E::network_error("Anthropic servers are overloaded"),
            "validation_error" => E::network_error(&format!("Validation error: {}", message)),
            _ => E::network_error(&format!("Anthropic Error: {}", message)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // Create a test error type for testing
    #[derive(Debug, PartialEq)]
    enum TestError {
        Authentication(String),
        RateLimit(Option<u64>),
        Network(String),
        NotSupported(String),
        Parsing(String),
    }

    impl std::fmt::Display for TestError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Authentication(msg) => write!(f, "Authentication: {}", msg),
                Self::RateLimit(retry) => write!(f, "Rate limit: {:?}", retry),
                Self::Network(msg) => write!(f, "Network: {}", msg),
                Self::NotSupported(msg) => write!(f, "Not supported: {}", msg),
                Self::Parsing(msg) => write!(f, "Parsing: {}", msg),
            }
        }
    }

    impl std::error::Error for TestError {}

    impl ProviderErrorTrait for TestError {
        fn error_type(&self) -> &'static str {
            match self {
                Self::Authentication(_) => "authentication_error",
                Self::RateLimit(_) => "rate_limit_error",
                Self::Network(_) => "network_error",
                Self::NotSupported(_) => "not_supported_error",
                Self::Parsing(_) => "parsing_error",
            }
        }

        fn is_retryable(&self) -> bool {
            matches!(self, Self::Network(_) | Self::RateLimit(_))
        }

        fn retry_delay(&self) -> Option<u64> {
            match self {
                Self::RateLimit(delay) => *delay,
                Self::Network(_) => Some(1),
                _ => None,
            }
        }

        fn http_status(&self) -> u16 {
            match self {
                Self::Authentication(_) => 401,
                Self::RateLimit(_) => 429,
                Self::NotSupported(_) => 404,
                Self::Network(_) => 500,
                Self::Parsing(_) => 400,
            }
        }

        fn not_supported(feature: &str) -> Self {
            Self::NotSupported(feature.to_string())
        }

        fn authentication_failed(reason: &str) -> Self {
            Self::Authentication(reason.to_string())
        }

        fn rate_limited(retry_after: Option<u64>) -> Self {
            Self::RateLimit(retry_after)
        }

        fn network_error(details: &str) -> Self {
            Self::Network(details.to_string())
        }

        fn parsing_error(details: &str) -> Self {
            Self::Parsing(details.to_string())
        }

        fn not_implemented(feature: &str) -> Self {
            Self::NotSupported(format!("Feature not implemented: {}", feature))
        }
    }

    #[test]
    fn test_generic_error_mapper() {
        let mapper = GenericErrorMapper;

        // Test authentication error
        let auth_error: TestError = mapper.map_http_error(401, "Unauthorized");
        assert_eq!(
            auth_error,
            TestError::Authentication("Authentication failed: Invalid credentials".to_string())
        );

        // Test rate limit error
        let rate_limit_error: TestError = mapper.map_http_error(429, "Too Many Requests");
        assert_eq!(rate_limit_error, TestError::RateLimit(None));

        // Test server error
        let server_error: TestError = mapper.map_http_error(500, "Internal Server Error");
        assert_eq!(
            server_error,
            TestError::Network("Internal server error".to_string())
        );
    }

    #[test]
    fn test_openai_error_mapper() {
        let mapper = OpenAIErrorMapper;

        // Test OpenAI JSON error format
        let error_json = json!({
            "error": {
                "type": "invalid_request_error",
                "code": "model_not_found",
                "message": "The model 'gpt-5' does not exist"
            }
        });

        let error: TestError = mapper.map_json_error(&error_json);
        assert_eq!(
            error,
            TestError::NotSupported("Model not found".to_string())
        );

        // Test rate limit with retry after
        let rate_limit_json = json!({
            "error": {
                "type": "rate_limit_error",
                "message": "Rate limit exceeded",
                "retry_after": 60
            }
        });

        let rate_error: TestError = mapper.map_json_error(&rate_limit_json);
        assert_eq!(rate_error, TestError::RateLimit(Some(60)));
    }

    #[test]
    fn test_anthropic_error_mapper() {
        let mapper = AnthropicErrorMapper;

        // Test Anthropic JSON error format
        let error_json = json!({
            "type": "authentication_error",
            "message": "Invalid API key provided"
        });

        let error: TestError = mapper.map_json_error(&error_json);
        assert_eq!(
            error,
            TestError::Authentication("Invalid API key provided".to_string())
        );

        // Test validation error
        let validation_json = json!({
            "type": "validation_error",
            "message": "Missing required parameter: messages"
        });

        let validation_error: TestError = mapper.map_json_error(&validation_json);
        assert_eq!(
            validation_error,
            TestError::Network(
                "Validation error: Missing required parameter: messages".to_string()
            )
        );
    }
}
