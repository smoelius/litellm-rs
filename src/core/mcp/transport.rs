//! MCP Transport Protocols
//!
//! Defines transport protocols for MCP communication.

use serde::{Deserialize, Serialize};

/// Transport protocol for MCP communication
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Transport {
    /// HTTP transport (request-response)
    #[default]
    Http,

    /// Server-Sent Events (streaming)
    Sse,

    /// Standard I/O (subprocess)
    Stdio,

    /// WebSocket (bidirectional)
    WebSocket,
}

impl Transport {
    /// Check if this transport supports streaming
    pub fn supports_streaming(&self) -> bool {
        matches!(self, Transport::Sse | Transport::WebSocket | Transport::Stdio)
    }

    /// Check if this transport is bidirectional
    pub fn is_bidirectional(&self) -> bool {
        matches!(self, Transport::WebSocket | Transport::Stdio)
    }

    /// Get the default port for this transport
    pub fn default_port(&self) -> Option<u16> {
        match self {
            Transport::Http | Transport::Sse => Some(80),
            Transport::WebSocket => Some(80),
            Transport::Stdio => None,
        }
    }

    /// Get a human-readable name for this transport
    pub fn display_name(&self) -> &'static str {
        match self {
            Transport::Http => "HTTP",
            Transport::Sse => "Server-Sent Events",
            Transport::Stdio => "Standard I/O",
            Transport::WebSocket => "WebSocket",
        }
    }
}

impl std::fmt::Display for Transport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl std::str::FromStr for Transport {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "http" => Ok(Transport::Http),
            "sse" | "server-sent-events" | "eventsource" => Ok(Transport::Sse),
            "stdio" | "process" | "subprocess" => Ok(Transport::Stdio),
            "ws" | "websocket" | "websockets" => Ok(Transport::WebSocket),
            _ => Err(format!("Unknown transport: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transport_default() {
        assert_eq!(Transport::default(), Transport::Http);
    }

    #[test]
    fn test_transport_streaming_support() {
        assert!(!Transport::Http.supports_streaming());
        assert!(Transport::Sse.supports_streaming());
        assert!(Transport::Stdio.supports_streaming());
        assert!(Transport::WebSocket.supports_streaming());
    }

    #[test]
    fn test_transport_bidirectional() {
        assert!(!Transport::Http.is_bidirectional());
        assert!(!Transport::Sse.is_bidirectional());
        assert!(Transport::Stdio.is_bidirectional());
        assert!(Transport::WebSocket.is_bidirectional());
    }

    #[test]
    fn test_transport_display_names() {
        assert_eq!(Transport::Http.display_name(), "HTTP");
        assert_eq!(Transport::Sse.display_name(), "Server-Sent Events");
        assert_eq!(Transport::Stdio.display_name(), "Standard I/O");
        assert_eq!(Transport::WebSocket.display_name(), "WebSocket");
    }

    #[test]
    fn test_transport_from_str() {
        assert_eq!("http".parse::<Transport>().unwrap(), Transport::Http);
        assert_eq!("sse".parse::<Transport>().unwrap(), Transport::Sse);
        assert_eq!("stdio".parse::<Transport>().unwrap(), Transport::Stdio);
        assert_eq!("websocket".parse::<Transport>().unwrap(), Transport::WebSocket);
        assert_eq!("ws".parse::<Transport>().unwrap(), Transport::WebSocket);
    }

    #[test]
    fn test_transport_from_str_case_insensitive() {
        assert_eq!("HTTP".parse::<Transport>().unwrap(), Transport::Http);
        assert_eq!("SSE".parse::<Transport>().unwrap(), Transport::Sse);
        assert_eq!("STDIO".parse::<Transport>().unwrap(), Transport::Stdio);
    }

    #[test]
    fn test_transport_from_str_invalid() {
        assert!("invalid".parse::<Transport>().is_err());
    }

    #[test]
    fn test_transport_serde() {
        let transport = Transport::Sse;
        let json = serde_json::to_string(&transport).unwrap();
        assert_eq!(json, "\"sse\"");

        let deserialized: Transport = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, Transport::Sse);
    }

    #[test]
    fn test_transport_default_port() {
        assert_eq!(Transport::Http.default_port(), Some(80));
        assert_eq!(Transport::Sse.default_port(), Some(80));
        assert_eq!(Transport::WebSocket.default_port(), Some(80));
        assert_eq!(Transport::Stdio.default_port(), None);
    }
}
