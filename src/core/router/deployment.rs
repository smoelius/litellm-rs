//! Deployment core data structures for Router Phase 1
//!
//! This module defines the fundamental building blocks for the LiteLLM Router:
//! - `Deployment`: A concrete provider deployment with configuration and runtime state
//! - `DeploymentConfig`: Configuration parameters (TPM/RPM limits, timeouts, weights)
//! - `DeploymentState`: Lock-free runtime state using atomic operations
//! - `HealthStatus`: Health status enumeration for deployments
//!
//! ## Design Philosophy
//!
//! All state tracking uses atomic operations with `Relaxed` ordering for maximum performance.
//! This is safe because:
//! - State values are eventually consistent (exact precision not required for routing decisions)
//! - No cross-field invariants need to be maintained atomically
//! - Routing can tolerate slightly stale state for massive performance gains
//!
//! ## Performance Characteristics
//!
//! - Lock-free: All state updates use atomics, zero contention
//! - Zero-copy: Deployments are accessed by reference, never cloned
//! - Cache-friendly: Hot path fields grouped together

use crate::core::providers::Provider;
use std::sync::atomic::{AtomicU8, AtomicU32, AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Deployment identifier (unique within router)
pub type DeploymentId = String;

/// Health status enumeration for deployments
///
/// Maps to AtomicU8 values for lock-free updates:
/// - 0 = Unknown (newly created, not yet health checked)
/// - 1 = Healthy (passing health checks, ready to serve)
/// - 2 = Degraded (experiencing issues but still functional)
/// - 3 = Unhealthy (failing health checks, should not serve)
/// - 4 = Cooldown (temporarily disabled after failures)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum HealthStatus {
    Unknown = 0,
    Healthy = 1,
    Degraded = 2,
    Unhealthy = 3,
    Cooldown = 4,
}

impl From<u8> for HealthStatus {
    fn from(value: u8) -> Self {
        match value {
            1 => HealthStatus::Healthy,
            2 => HealthStatus::Degraded,
            3 => HealthStatus::Unhealthy,
            4 => HealthStatus::Cooldown,
            _ => HealthStatus::Unknown,
        }
    }
}

impl From<HealthStatus> for u8 {
    fn from(status: HealthStatus) -> Self {
        status as u8
    }
}

/// Deployment configuration
///
/// These are static parameters that don't change during runtime.
/// All are stored as simple values (no atomics needed).
#[derive(Debug, Clone)]
pub struct DeploymentConfig {
    /// Tokens per minute limit (None = unlimited)
    pub tpm_limit: Option<u64>,

    /// Requests per minute limit (None = unlimited)
    pub rpm_limit: Option<u64>,

    /// Maximum parallel requests (None = unlimited)
    pub max_parallel_requests: Option<u32>,

    /// Weight for weighted random selection (higher = more likely to be selected)
    pub weight: u32,

    /// Timeout in seconds
    pub timeout_secs: u64,

    /// Priority (lower value = higher priority)
    pub priority: u32,
}

impl Default for DeploymentConfig {
    fn default() -> Self {
        Self {
            tpm_limit: None,
            rpm_limit: None,
            max_parallel_requests: None,
            weight: 1,
            timeout_secs: 60,
            priority: 0,
        }
    }
}

/// Deployment runtime state
///
/// All fields use atomics for lock-free updates with `Relaxed` ordering.
/// This is safe because routing decisions can tolerate eventual consistency.
///
/// ## State Reset
///
/// TPM/RPM counters are reset every minute by a background task.
/// The `minute_reset_at` timestamp tracks when the last reset occurred.
#[derive(Debug)]
pub struct DeploymentState {
    /// Health status (0=unknown, 1=healthy, 2=degraded, 3=unhealthy, 4=cooldown)
    pub health: AtomicU8,

    /// Current minute TPM usage
    pub tpm_current: AtomicU64,

    /// Current minute RPM usage
    pub rpm_current: AtomicU64,

    /// Current active requests
    pub active_requests: AtomicU32,

    /// Total requests (lifetime)
    pub total_requests: AtomicU64,

    /// Successful requests (lifetime)
    pub success_requests: AtomicU64,

    /// Failed requests (lifetime)
    pub fail_requests: AtomicU64,

    /// Failures this minute (for cooldown detection)
    pub fails_this_minute: AtomicU32,

    /// Cooldown end timestamp (unix seconds)
    pub cooldown_until: AtomicU64,

