//! Batch processor module
//!
//! This module provides the batch processing functionality split into logical components:
//! - `core`: Core BatchProcessor struct and public CRUD methods
//! - `validation`: Request and item validation logic
//! - `execution`: Batch execution and processing logic
//! - `utils`: Utility methods for status updates and progress tracking

pub mod core;
mod execution;
mod utils;
mod validation;
