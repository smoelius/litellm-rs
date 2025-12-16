//! Error types for the Gateway

use crate::core::providers::unified_provider::ProviderError;
use thiserror::Error;

/// Result type alias for the Gateway
pub type Result<T> = std::result::Result<T, GatewayError>;

/// Main error type for the Gateway
#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum GatewayError {
    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// Database errors
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),

    /// Redis errors
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    /// HTTP client errors
    #[error("HTTP client error: {0}")]
    HttpClient(#[from] reqwest::Error),

    /// Serialization errors
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// YAML parsing errors
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Authentication errors
    #[error("Authentication error: {0}")]
    Auth(String),

    /// Authorization errors
    #[error("Authorization error: {0}")]
    Authorization(String),

    /// Provider errors
    #[error("Provider error: {0}")]
    Provider(ProviderError),

    /// Rate limiting errors
    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    /// Validation errors
    #[error("Validation error: {0}")]
    Validation(String),

    /// Cache errors
    #[error("Cache error: {0}")]
    Cache(String),

    /// Circuit breaker errors
    #[error("Circuit breaker error: {0}")]
    CircuitBreaker(String),

    /// Timeout errors
    #[error("Timeout error: {0}")]
    Timeout(String),

    /// Not found errors
    #[error("Not found: {0}")]
    NotFound(String),

    /// Conflict errors
    #[error("Conflict: {0}")]
    Conflict(String),

    /// Bad request errors
    #[error("Bad request: {0}")]
    BadRequest(String),

    /// Internal server errors
    #[error("Internal server error: {0}")]
    Internal(String),

    /// Service unavailable errors
    #[error("Service unavailable: {0}")]
    ProviderUnavailable(String),

    /// JWT errors
    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    /// Crypto errors
    #[error("Crypto error: {0}")]
    Crypto(String),

    /// File storage errors
    #[error("File storage error: {0}")]
    FileStorage(String),

    /// Vector database errors
    #[error("Vector database error: {0}")]
    VectorDb(String),

    /// Monitoring errors
    #[error("Monitoring error: {0}")]
    Monitoring(String),

    /// Integration errors
    #[error("Integration error: {0}")]
    Integration(String),

    /// Network errors
    #[error("Network error: {0}")]
    Network(String),

    /// Parsing errors
    #[error("Parsing error: {0}")]
    Parsing(String),

    /// Alert errors
    #[error("Alert error: {0}")]
    Alert(String),

    /// Not implemented errors
    #[error("Not implemented: {0}")]
    NotImplemented(String),

    /// Unauthorized errors
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    /// Forbidden errors
    #[error("Forbidden: {0}")]
    Forbidden(String),

    /// External service errors
    #[error("External service error: {0}")]
    External(String),

    /// Invalid request errors
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// No providers available
    #[error("No providers available: {0}")]
    NoProvidersAvailable(String),

    /// Provider not found
    #[error("Provider not found: {0}")]
    ProviderNotFound(String),

    /// No providers for model
    #[error("No providers for model: {0}")]
    NoProvidersForModel(String),

    /// No healthy providers
    #[error("No healthy providers: {0}")]
    NoHealthyProviders(String),

    /// S3 storage errors
    #[cfg(feature = "s3")]
    #[error("S3 error: {0}")]
    S3(String),

    /// Vector database client errors
    #[cfg(feature = "vector-db")]
    #[error("Qdrant error: {0}")]
    Qdrant(String),

    /// WebSocket errors
    #[cfg(feature = "websockets")]
    #[error("WebSocket error: {0}")]
    WebSocket(String),

    /// Migration errors
    #[error("Migration error: {0}")]
    Migration(String),

    /// Session errors
    #[error("Session error: {0}")]
    Session(String),

    /// Email service errors
    #[error("Email error: {0}")]
    Email(String),
}
