//! Webhook integration system
//!
//! This module provides webhook functionality for external system integration.

mod delivery;
pub mod events;
mod manager;
#[cfg(test)]
mod tests;
mod types;

// Re-export public types and structs for backward compatibility
pub use manager::WebhookManager;
pub use types::{
    WebhookConfig, WebhookDelivery, WebhookDeliveryStatus, WebhookEventType, WebhookPayload,
    WebhookStats,
};