    /// Last request timestamp (unix seconds)
    pub last_request_at: AtomicU64,

    /// Average latency in microseconds (sliding window)
    pub avg_latency_us: AtomicU64,

    /// Last minute reset timestamp (unix seconds)
    pub minute_reset_at: AtomicU64,
}

impl DeploymentState {
    /// Create new deployment state with default values
    pub fn new() -> Self {
        let now = current_timestamp();
        Self {
            health: AtomicU8::new(HealthStatus::Healthy as u8),
            tpm_current: AtomicU64::new(0),
            rpm_current: AtomicU64::new(0),
            active_requests: AtomicU32::new(0),
            total_requests: AtomicU64::new(0),
            success_requests: AtomicU64::new(0),
            fail_requests: AtomicU64::new(0),
            fails_this_minute: AtomicU32::new(0),
            cooldown_until: AtomicU64::new(0),
            last_request_at: AtomicU64::new(0),
            avg_latency_us: AtomicU64::new(0),
            minute_reset_at: AtomicU64::new(now),
        }
    }

    /// Reset per-minute counters
    ///
    /// Should be called by a background task every minute.
    pub fn reset_minute(&self) {
        self.tpm_current.store(0, Ordering::Relaxed);
        self.rpm_current.store(0, Ordering::Relaxed);
        self.fails_this_minute.store(0, Ordering::Relaxed);
        self.minute_reset_at
            .store(current_timestamp(), Ordering::Relaxed);
    }

    /// Get current health status
    pub fn health_status(&self) -> HealthStatus {
        self.health.load(Ordering::Relaxed).into()
    }
}

impl Default for DeploymentState {
    fn default() -> Self {
        Self::new()
    }
}

// Manual Clone implementation because AtomicU64 doesn't implement Clone
impl Clone for DeploymentState {
    fn clone(&self) -> Self {
        Self {
            health: AtomicU8::new(self.health.load(Ordering::Relaxed)),
            tpm_current: AtomicU64::new(self.tpm_current.load(Ordering::Relaxed)),
            rpm_current: AtomicU64::new(self.rpm_current.load(Ordering::Relaxed)),
            active_requests: AtomicU32::new(self.active_requests.load(Ordering::Relaxed)),
            total_requests: AtomicU64::new(self.total_requests.load(Ordering::Relaxed)),
            success_requests: AtomicU64::new(self.success_requests.load(Ordering::Relaxed)),
            fail_requests: AtomicU64::new(self.fail_requests.load(Ordering::Relaxed)),
            fails_this_minute: AtomicU32::new(self.fails_this_minute.load(Ordering::Relaxed)),
            cooldown_until: AtomicU64::new(self.cooldown_until.load(Ordering::Relaxed)),
            last_request_at: AtomicU64::new(self.last_request_at.load(Ordering::Relaxed)),
            avg_latency_us: AtomicU64::new(self.avg_latency_us.load(Ordering::Relaxed)),
            minute_reset_at: AtomicU64::new(self.minute_reset_at.load(Ordering::Relaxed)),
        }
    }
}

/// Deployment - a concrete provider deployment
///
/// Represents a single deployment of a provider (e.g., "openai-gpt4-primary").
/// Multiple deployments can serve the same model_name (e.g., "gpt-4").
///
/// ## Example
///
/// ```rust,ignore
/// use litellm_rs::core::providers::Provider;
/// use litellm_rs::core::router::deployment::{Deployment, DeploymentConfig};
///
/// let deployment = Deployment::new(
///     "openai-gpt4-primary".to_string(),
///     provider,
///     "gpt-4-turbo".to_string(),
///     "gpt-4".to_string(),
/// )
/// .with_config(DeploymentConfig {
///     tpm_limit: Some(100_000),
///     rpm_limit: Some(500),
///     weight: 2,
///     ..Default::default()
/// })
/// .with_tags(vec!["production".to_string(), "fast".to_string()]);
/// ```
#[derive(Debug, Clone)]
pub struct Deployment {
    /// Unique deployment ID
    pub id: DeploymentId,

    /// Provider instance
    pub provider: Provider,

    /// Actual model name (e.g., "azure/gpt-4-turbo")
    pub model: String,

    /// User-facing model name / model group (e.g., "gpt-4")
    pub model_name: String,

    /// Configuration
    pub config: DeploymentConfig,

    /// Runtime state (lock-free)
    pub state: DeploymentState,

