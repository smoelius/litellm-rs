//! DeepSeek Error Handling
//!
//! Error handling

use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::ErrorMapper;

/// Error
#[derive(Debug)]
pub struct DeepSeekErrorMapper;

impl ErrorMapper<ProviderError> for DeepSeekErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> ProviderError {
        match status_code {
            401 => ProviderError::authentication("deepseek", "Invalid API key"),
            403 => ProviderError::authentication("deepseek", "Permission denied"),
            404 => ProviderError::model_not_found("deepseek", "Model not found"),
            429 => {
                // Response
                let retry_after = parse_retry_after(response_body);
                ProviderError::rate_limit("deepseek", retry_after)
            }
            500..=599 => ProviderError::api_error("deepseek", status_code, response_body),
            _ => ProviderError::api_error("deepseek", status_code, response_body),
        }
    }
}

/// Response
fn parse_retry_after(response_body: &str) -> Option<u64> {
    // Simple retry time parsing, can be improved based on DeepSeek's API format
    if response_body.contains("rate limit") {
        Some(60) // Default
    } else {
        None
    }
}
