//! Server configuration types

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Listen address
    pub host: String,
    /// Listen port
    pub port: u16,
    /// Worker thread count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workers: Option<usize>,
    /// Maximum connections
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_connections: Option<usize>,
    /// Request timeout
    #[serde(with = "super::duration_serde")]
    pub timeout: Duration,
    /// TLS configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls: Option<TlsConfig>,
    /// Enabled features
    #[serde(default)]
    pub features: Vec<String>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            workers: None,
            max_connections: None,
            timeout: Duration::from_secs(30),
            tls: None,
            features: Vec::new(),
        }
    }
}

/// TLS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Certificate file path
    pub cert_file: String,
    /// Private key file path
    pub key_file: String,
    /// Enable HTTP/2
    #[serde(default)]
    pub http2: bool,
}
