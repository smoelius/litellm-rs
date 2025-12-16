//! Request context types

use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// Request context for tracking and metadata
#[derive(Debug, Clone)]
pub struct RequestContext {
    /// Request ID
    pub request_id: String,
    /// User ID
    pub user_id: Option<String>,
    /// Client IP
    pub client_ip: Option<String>,
    /// User agent
    pub user_agent: Option<String>,
    /// Custom headers
    pub headers: HashMap<String, String>,
    /// Start time
    pub start_time: SystemTime,
    /// Extra metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Trace ID (for distributed tracing)
    pub trace_id: Option<String>,
    /// Span ID
    pub span_id: Option<String>,
}

impl Default for RequestContext {
    fn default() -> Self {
        Self {
            request_id: uuid::Uuid::new_v4().to_string(),
            user_id: None,
            client_ip: None,
            user_agent: None,
            headers: HashMap::new(),
            start_time: SystemTime::now(),
            metadata: HashMap::new(),
            trace_id: None,
            span_id: None,
        }
    }
}

impl RequestContext {
    /// Create new request context
    pub fn new() -> Self {
        Self::default()
    }

    /// Set user ID
    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// Set client IP
    pub fn with_client_ip(mut self, client_ip: impl Into<String>) -> Self {
        self.client_ip = Some(client_ip.into());
        self
    }

    /// Set user agent
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    /// Add header
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// Set trace ID
    pub fn with_trace_id(mut self, trace_id: impl Into<String>) -> Self {
        self.trace_id = Some(trace_id.into());
        self
    }

    /// Get elapsed time
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed().unwrap_or_default()
    }
}
