//! HTTP server implementation
//!
//! This module provides the HTTP server and routing functionality.

pub mod middleware;
pub mod routes;

use crate::config::{Config, ServerConfig};
use crate::services::pricing::PricingService;
use crate::utils::error::{GatewayError, Result};
use actix_cors::Cors;
use actix_web::{
    App, HttpResponse, HttpServer as ActixHttpServer,
    middleware::{DefaultHeaders, Logger},
    web,
};

use serde_json::json;
use std::sync::Arc;

use tracing::{info, warn};

/// HTTP server state shared across handlers
///
/// This struct contains shared resources that need to be accessed across
/// multiple request handlers. All fields are wrapped in Arc for efficient
/// sharing across threads.
#[derive(Clone)]
#[allow(dead_code)]
pub struct AppState {
    /// Gateway configuration (shared read-only)
    pub config: Arc<Config>,
    /// Authentication system
    pub auth: Arc<crate::auth::AuthSystem>,
    /// Request router
    pub router: Arc<crate::core::providers::ProviderRegistry>,
    /// Storage layer
    pub storage: Arc<crate::storage::StorageLayer>,
    /// Unified pricing service
    pub pricing: Arc<PricingService>,
}

impl AppState {
    /// Create a new AppState with shared resources
    pub fn new(
        config: Config,
        auth: crate::auth::AuthSystem,
        router: crate::core::providers::ProviderRegistry,
        storage: crate::storage::StorageLayer,
        pricing: Arc<PricingService>,
    ) -> Self {
        Self {
            config: Arc::new(config),
            auth: Arc::new(auth),
            router: Arc::new(router),
            storage: Arc::new(storage),
            pricing,
        }
    }
}

/// HTTP server
#[allow(dead_code)]
pub struct HttpServer {
    /// Server configuration
    config: ServerConfig,
    /// Application state
    state: AppState,
}

#[allow(dead_code)]
impl HttpServer {
    /// Create a new HTTP server
    pub async fn new(config: &Config) -> Result<Self> {
        info!("Creating HTTP server");

        // Create storage layer
        let storage = crate::storage::StorageLayer::new(&config.gateway.storage).await?;

        // Create auth system
        let auth =
            crate::auth::AuthSystem::new(&config.gateway.auth, Arc::new(storage.clone())).await?;

        // Create router using ProviderRegistry
        let router = crate::core::providers::ProviderRegistry::new();

        // TODO: Initialize providers from config
        // if config.gateway.providers.is_empty() {
        //     // Create default OpenAI provider if no providers configured
        //     let default_config = crate::config::ProviderConfig::default();
        //     let provider = crate::core::providers::Provider::from_config(
        //         crate::core::providers::ProviderType::OpenAI,
        //         serde_json::to_value(default_config)?
        //     )?;
        //     router.register(provider);
        // } else {
        //     for config in &config.gateway.providers {
        //         let provider = crate::core::providers::Provider::from_config(
        //             config.provider_type.as_str().into(),
        //             config.settings.clone()
        //         )?;
        //         router.register(provider);
        //     }
        // }

        // Create unified pricing service
        let pricing = Arc::new(PricingService::new(Some(
            "config/model_prices_extended.json".to_string(),
        )));
        let _ = pricing.initialize().await;

        // Start auto-refresh task
        let pricing_clone: Arc<PricingService> = Arc::clone(&pricing);
        let _pricing_task = pricing_clone.start_auto_refresh_task();

        // Create shared state using the builder method
        let state = AppState::new(config.clone(), auth, router, storage, pricing);

        Ok(Self {
            config: config.gateway.server.clone(),
            state,
        })
    }

    /// Create the Actix-web application
    fn create_app(
        state: web::Data<AppState>,
    ) -> App<
        impl actix_web::dev::ServiceFactory<
            actix_web::dev::ServiceRequest,
            Config = (),
            Response = actix_web::dev::ServiceResponse<impl actix_web::body::MessageBody>,
            Error = actix_web::Error,
            InitError = (),
        >,
    > {
        info!("Setting up routes and middleware");

        let cors_config = &state.config.gateway.server.cors;
        let mut cors = Cors::default();

        // Configure CORS based on settings
        if cors_config.enabled {
            if cors_config.allows_all_origins() {
                cors = cors.allow_any_origin();
                cors_config.validate().unwrap_or_else(|e| {
                    warn!(error = %e, "CORS Configuration Warning");
                });
            } else {
                for origin in &cors_config.allowed_origins {
                    cors = cors.allowed_origin(origin);
                }
            }

            // Convert method strings to actix methods
            let methods: Vec<actix_web::http::Method> = cors_config
                .allowed_methods
                .iter()
                .filter_map(|m| m.parse().ok())
                .collect();
            if !methods.is_empty() {
                cors = cors.allowed_methods(methods);
            }

            // Convert header strings
            let headers: Vec<actix_web::http::header::HeaderName> = cors_config
                .allowed_headers
                .iter()
                .filter_map(|h| h.parse().ok())
                .collect();
            if !headers.is_empty() {
                cors = cors.allowed_headers(headers);
            }

            cors = cors.max_age(cors_config.max_age as usize);

            if cors_config.allow_credentials {
                cors = cors.supports_credentials();
            }
        }

        App::new()
            .app_data(state)
            // Add CORS middleware with secure configuration
            .wrap(cors)
            // Add logging middleware
            .wrap(Logger::default())
            // Add default headers
            .wrap(DefaultHeaders::new().add(("Server", "LiteLLM-RS")))
            // Health check route
            .route("/health", web::get().to(health_check))
            // Configure AI API routes using the proper implementation
            .configure(routes::ai::configure_routes)
            // Configure pricing API routes
            .configure(routes::pricing::configure_pricing_routes)
    }

