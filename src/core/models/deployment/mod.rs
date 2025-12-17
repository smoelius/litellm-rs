//! Deployment models for the Gateway
//!
//! This module defines deployment and provider configuration structures.

mod health;
mod implementation;
mod metrics;
mod types;

// Re-export all public types for backward compatibility
pub use health::{CircuitBreakerState, DeploymentHealth};
pub use metrics::DeploymentMetrics;
pub use types::{
    BillingModel, Deployment, DeploymentCostConfig, DeploymentMetricsSnapshot,
    DeploymentRateLimits, DeploymentSnapshot, DeploymentState,
};
