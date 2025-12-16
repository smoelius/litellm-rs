//! Batch processing types and data structures

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Batch request for processing multiple operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRequest {
    /// Unique batch ID
    pub batch_id: String,
    /// User ID who created the batch
    pub user_id: String,
    /// Batch type
    pub batch_type: BatchType,
    /// Individual requests in the batch
    pub requests: Vec<BatchItem>,
    /// Batch metadata
    pub metadata: HashMap<String, String>,
    /// Completion window in hours (24h default)
    pub completion_window: Option<u32>,
    /// Webhook URL for completion notification
    pub webhook_url: Option<String>,
}

/// Types of batch operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BatchType {
    /// Chat completion batch requests
    ChatCompletion,
    /// Embedding batch requests
    Embedding,
    /// Image generation batch requests
    ImageGeneration,
    /// Custom batch request type
    Custom(String),
}

/// Individual item in a batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchItem {
    /// Custom ID for this request
    pub custom_id: String,
    /// HTTP method (usually POST)
    pub method: String,
    /// API endpoint
    pub url: String,
    /// Request body
    pub body: serde_json::Value,
}

/// Database batch record (different from BatchItem)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRecord {
    /// Batch ID
    pub id: String,
    /// Object type
    pub object: String,
    /// Endpoint used
    pub endpoint: String,
    /// Input file ID
    pub input_file_id: Option<String>,
    /// Completion window
    pub completion_window: String,
    /// Batch status
    pub status: BatchStatus,
    /// Output file ID
    pub output_file_id: Option<String>,
    /// Error file ID
    pub error_file_id: Option<String>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// In progress timestamp
    pub in_progress_at: Option<DateTime<Utc>>,
    /// Expiration timestamp
    pub expires_at: Option<DateTime<Utc>>,
    /// Finalizing timestamp
    pub finalizing_at: Option<DateTime<Utc>>,
    /// Completion timestamp
    pub completed_at: Option<DateTime<Utc>>,
    /// Failed timestamp
    pub failed_at: Option<DateTime<Utc>>,
    /// Expired timestamp
    pub expired_at: Option<DateTime<Utc>>,
    /// Cancelling timestamp
    pub cancelling_at: Option<DateTime<Utc>>,
    /// Cancelled timestamp
    pub cancelled_at: Option<DateTime<Utc>>,
    /// Request counts
    pub request_counts: BatchRequestCounts,
    /// Batch metadata
    pub metadata: Option<serde_json::Value>,
}

/// Batch response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResponse {
    /// Batch ID
    pub id: String,
    /// Object type
    pub object: String,
    /// Endpoint used
    pub endpoint: String,
    /// Batch status
    pub status: BatchStatus,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Completion timestamp
    pub completed_at: Option<DateTime<Utc>>,
    /// Expiration timestamp
    pub expires_at: Option<DateTime<Utc>>,
    /// Input file ID (for file-based batches)
    pub input_file_id: Option<String>,
    /// Output file ID (for completed batches)
    pub output_file_id: Option<String>,
    /// Error file ID (for failed requests)
    pub error_file_id: Option<String>,
    /// Request counts
    pub request_counts: BatchRequestCounts,
    /// Batch metadata
    pub metadata: Option<serde_json::Value>,
    /// Completion window
    pub completion_window: String,
    /// In progress timestamp
    pub in_progress_at: Option<DateTime<Utc>>,
    /// Finalizing timestamp
    pub finalizing_at: Option<DateTime<Utc>>,
    /// Failed timestamp
    pub failed_at: Option<DateTime<Utc>>,
    /// Expired timestamp
    pub expired_at: Option<DateTime<Utc>>,
    /// Cancelling timestamp
    pub cancelling_at: Option<DateTime<Utc>>,
    /// Cancelled timestamp
    pub cancelled_at: Option<DateTime<Utc>>,
}

/// Batch processing status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BatchStatus {
    /// Batch is being validated
    Validating,
    /// Batch validation failed
    Failed,
    /// Batch is being processed
    InProgress,
    /// Batch is being finalized
    Finalizing,
    /// Batch processing completed
    Completed,
    /// Batch has expired
    Expired,
    /// Batch is being cancelled
    Cancelling,
    /// Batch has been cancelled
    Cancelled,
}

/// Request counts for batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRequestCounts {
    /// Total requests in batch
    pub total: i32,
    /// Completed requests
    pub completed: i32,
    /// Failed requests
    pub failed: i32,
}

/// Individual batch result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResult {
    /// Custom ID from request
    pub custom_id: String,
    /// HTTP response
    pub response: Option<BatchHttpResponse>,
    /// Error information
    pub error: Option<BatchError>,
}

/// HTTP response for batch item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchHttpResponse {
    /// HTTP status code
    pub status_code: u16,
    /// Response headers
    pub headers: HashMap<String, String>,
    /// Response body
    pub body: serde_json::Value,
}

/// Batch error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchError {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
    /// Additional error details
    pub details: Option<serde_json::Value>,
}