    /// Tags for filtering (e.g., ["production", "fast"])
    pub tags: Vec<String>,
}

impl Deployment {
    /// Create a new deployment
    ///
    /// # Arguments
    ///
    /// * `id` - Unique deployment identifier
    /// * `provider` - Provider instance
    /// * `model` - Actual model name (provider-specific)
    /// * `model_name` - User-facing model name (model group)
    pub fn new(id: DeploymentId, provider: Provider, model: String, model_name: String) -> Self {
        Self {
            id,
            provider,
            model,
            model_name,
            config: DeploymentConfig::default(),
            state: DeploymentState::new(),
            tags: Vec::new(),
        }
    }

    /// Set deployment configuration (builder pattern)
    pub fn with_config(mut self, config: DeploymentConfig) -> Self {
        self.config = config;
        self
    }

    /// Set deployment tags (builder pattern)
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Check if deployment is healthy
    ///
    /// Returns true if health status is Healthy or Degraded (but not Unknown, Unhealthy, or Cooldown).
    pub fn is_healthy(&self) -> bool {
        let status = self.state.health_status();
        matches!(status, HealthStatus::Healthy | HealthStatus::Degraded)
    }

    /// Check if deployment is in cooldown
    ///
    /// Returns true if current time is before cooldown_until timestamp.
    pub fn is_in_cooldown(&self) -> bool {
        let now = current_timestamp();
        let cooldown_until = self.state.cooldown_until.load(Ordering::Relaxed);
        cooldown_until > now
    }

    /// Record a successful request
    ///
    /// Updates counters and calculates exponential moving average for latency.
    ///
    /// # Arguments
    ///
    /// * `tokens` - Number of tokens consumed
    /// * `latency_us` - Request latency in microseconds
    pub fn record_success(&self, tokens: u64, latency_us: u64) {
        // Update counters
        self.state.total_requests.fetch_add(1, Ordering::Relaxed);
        self.state.success_requests.fetch_add(1, Ordering::Relaxed);
        self.state.tpm_current.fetch_add(tokens, Ordering::Relaxed);
        self.state.rpm_current.fetch_add(1, Ordering::Relaxed);
        self.state
            .last_request_at
            .store(current_timestamp(), Ordering::Relaxed);

        // Update average latency using exponential moving average (alpha = 0.2)
        let current_avg = self.state.avg_latency_us.load(Ordering::Relaxed);
        let new_avg = if current_avg == 0 {
            latency_us
        } else {
            // EMA: new_avg = alpha * new_value + (1 - alpha) * old_avg
            // Using alpha = 0.2 = 1/5
            (latency_us + 4 * current_avg) / 5
        };
        self.state.avg_latency_us.store(new_avg, Ordering::Relaxed);

        // If health was Degraded, consider promoting to Healthy
        let current_health = self.state.health.load(Ordering::Relaxed);
        if current_health == HealthStatus::Degraded as u8 {
            // Simple heuristic: promote after successful request
            self.state
                .health
                .store(HealthStatus::Healthy as u8, Ordering::Relaxed);
        }
    }

    /// Record a failed request
    ///
    /// Increments failure counters. The caller is responsible for deciding
    /// whether to enter cooldown based on failure rate.
    pub fn record_failure(&self) {
        self.state.total_requests.fetch_add(1, Ordering::Relaxed);
        self.state.fail_requests.fetch_add(1, Ordering::Relaxed);
        self.state.fails_this_minute.fetch_add(1, Ordering::Relaxed);
        self.state
            .last_request_at
            .store(current_timestamp(), Ordering::Relaxed);

        // Mark as degraded (caller can escalate to Unhealthy/Cooldown if needed)
        self.state
            .health
            .store(HealthStatus::Degraded as u8, Ordering::Relaxed);
    }

    /// Enter cooldown state
    ///
    /// Sets health to Cooldown and configures cooldown end time.
    ///
    /// # Arguments
    ///
    /// * `duration_secs` - Cooldown duration in seconds
    pub fn enter_cooldown(&self, duration_secs: u64) {
        let cooldown_until = current_timestamp() + duration_secs;
        self.state
            .cooldown_until
            .store(cooldown_until, Ordering::Relaxed);
        self.state
            .health
            .store(HealthStatus::Cooldown as u8, Ordering::Relaxed);
    }
}

