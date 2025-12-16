//! Virtual Keys management system
//!
//! This module provides comprehensive virtual key management for the LiteLLM proxy.

mod manager;
mod requests;
mod types;

#[cfg(test)]
mod tests;

// Re-export all public types
pub use manager::VirtualKeyManager;
pub use requests::{CreateKeyRequest, UpdateKeyRequest};
pub use types::{KeyGenerationSettings, Permission, RateLimitState, RateLimits, VirtualKey};
