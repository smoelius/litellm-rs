//! Monitoring and observability system
//!
//! This module provides comprehensive monitoring, metrics, and observability functionality.

// Public submodules
pub mod alerts;
pub mod health;
pub mod metrics;

// Internal submodules
mod background;
mod system;
mod tests;
mod types;

// Re-export public types
pub use system::MonitoringSystem;
pub use types::{
    Alert, AlertSeverity, ErrorMetrics, LatencyPercentiles, PerformanceMetrics, ProviderMetrics,
    RequestMetrics, SystemResourceMetrics,
};

// SystemMetrics is used internally but also available if needed
#[allow(unused_imports)]
pub use types::SystemMetrics;
