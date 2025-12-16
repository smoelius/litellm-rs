//! Validation trait definition
//!
//! This module defines the core Validate trait used by all configuration structures.

/// Validation trait for configuration structures
pub trait Validate {
    fn validate(&self) -> Result<(), String>;
}
