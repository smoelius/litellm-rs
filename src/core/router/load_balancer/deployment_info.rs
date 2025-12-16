//! Deployment information for tag/group-based routing
//!
//! **DEPRECATED**: This module is part of the legacy load balancer system.
//! For new code, use `crate::core::router::Deployment` which has built-in
//! tag support and more sophisticated health tracking.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Deployment information for tag/group-based routing
///
/// **DEPRECATED**: Use `crate::core::router::Deployment` for new code.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeploymentInfo {
    /// Tags for this deployment (e.g., ["fast", "high-quality", "cost-effective"])
    #[serde(default)]
    pub tags: Vec<String>,
    /// Model group this deployment belongs to (e.g., "gpt-4-group")
    #[serde(default)]
    pub model_group: Option<String>,
    /// Priority within the group (lower = higher priority)
    #[serde(default)]
    pub priority: u32,
    /// Custom metadata for this deployment
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl DeploymentInfo {
    /// Create new deployment info
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a tag
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Add multiple tags
    pub fn with_tags(mut self, tags: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.tags.extend(tags.into_iter().map(|t| t.into()));
        self
    }

    /// Set model group
    pub fn with_group(mut self, group: impl Into<String>) -> Self {
        self.model_group = Some(group.into());
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// Check if deployment has all specified tags
    pub fn has_all_tags(&self, required_tags: &[String]) -> bool {
        required_tags.iter().all(|tag| self.tags.contains(tag))
    }

    /// Check if deployment has any of the specified tags
    pub fn has_any_tag(&self, tags: &[String]) -> bool {
        tags.iter().any(|tag| self.tags.contains(tag))
    }
}
