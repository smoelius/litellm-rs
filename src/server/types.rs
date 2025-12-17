//! Server types for monitoring and health checks
//!
//! This module provides types used for server monitoring and metrics.

/// Server health status
#[derive(Debug, Clone, serde::Serialize)]
pub struct ServerHealth {
    /// Server status
    pub status: String,
    /// Server uptime in seconds
    pub uptime: u64,
    /// Number of active connections
    pub active_connections: u32,
    /// Memory usage in bytes
    pub memory_usage: u64,
    /// CPU usage percentage
    pub cpu_usage: f64,
    /// Storage health
    pub storage_health: crate::storage::StorageHealthStatus,
}

/// Request metrics for monitoring
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RequestMetrics {
    /// Request ID
    pub request_id: String,
    /// HTTP method
    pub method: String,
    /// Request path
    pub path: String,
    /// Response status code
    pub status_code: u16,
    /// Response time in milliseconds
    pub response_time_ms: u64,
    /// Request size in bytes
    pub request_size: u64,
    /// Response size in bytes
    pub response_size: u64,
    /// User agent
    pub user_agent: Option<String>,
    /// Client IP address
    pub client_ip: Option<String>,
    /// User ID (if authenticated)
    pub user_id: Option<uuid::Uuid>,
    /// API key ID (if used)
    pub api_key_id: Option<uuid::Uuid>,
}
