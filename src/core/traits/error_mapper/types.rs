//! Generic error mapper implementation
//!
//! This module provides a generic error mapper with standard HTTP status code handling
//! that can be used as a fallback or base implementation.

use crate::core::types::errors::ProviderErrorTrait;
use super::trait_def::ErrorMapper;

/// Generic error mapper with standard HTTP status code handling
///
/// Provides reasonable defaults for common HTTP status codes.
/// Can be used as a fallback or base implementation.
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
