//! Fallback configuration and execution result types
//!
//! This module defines fallback configuration for error handling
//! and execution result metadata.

use super::deployment::DeploymentId;
use std::collections::HashMap;
use std::sync::RwLock;

/// Fallback type enumeration
///
/// Defines different types of fallback scenarios that can trigger alternative model selection.
/// Each type corresponds to a specific error condition and has its own fallback mapping.
///
/// ## Fallback Priority
///
/// When determining fallback models, the router checks in this order:
/// 1. Specific fallback type (ContextWindow, ContentPolicy, RateLimit)
/// 2. General fallback (if no specific type matches)
/// 3. Empty list (no fallback available)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FallbackType {
    /// General fallback for any error
    General,
    /// Context window exceeded - model cannot handle the input size
    ContextWindow,
    /// Content policy violation - content was filtered/rejected
    ContentPolicy,
    /// Rate limit exceeded - too many requests
    RateLimit,
}

/// Execution result with metadata
///
/// Contains the result of a successful execution along with metadata about
/// the execution such as which deployment was used, how many attempts were made,
/// and whether fallback was used.
///
/// # Type Parameters
///
/// * `T` - The type of the result value
#[derive(Debug, Clone)]
pub struct ExecutionResult<T> {
    /// The successful result value
    pub result: T,
    /// The deployment ID that successfully handled the request
    pub deployment_id: DeploymentId,
    /// Total number of attempts across all retries and fallbacks
    pub attempts: u32,
    /// The actual model that was used (may differ from requested if fallback occurred)
    pub model_used: String,
    /// Whether a fallback model was used (true if not the original model)
    pub used_fallback: bool,
    /// Total execution latency in microseconds (including retries)
    pub latency_us: u64,
}

/// Fallback configuration
///
/// Manages fallback mappings for different error types. Each model can have different
/// fallback models configured for different scenarios.
///
/// ## Thread Safety
///
/// Uses `RwLock` to allow concurrent reads and exclusive writes.
#[derive(Debug, Default)]
pub struct FallbackConfig {
    /// General fallback: model_name -> fallback model_names
    general: RwLock<HashMap<String, Vec<String>>>,

    /// Context window exceeded fallback
    context_window: RwLock<HashMap<String, Vec<String>>>,

    /// Content policy violation fallback
    content_policy: RwLock<HashMap<String, Vec<String>>>,

    /// Rate limit exceeded fallback
    rate_limit: RwLock<HashMap<String, Vec<String>>>,
}

impl FallbackConfig {
    /// Create a new empty fallback configuration
    pub fn new() -> Self {
        Self {
            general: RwLock::new(HashMap::new()),
            context_window: RwLock::new(HashMap::new()),
            content_policy: RwLock::new(HashMap::new()),
            rate_limit: RwLock::new(HashMap::new()),
        }
    }

    /// Add general fallback models for a model (builder pattern)
    ///
    /// General fallbacks are used when no specific fallback type matches the error.
    pub fn add_general(self, model: &str, fallbacks: Vec<String>) -> Self {
        self.general
            .write()
            .unwrap()
            .insert(model.to_string(), fallbacks);
        self
    }

    /// Add context window fallback models for a model (builder pattern)
    ///
    /// Context window fallbacks are used when the input exceeds the model's maximum context length.
    pub fn add_context_window(self, model: &str, fallbacks: Vec<String>) -> Self {
        self.context_window
            .write()
            .unwrap()
            .insert(model.to_string(), fallbacks);
        self
    }

    /// Add content policy fallback models for a model (builder pattern)
    ///
    /// Content policy fallbacks are used when content is filtered or rejected by safety systems.
    pub fn add_content_policy(self, model: &str, fallbacks: Vec<String>) -> Self {
        self.content_policy
            .write()
            .unwrap()
            .insert(model.to_string(), fallbacks);
        self
    }

    /// Add rate limit fallback models for a model (builder pattern)
    ///
    /// Rate limit fallbacks are used when the model's rate limit is exceeded.
    pub fn add_rate_limit(self, model: &str, fallbacks: Vec<String>) -> Self {
        self.rate_limit
            .write()
            .unwrap()
            .insert(model.to_string(), fallbacks);
        self
    }

    /// Get fallback models for a specific type
    ///
    /// Returns a cloned vector of fallback model names. Returns empty vector if no fallbacks
    /// are configured for the given model and type.
    pub fn get_fallbacks_for_type(
        &self,
        model_name: &str,
        fallback_type: FallbackType,
    ) -> Vec<String> {
        let lock = match fallback_type {
            FallbackType::General => &self.general,
            FallbackType::ContextWindow => &self.context_window,
            FallbackType::ContentPolicy => &self.content_policy,
            FallbackType::RateLimit => &self.rate_limit,
        };

        lock.read()
            .unwrap()
            .get(model_name)
            .cloned()
            .unwrap_or_default()
    }
}