/// Get current Unix timestamp in seconds
///
/// Returns the number of seconds since UNIX_EPOCH.
/// Panics if system time is before UNIX_EPOCH (should never happen on modern systems).
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time before UNIX_EPOCH")
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::providers::openai::OpenAIProvider;

    async fn create_test_provider() -> Provider {
        // Use a properly formatted test key (sk- prefix required by OpenAI provider validation)
        let openai = OpenAIProvider::with_api_key("sk-test-key-for-unit-testing-only")
            .await
            .expect("Failed to create OpenAI provider");
        Provider::OpenAI(openai)
    }

    #[tokio::test]
    async fn test_deployment_creation() {
        let provider = create_test_provider().await;
        let deployment = Deployment::new(
            "test-deployment".to_string(),
            provider,
            "gpt-4-turbo".to_string(),
            "gpt-4".to_string(),
        );

        assert_eq!(deployment.id, "test-deployment");
        assert_eq!(deployment.model, "gpt-4-turbo");
        assert_eq!(deployment.model_name, "gpt-4");
        assert_eq!(deployment.config.weight, 1);
        assert_eq!(deployment.tags.len(), 0);
    }

    #[tokio::test]
    async fn test_deployment_with_config() {
        let provider = create_test_provider().await;
        let config = DeploymentConfig {
            tpm_limit: Some(100_000),
            rpm_limit: Some(500),
            weight: 2,
            priority: 1,
            ..Default::default()
        };

        let deployment = Deployment::new(
            "test-deployment".to_string(),
            provider,
            "gpt-4-turbo".to_string(),
            "gpt-4".to_string(),
        )
        .with_config(config);

        assert_eq!(deployment.config.tpm_limit, Some(100_000));
        assert_eq!(deployment.config.rpm_limit, Some(500));
        assert_eq!(deployment.config.weight, 2);
        assert_eq!(deployment.config.priority, 1);
    }

    #[tokio::test]
    async fn test_deployment_with_tags() {
        let provider = create_test_provider().await;
        let deployment = Deployment::new(
            "test-deployment".to_string(),
            provider,
            "gpt-4-turbo".to_string(),
            "gpt-4".to_string(),
        )
        .with_tags(vec!["production".to_string(), "fast".to_string()]);

        assert_eq!(deployment.tags.len(), 2);
        assert!(deployment.tags.contains(&"production".to_string()));
        assert!(deployment.tags.contains(&"fast".to_string()));
    }

    #[tokio::test]
    async fn test_record_success() {
        let provider = create_test_provider().await;
        let deployment = Deployment::new(
            "test-deployment".to_string(),
            provider,
            "gpt-4-turbo".to_string(),
            "gpt-4".to_string(),
        );

        deployment.record_success(100, 5000);

        assert_eq!(deployment.state.total_requests.load(Ordering::Relaxed), 1);
        assert_eq!(deployment.state.success_requests.load(Ordering::Relaxed), 1);
        assert_eq!(deployment.state.tpm_current.load(Ordering::Relaxed), 100);
        assert_eq!(deployment.state.rpm_current.load(Ordering::Relaxed), 1);
        assert_eq!(
            deployment.state.avg_latency_us.load(Ordering::Relaxed),
            5000
        );
    }

    #[tokio::test]
    async fn test_record_failure() {
        let provider = create_test_provider().await;
        let deployment = Deployment::new(
            "test-deployment".to_string(),
            provider,
            "gpt-4-turbo".to_string(),
            "gpt-4".to_string(),
        );

        deployment.record_failure();

        assert_eq!(deployment.state.total_requests.load(Ordering::Relaxed), 1);
        assert_eq!(deployment.state.fail_requests.load(Ordering::Relaxed), 1);
        assert_eq!(
            deployment.state.fails_this_minute.load(Ordering::Relaxed),
            1
        );
        assert_eq!(
            deployment.state.health.load(Ordering::Relaxed),
            HealthStatus::Degraded as u8
        );
    }

    #[tokio::test]
    async fn test_cooldown() {
        let provider = create_test_provider().await;
        let deployment = Deployment::new(
            "test-deployment".to_string(),
            provider,
            "gpt-4-turbo".to_string(),
            "gpt-4".to_string(),
        );

        // Initially not in cooldown
        assert!(!deployment.is_in_cooldown());

        // Enter cooldown for 60 seconds
        deployment.enter_cooldown(60);

        // Should be in cooldown now
        assert!(deployment.is_in_cooldown());
        assert_eq!(
            deployment.state.health.load(Ordering::Relaxed),
            HealthStatus::Cooldown as u8
        );

        // Enter cooldown with 0 duration (effectively immediate exit)
        deployment.enter_cooldown(0);
        assert!(!deployment.is_in_cooldown());
    }

    #[tokio::test]
    async fn test_is_healthy() {
        let provider = create_test_provider().await;
        let deployment = Deployment::new(
            "test-deployment".to_string(),
            provider,
            "gpt-4-turbo".to_string(),
            "gpt-4".to_string(),
        );

        // Starts with Healthy status
        assert!(deployment.is_healthy());

        // Set to Unknown - not healthy
        deployment
            .state
            .health
            .store(HealthStatus::Unknown as u8, Ordering::Relaxed);
        assert!(!deployment.is_healthy());

        // Set to Healthy
        deployment
            .state
            .health
            .store(HealthStatus::Healthy as u8, Ordering::Relaxed);
        assert!(deployment.is_healthy());

        // Set to Degraded - still considered healthy for routing
        deployment
            .state
            .health
            .store(HealthStatus::Degraded as u8, Ordering::Relaxed);
        assert!(deployment.is_healthy());

        // Set to Unhealthy
        deployment
            .state
            .health
            .store(HealthStatus::Unhealthy as u8, Ordering::Relaxed);
        assert!(!deployment.is_healthy());

        // Set to Cooldown
        deployment
            .state
            .health
            .store(HealthStatus::Cooldown as u8, Ordering::Relaxed);
        assert!(!deployment.is_healthy());
    }

    #[tokio::test]
    async fn test_reset_minute() {
        let provider = create_test_provider().await;
        let deployment = Deployment::new(
            "test-deployment".to_string(),
            provider,
            "gpt-4-turbo".to_string(),
            "gpt-4".to_string(),
        );

        // Record some activity
        deployment.record_success(100, 5000);
        deployment.record_failure();

        assert_eq!(deployment.state.tpm_current.load(Ordering::Relaxed), 100);
        assert_eq!(deployment.state.rpm_current.load(Ordering::Relaxed), 1);
        assert_eq!(
            deployment.state.fails_this_minute.load(Ordering::Relaxed),
            1
        );

        // Reset minute
        deployment.state.reset_minute();

        assert_eq!(deployment.state.tpm_current.load(Ordering::Relaxed), 0);
        assert_eq!(deployment.state.rpm_current.load(Ordering::Relaxed), 0);
        assert_eq!(
            deployment.state.fails_this_minute.load(Ordering::Relaxed),
            0
        );

        // Lifetime counters should not be reset
        assert_eq!(deployment.state.total_requests.load(Ordering::Relaxed), 2);
    }

    #[tokio::test]
    async fn test_exponential_moving_average() {
        let provider = create_test_provider().await;
        let deployment = Deployment::new(
            "test-deployment".to_string(),
            provider,
            "gpt-4-turbo".to_string(),
            "gpt-4".to_string(),
        );

        // First request: latency should be set directly
        deployment.record_success(100, 10000);
        assert_eq!(
            deployment.state.avg_latency_us.load(Ordering::Relaxed),
            10000
        );

        // Second request: should calculate EMA
        // EMA = (new + 4*old) / 5 = (20000 + 4*10000) / 5 = 60000 / 5 = 12000
        deployment.record_success(100, 20000);
        assert_eq!(
            deployment.state.avg_latency_us.load(Ordering::Relaxed),
            12000
        );
    }

    #[test]
    fn test_health_status_conversion() {
        assert_eq!(HealthStatus::from(0), HealthStatus::Unknown);
        assert_eq!(HealthStatus::from(1), HealthStatus::Healthy);
        assert_eq!(HealthStatus::from(2), HealthStatus::Degraded);
        assert_eq!(HealthStatus::from(3), HealthStatus::Unhealthy);
        assert_eq!(HealthStatus::from(4), HealthStatus::Cooldown);
        assert_eq!(HealthStatus::from(99), HealthStatus::Unknown);

        assert_eq!(u8::from(HealthStatus::Unknown), 0);
        assert_eq!(u8::from(HealthStatus::Healthy), 1);
        assert_eq!(u8::from(HealthStatus::Degraded), 2);
        assert_eq!(u8::from(HealthStatus::Unhealthy), 3);
        assert_eq!(u8::from(HealthStatus::Cooldown), 4);
    }
}
