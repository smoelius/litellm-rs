//! API key authentication and management
//!
//! This module provides API key creation, verification, and management functionality.

mod creation;
mod management;
mod permissions;
mod types;

#[cfg(test)]
mod tests;

// Re-export public types and structs for backward compatibility
pub use creation::ApiKeyHandler;
pub use types::{ApiKeyVerification, CreateApiKeyRequest};
