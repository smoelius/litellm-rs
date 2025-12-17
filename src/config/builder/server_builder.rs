//! Server configuration builder implementation

use super::types::ServerConfigBuilder;
use crate::config::ServerConfig;
use crate::utils::data::type_utils::Builder;
use std::time::Duration;

impl ServerConfigBuilder {
    /// Create a new server configuration builder
    pub fn new() -> Self {
        Self {
            host: None,
            port: None,
            workers: None,
            timeout: None,
            max_connections: None,
            enable_cors: false,
            cors_origins: Vec::new(),
        }
    }

    /// Set the host
    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.host = Some(host.into());
        self
    }

    /// Set the port
    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    /// Set the number of workers
    pub fn workers(mut self, workers: usize) -> Self {
        self.workers = Some(workers);
        self
    }

    /// Set the request timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set the maximum number of connections
    pub fn max_connections(mut self, max_connections: usize) -> Self {
        self.max_connections = Some(max_connections);
        self
    }

    /// Enable CORS
    pub fn enable_cors(mut self) -> Self {
        self.enable_cors = true;
        self
    }

    /// Add CORS origin
    pub fn add_cors_origin(mut self, origin: impl Into<String>) -> Self {
        self.cors_origins.push(origin.into());
        self
    }

    /// Build the server configuration
    pub fn build(self) -> ServerConfig {
        ServerConfig {
            host: self.host.unwrap_or_else(|| "127.0.0.1".to_string()),
            port: self.port.unwrap_or(8080),
            workers: self.workers,
            timeout: self.timeout.map(|d| d.as_secs()).unwrap_or(30),
            max_body_size: 1024 * 1024, // 1MB default
            dev_mode: false,
            tls: None,
            cors: crate::config::CorsConfig {
                enabled: self.enable_cors,
                allowed_origins: if self.cors_origins.is_empty() {
                    vec!["*".to_string()]
                } else {
                    self.cors_origins
                },
                allowed_methods: vec!["GET".to_string(), "POST".to_string(), "OPTIONS".to_string()],
                allowed_headers: vec!["Content-Type".to_string(), "Authorization".to_string()],
                max_age: 3600,
                allow_credentials: false,
            },
        }
    }
}

impl Default for ServerConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Builder<ServerConfig> for ServerConfigBuilder {
    fn build(self) -> ServerConfig {
        self.build()
    }
}
