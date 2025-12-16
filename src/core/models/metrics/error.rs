//! Error information models

use serde::{Deserialize, Serialize};

/// Error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorInfo {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
    /// Error type
    pub error_type: String,
    /// Provider error code
    pub provider_code: Option<String>,
    /// Stack trace
    pub stack_trace: Option<String>,
}
