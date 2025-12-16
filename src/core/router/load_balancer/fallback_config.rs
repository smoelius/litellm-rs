//! Legacy fallback configuration for load balancer
//!
//! **DEPRECATED**: This module is part of the legacy load balancer system.
//! For new code, use `crate::core::router::fallback::FallbackConfig` instead,
//! which provides a more ergonomic builder pattern and thread-safe DashMap storage.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for error-specific fallbacks
///
/// **DEPRECATED**: Use `crate::core::router::fallback::FallbackConfig` for new code.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FallbackConfig {
    /// General fallbacks for any error (model -> fallback models)
    #[serde(default)]
    pub general_fallbacks: HashMap<String, Vec<String>>,
    /// Fallbacks for content policy violations
    #[serde(default)]
    pub content_policy_fallbacks: HashMap<String, Vec<String>>,
    /// Fallbacks for context window exceeded errors
    #[serde(default)]
    pub context_window_fallbacks: HashMap<String, Vec<String>>,
    /// Fallbacks for rate limit errors
    #[serde(default)]
    pub rate_limit_fallbacks: HashMap<String, Vec<String>>,
}

impl FallbackConfig {
    /// Create a new fallback config
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a general fallback
    pub fn add_general_fallback(&mut self, model: &str, fallbacks: Vec<String>) -> &mut Self {
        self.general_fallbacks.insert(model.to_string(), fallbacks);
        self
    }

    /// Add a content policy fallback
    pub fn add_content_policy_fallback(
        &mut self,
        model: &str,
        fallbacks: Vec<String>,
    ) -> &mut Self {
        self.content_policy_fallbacks
            .insert(model.to_string(), fallbacks);
        self
    }

    /// Add a context window fallback
    pub fn add_context_window_fallback(
        &mut self,
        model: &str,
        fallbacks: Vec<String>,
    ) -> &mut Self {
        self.context_window_fallbacks
            .insert(model.to_string(), fallbacks);
        self
    }

    /// Add a rate limit fallback
    pub fn add_rate_limit_fallback(&mut self, model: &str, fallbacks: Vec<String>) -> &mut Self {
        self.rate_limit_fallbacks
            .insert(model.to_string(), fallbacks);
        self
    }
}
