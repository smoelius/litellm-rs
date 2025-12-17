//! Health checking system
//!
//! This module provides comprehensive health checking for all system components.

#![allow(dead_code)]

mod checker;
mod components;
mod tasks;
mod types;

#[cfg(test)]
mod tests;

// Re-export public types
pub use checker::HealthChecker;
pub use types::{ComponentHealth, HealthCheckConfig, HealthStatus, HealthSummary};
