//! Error response types

use serde::{Deserialize, Serialize};

/// Error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Error details
    pub error: ErrorDetail,
}

/// Error detail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetail {
    /// Error message
    pub message: String,
    /// Error type
    #[serde(rename = "type")]
    pub error_type: String,
    /// Error code
    pub code: Option<String>,
    /// Parameter that caused the error
    pub param: Option<String>,
}
