//! Core types for logging system

use serde::Serialize;
use std::collections::HashMap;
use uuid::Uuid;

/// Log entry for async processing
#[derive(Debug, Clone, Serialize)]
pub struct LogEntry {
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Log level
    pub level: String,
    /// Logger name/component
    pub logger: String,
    /// Log message
    pub message: String,
    /// Structured fields
    pub fields: HashMap<String, serde_json::Value>,
    /// Request ID for correlation
    pub request_id: Option<String>,
    /// User ID if available
    pub user_id: Option<Uuid>,
    /// Trace ID for distributed tracing
    pub trace_id: Option<String>,
}

/// Async logger configuration
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AsyncLoggerConfig {
    /// Buffer size for log entries
    pub buffer_size: usize,
    /// Whether to drop logs on buffer overflow
    pub drop_on_overflow: bool,
    /// Sampling rate for high-frequency logs (0.0 to 1.0)
    pub sample_rate: f64,
    /// Maximum log message length
    pub max_message_length: usize,
}

impl Default for AsyncLoggerConfig {
    fn default() -> Self {
        Self {
            buffer_size: 10000,
            drop_on_overflow: false,
            sample_rate: 1.0,
            max_message_length: 1024,
        }
    }
}

/// Request metrics for performance logging
#[derive(Debug)]
pub struct RequestMetrics {
    /// HTTP method (GET, POST, etc.)
    pub method: String,
    /// Request path
    pub path: String,
    /// HTTP status code
    pub status_code: u16,
    /// Request duration in milliseconds
    pub duration_ms: u64,
    /// Request size in bytes
    pub request_size: u64,
    /// Response size in bytes
    pub response_size: u64,
    /// Optional user ID
    pub user_id: Option<Uuid>,
    /// Optional request ID for tracing
    pub request_id: Option<String>,
}
