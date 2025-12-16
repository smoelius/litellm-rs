//! Router error types
//!
//! This module defines error types for the router system including
//! routing errors and cooldown triggers.

/// Cooldown trigger reason
///
/// Defines the reasons why a deployment enters cooldown state.
/// Different reasons may have different cooldown behaviors and durations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CooldownReason {
    /// Rate limit (429) - immediate cooldown
    RateLimit,
    /// Authentication error (401) - immediate cooldown
    AuthError,
    /// Not found (404) - immediate cooldown
    NotFound,
    /// Timeout (408) - immediate cooldown
    Timeout,
    /// Consecutive failures exceeded threshold
    ConsecutiveFailures,
    /// High failure rate (>50%)
    HighFailureRate,
    /// Manual cooldown
    Manual,
}

/// Router error types
///
/// Defines errors that can occur during routing operations.
#[derive(Debug, Clone, thiserror::Error)]
pub enum RouterError {
    /// Model not found in router configuration
    #[error("Model not found: {0}")]
    ModelNotFound(String),

    /// No available deployment for the requested model
    #[error("No available deployment for model: {0}")]
    NoAvailableDeployment(String),

    /// Deployment not found by ID
    #[error("Deployment not found: {0}")]
    DeploymentNotFound(String),

    /// All deployments are in cooldown state
    #[error("All deployments in cooldown for model: {0}")]
    AllDeploymentsInCooldown(String),

    /// Rate limit exceeded for model
    #[error("Rate limit exceeded for model: {0}")]
    RateLimitExceeded(String),
}
