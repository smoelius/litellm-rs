//! Azure OpenAI Error Handling
//!
//! Simplified error handling for Azure OpenAI Service using ProviderError directly

use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::ErrorMapper;

/// Azure error mapper for unified error handling
#[derive(Debug)]
pub struct AzureErrorMapper;

impl ErrorMapper<ProviderError> for AzureErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> ProviderError {
        match status_code {
            400 => ProviderError::invalid_request("azure", format!("Bad request: {}", response_body)),
            401 => ProviderError::authentication("azure", "Invalid Azure API key or credentials"),
            403 => ProviderError::authentication("azure", "Forbidden: insufficient permissions"),
            404 => azure_deployment_error("Azure deployment not found"),
            429 => ProviderError::rate_limit("azure", Some(60)),
            500..=599 => ProviderError::api_error("azure", status_code, format!("Server error: {}", response_body)),
            _ => ProviderError::api_error("azure", status_code, response_body),
        }
    }

    fn map_network_error(&self, error: &dyn std::error::Error) -> ProviderError {
        ProviderError::network("azure", error.to_string())
    }

    fn map_parsing_error(&self, error: &dyn std::error::Error) -> ProviderError {
        ProviderError::serialization("azure", error.to_string())
    }
}

// Azure-specific error helper functions

/// Create an Azure AD authentication error
pub fn azure_ad_error(msg: impl Into<String>) -> ProviderError {
    ProviderError::authentication("azure", format!("Azure AD: {}", msg.into()))
}

/// Create an Azure deployment error
pub fn azure_deployment_error(msg: impl Into<String>) -> ProviderError {
    ProviderError::model_not_found("azure", msg.into())
}

/// Create an Azure configuration error
pub fn azure_config_error(msg: impl Into<String>) -> ProviderError {
    ProviderError::configuration("azure", msg.into())
}

/// Create an Azure API error with status code
pub fn azure_api_error(status: u16, msg: impl Into<String>) -> ProviderError {
    ProviderError::api_error("azure", status, msg.into())
}

/// Create an Azure header validation error
pub fn azure_header_error(msg: impl Into<String>) -> ProviderError {
    ProviderError::invalid_request("azure", format!("Invalid header: {}", msg.into()))
}

// Conversion implementations are in unified_provider.rs to avoid conflicts

/// Extract error message from Azure response
pub fn extract_azure_error_message(response: &serde_json::Value) -> String {
    if let Some(error) = response.get("error") {
        if let Some(message) = error.get("message") {
            if let Some(msg_str) = message.as_str() {
                return msg_str.to_string();
            }
        }
        // Try Azure-specific error format
        if let Some(code) = error.get("code") {
            if let Some(code_str) = code.as_str() {
                let message = error.get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("Unknown error");
                return format!("{}: {}", code_str, message);
            }
        }
    }
    
    // Fallback to generic message
    response.to_string()
}

// Re-export ProviderError as AzureError for backward compatibility (temporary)
pub type AzureError = ProviderError;