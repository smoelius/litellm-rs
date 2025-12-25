//! A2A Error types
//!
//! Defines error types for A2A protocol operations.

use std::fmt;

/// Result type for A2A operations
pub type A2AResult<T> = Result<T, A2AError>;

/// A2A-specific errors
#[derive(Debug, Clone)]
pub enum A2AError {
    /// Agent not found
    AgentNotFound {
        agent_name: String,
    },

    /// Agent already registered
    AgentAlreadyExists {
        agent_name: String,
    },

    /// Connection error
    ConnectionError {
        agent_name: String,
        message: String,
    },

    /// Authentication error
    AuthenticationError {
        agent_name: String,
        message: String,
    },

    /// Task not found
    TaskNotFound {
        agent_name: String,
        task_id: String,
    },

    /// Task failed
    TaskFailed {
        agent_name: String,
        task_id: String,
        message: String,
    },

    /// Protocol error (invalid JSON-RPC message)
    ProtocolError {
        message: String,
    },

    /// Invalid request
    InvalidRequest {
        message: String,
    },

    /// Timeout error
    Timeout {
        agent_name: String,
        timeout_ms: u64,
    },

    /// Configuration error
    ConfigurationError {
        message: String,
    },

    /// Serialization error
    SerializationError {
        message: String,
    },

    /// Provider not supported
    UnsupportedProvider {
        provider: String,
    },

    /// Rate limit exceeded
    RateLimitExceeded {
        agent_name: String,
        retry_after_ms: Option<u64>,
    },

    /// Agent busy
    AgentBusy {
        agent_name: String,
        message: String,
    },

    /// Content blocked by moderation
    ContentBlocked {
        agent_name: String,
        reason: String,
    },
}

impl fmt::Display for A2AError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            A2AError::AgentNotFound { agent_name } => {
                write!(f, "Agent not found: {}", agent_name)
            }
            A2AError::AgentAlreadyExists { agent_name } => {
                write!(f, "Agent already registered: {}", agent_name)
            }
            A2AError::ConnectionError {
                agent_name,
                message,
            } => {
                write!(f, "Connection error to agent '{}': {}", agent_name, message)
            }
            A2AError::AuthenticationError {
                agent_name,
                message,
            } => {
                write!(
                    f,
                    "Authentication failed for agent '{}': {}",
                    agent_name, message
                )
            }
            A2AError::TaskNotFound { agent_name, task_id } => {
                write!(
                    f,
                    "Task '{}' not found on agent '{}'",
                    task_id, agent_name
                )
            }
            A2AError::TaskFailed {
                agent_name,
                task_id,
                message,
            } => {
                write!(
                    f,
                    "Task '{}' failed on agent '{}': {}",
                    task_id, agent_name, message
                )
            }
            A2AError::ProtocolError { message } => {
                write!(f, "A2A protocol error: {}", message)
            }
            A2AError::InvalidRequest { message } => {
                write!(f, "Invalid A2A request: {}", message)
            }
            A2AError::Timeout {
                agent_name,
                timeout_ms,
            } => {
                write!(
                    f,
                    "Timeout waiting for agent '{}' ({}ms)",
                    agent_name, timeout_ms
                )
            }
            A2AError::ConfigurationError { message } => {
                write!(f, "A2A configuration error: {}", message)
            }
            A2AError::SerializationError { message } => {
                write!(f, "A2A serialization error: {}", message)
            }
            A2AError::UnsupportedProvider { provider } => {
                write!(f, "Unsupported agent provider: {}", provider)
            }
            A2AError::RateLimitExceeded {
                agent_name,
                retry_after_ms,
            } => {
                if let Some(ms) = retry_after_ms {
                    write!(
                        f,
                        "Rate limit exceeded for agent '{}', retry after {}ms",
                        agent_name, ms
                    )
                } else {
                    write!(f, "Rate limit exceeded for agent '{}'", agent_name)
                }
            }
            A2AError::AgentBusy { agent_name, message } => {
                write!(f, "Agent '{}' is busy: {}", agent_name, message)
            }
            A2AError::ContentBlocked { agent_name, reason } => {
                write!(
                    f,
                    "Content blocked by agent '{}': {}",
                    agent_name, reason
                )
            }
        }
    }
}

impl std::error::Error for A2AError {}

impl From<serde_json::Error> for A2AError {
    fn from(e: serde_json::Error) -> Self {
        A2AError::SerializationError {
            message: e.to_string(),
        }
    }
}

impl From<reqwest::Error> for A2AError {
    fn from(e: reqwest::Error) -> Self {
        if e.is_timeout() {
            A2AError::Timeout {
                agent_name: "unknown".to_string(),
                timeout_ms: 0,
            }
        } else if e.is_connect() {
            A2AError::ConnectionError {
                agent_name: "unknown".to_string(),
                message: e.to_string(),
            }
        } else {
            A2AError::ProtocolError {
                message: e.to_string(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_not_found_display() {
        let err = A2AError::AgentNotFound {
            agent_name: "my-agent".to_string(),
        };
        assert!(err.to_string().contains("my-agent"));
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn test_task_failed_display() {
        let err = A2AError::TaskFailed {
            agent_name: "agent".to_string(),
            task_id: "task-123".to_string(),
            message: "Something went wrong".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("task-123"));
        assert!(msg.contains("Something went wrong"));
    }

    #[test]
    fn test_rate_limit_with_retry() {
        let err = A2AError::RateLimitExceeded {
            agent_name: "agent".to_string(),
            retry_after_ms: Some(5000),
        };
        assert!(err.to_string().contains("5000ms"));
    }

    #[test]
    fn test_error_is_error_trait() {
        let err: Box<dyn std::error::Error> = Box::new(A2AError::AgentNotFound {
            agent_name: "test".to_string(),
        });
        assert!(!err.to_string().is_empty());
    }
}
