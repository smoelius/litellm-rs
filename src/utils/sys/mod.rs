//! System and Architecture utilities
//!
//! This module provides dependency injection, shared state management, and system utilities.

pub mod di;
pub mod result;
pub mod state;

// Re-export commonly used types and functions
pub use di::*;
pub use result::*;
pub use state::*;