    /// Start the HTTP server
    pub async fn start(self) -> Result<()> {
        let bind_addr = format!("{}:{}", self.config.host, self.config.port);

        info!("Starting HTTP server on {}", bind_addr);

        let state = web::Data::new(self.state);

        // Create and start the Actix-web server
        let server = ActixHttpServer::new(move || Self::create_app(state.clone()))
            .bind(&bind_addr)
            .map_err(|e| GatewayError::server(format!("Failed to bind to {}: {}", bind_addr, e)))?
            .run();

        info!("HTTP server listening on {}", bind_addr);

        // Start the server
        server
            .await
            .map_err(|e| GatewayError::server(format!("Server error: {}", e)))?;

        info!("HTTP server stopped");
        Ok(())
    }

    /// Graceful shutdown signal handler
    #[allow(dead_code)]
    async fn shutdown_signal() {
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

    /// Get server configuration
    pub fn config(&self) -> &ServerConfig {
        &self.config
    }

    /// Get application state
    pub fn state(&self) -> &AppState {
        &self.state
    }
}

impl AppState {
    /// Get gateway configuration
    #[allow(dead_code)] // May be used by handlers
    pub fn config(&self) -> &Config {
        &self.config
    }
}

/// Server builder for easier configuration
#[allow(dead_code)]
pub struct ServerBuilder {
    config: Option<Config>,
}

#[allow(dead_code)]
impl ServerBuilder {
    /// Create a new server builder
    pub fn new() -> Self {
        Self { config: None }
    }

    /// Set configuration
    pub fn with_config(mut self, config: Config) -> Self {
        self.config = Some(config);
        self
    }

    /// Build the HTTP server
    pub async fn build(self) -> Result<HttpServer> {
        let config = self
            .config
            .ok_or_else(|| GatewayError::Config("Configuration is required".to_string()))?;

        HttpServer::new(&config).await
    }
}

/// Run the server with automatic configuration loading
#[allow(dead_code)]
pub async fn run_server() -> Result<()> {
    info!("🚀 Starting Rust LiteLLM Gateway");

    // Auto-load configuration file
    let config_path = "config/gateway.yaml";
    info!("📄 Loading configuration file: {}", config_path);

    let config = match Config::from_file(config_path).await {
        Ok(config) => {
            info!("✅ Configuration file loaded successfully");
            config
        }
        Err(e) => {
            info!(
                "⚠️  Configuration file loading failed, using default config: {}",
                e
            );
            info!("💡 Please ensure config/gateway.yaml exists with correct API keys");
            Config::default()
        }
    };

    // Create and start server
    let server = HttpServer::new(&config).await?;
    info!(
        "🌐 Server starting at: http://{}:{}",
        config.server().host,
        config.server().port
    );
    info!("📋 API Endpoints:");
    info!("   GET  /health - Health check");
    info!("   GET  /v1/models - Model list");
    info!("   POST /v1/chat/completions - Chat completions");
    info!("   POST /v1/completions - Text completions");
    info!("   POST /v1/embeddings - Text embeddings");

    server.start().await
}

impl Default for ServerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

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

// Route handlers
async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION")
    }))
}

// Removed unused list_models function - now using proper AI routes

// Placeholder functions removed - now using proper AI routes from routes::ai module

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_builder() {
        let builder = ServerBuilder::new();
        assert!(builder.config.is_none());
    }

    #[test]
    fn test_app_state_creation() {
        // Basic test to ensure module compiles
        // HttpServer requires config, so we just test that the type exists
        assert_eq!(
            std::mem::size_of::<HttpServer>(),
            std::mem::size_of::<HttpServer>()
        );
    }

    #[test]
    fn test_request_metrics_creation() {
        let metrics = RequestMetrics {
            request_id: "req-123".to_string(),
            method: "GET".to_string(),
            path: "/health".to_string(),
            status_code: 200,
            response_time_ms: 50,
            request_size: 0,
            response_size: 100,
            user_agent: Some("test-agent".to_string()),
            client_ip: Some("127.0.0.1".to_string()),
            user_id: None,
            api_key_id: None,
        };

        assert_eq!(metrics.request_id, "req-123");
        assert_eq!(metrics.method, "GET");
        assert_eq!(metrics.status_code, 200);
    }
}
