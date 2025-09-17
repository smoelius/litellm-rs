//! Performance and Optimization utilities
//!
//! This module provides async utilities, memory management, and performance optimization tools.

pub mod r#async;
pub mod memory;
pub mod optimizer;
pub mod strings;

// Re-export commonly used types and functions
pub use r#async::*;
pub use memory::*;
pub use optimizer::*;
pub use strings::*;
