//! Routing strategies for provider selection
//!
//! This module provides various strategies for selecting providers based on different criteria
//! such as round-robin, latency, cost, usage, and more.

mod executor;
mod selection;
mod types;

// Re-export all public items for backward compatibility
pub use executor::StrategyExecutor;
pub use types::{ProviderUsage, RoutingStrategy};
