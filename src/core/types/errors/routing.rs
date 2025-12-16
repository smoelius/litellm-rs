//! Routing error types

/// Routing error types
#[derive(Debug, thiserror::Error)]
pub enum RoutingError {
    #[error("No healthy providers available")]
    NoHealthyProviders,

    #[error("No suitable provider found for request")]
    NoSuitableProvider,

    #[error("All providers failed")]
    AllProvidersFailed,

    #[error("Provider '{provider}' not found")]
    ProviderNotFound { provider: String },

    #[error("Invalid routing strategy: {strategy}")]
    InvalidStrategy { strategy: String },

    #[error("Route selection failed: {reason}")]
    SelectionFailed { reason: String },

    #[error("Circuit breaker is open for provider '{provider}'")]
    CircuitBreakerOpen { provider: String },

    #[error("Load balancing failed: {reason}")]
    LoadBalancingFailed { reason: String },
}

/// Result type alias
pub type RoutingResult<T> = Result<T, RoutingError>;
