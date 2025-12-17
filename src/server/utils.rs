//! HTTP server utility methods
//!
//! This module provides utility methods for the HttpServer.

use crate::server::server::HttpServer;
use crate::utils::error::GatewayError;
use tracing::{info, warn};

impl HttpServer {
    /// Graceful shutdown signal handler
    #[allow(dead_code)]
    pub async fn shutdown_signal() {
        let ctrl_c = async {
            match tokio::signal::ctrl_c().await {
                Ok(()) => info!("Received Ctrl+C signal, shutting down gracefully"),
                Err(e) => warn!("Failed to install Ctrl+C handler: {}", e),
            }
        };

        #[cfg(unix)]
        let terminate = async {
            match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
                Ok(mut signal) => {
                    signal.recv().await;
                    info!("Received terminate signal, shutting down gracefully");
                }
                Err(e) => {
                    warn!("Failed to install SIGTERM handler: {}", e);
                    // Wait indefinitely if signal handler fails
                    std::future::pending::<()>().await;
                }
            }
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {},
            _ = terminate => {},
        }
    }

    /// Format a user-friendly error message for port binding failures
    pub(crate) fn format_bind_error(
        error: std::io::Error,
        bind_addr: &str,
        port: u16,
    ) -> GatewayError {
        let error_str = error.to_string();

        // Check if it's an "address already in use" error
        if error_str.contains("Address already in use")
            || error_str.contains("os error 48")
            || error_str.contains("os error 98")
        {
            let message = format!(
                r#"
┌─────────────────────────────────────────────────────────────────┐
│  ❌ Error: Port {} is already in use
├─────────────────────────────────────────────────────────────────┤
│  Possible solutions:
│
│  1. Kill the existing process:
│     lsof -ti:{} | xargs kill -9
│
│  2. Use a different port:
│     --port {} or PORT={}
│
│  3. Check what's using it:
│     lsof -i:{}
└─────────────────────────────────────────────────────────────────┘
"#,
                port, port, port + 1, port + 1, port
            );
            GatewayError::server(message)
        } else if error_str.contains("Permission denied") || error_str.contains("os error 13") {
            let message = format!(
                r#"
┌─────────────────────────────────────────────────────────────────┐
│  ❌ Error: Permission denied for port {}
├─────────────────────────────────────────────────────────────────┤
│  Possible solutions:
│
│  1. Use a port >= 1024 (non-privileged):
│     --port 8000 or PORT=8000
│
│  2. Run with elevated privileges (not recommended):
│     sudo ./gateway
└─────────────────────────────────────────────────────────────────┘
"#,
                port
            );
            GatewayError::server(message)
        } else {
            GatewayError::server(format!("Failed to bind to {}: {}", bind_addr, error))
        }
    }
}
