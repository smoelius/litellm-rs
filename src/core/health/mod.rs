//! Health monitoring system for providers and services
//!
//! This module provides comprehensive health monitoring capabilities including
//! provider health checks, service availability monitoring, and health-based routing.
//!
//! # Module Structure
//!
//! - `types` - Core health status types and check results
//! - `provider` - Provider health tracking and system health aggregation
//! - `monitor` - Health monitor implementation and configuration
//! - `checker` - Health checking methods and implementations
//! - `routing` - Health-based routing methods
//! - `tests` - Test suite for health monitoring

// Module declarations
pub mod checker;
pub mod monitor;
pub mod provider;
pub mod routing;
#[cfg(test)]
mod tests;
pub mod types;

// Re-export all public types for backward compatibility
pub use monitor::{HealthMonitor, HealthMonitorConfig};
pub use provider::{ProviderHealth, SystemHealth};
pub use types::{HealthCheckResult, HealthStatus};
