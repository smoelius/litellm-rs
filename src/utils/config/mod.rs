//! Configuration Management utilities
//!
//! This module provides configuration loading, validation, and management utilities.

pub mod config;
pub mod optimized;
pub mod utils;

// Re-export commonly used types and functions
pub use config::*;
pub use optimized::*;
pub use utils::{ConfigDefaults, ConfigManager, ConfigUtils};
