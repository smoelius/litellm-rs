//! Deployment models for the Gateway
//!
//! This module defines deployment and provider configuration structures.

use super::{HealthStatus, Metadata};
use crate::config::ProviderConfig;
use serde::{Deserialize, Serialize};

use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use uuid::Uuid;

/// Provider deployment configuration
#[derive(Debug, Clone)]
pub struct Deployment {
    /// Deployment metadata
    pub metadata: Metadata,
    /// Deployment configuration
    pub config: ProviderConfig,
    /// Current health status
    pub health: Arc<DeploymentHealth>,
    /// Runtime metrics
    pub metrics: Arc<DeploymentMetrics>,
    /// Deployment state
    pub state: DeploymentState,
    /// Tags for routing
    pub tags: Vec<String>,
    /// Weight for load balancing
    pub weight: f32,
    /// Rate limits
    pub rate_limits: Option<DeploymentRateLimits>,
    /// Cost configuration
    pub cost_config: Option<DeploymentCostConfig>,
}

/// Deployment state
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeploymentState {
    /// Deployment is active and healthy
    Active,
    /// Deployment is active but degraded
    Degraded,
    /// Deployment is temporarily disabled
    Disabled,
    /// Deployment is draining (no new requests)
    Draining,
    /// Deployment is in maintenance mode
    Maintenance,
    /// Deployment failed health checks
    Failed,
}

/// Deployment health information
#[derive(Debug)]
pub struct DeploymentHealth {
    /// Current health status
    pub status: parking_lot::RwLock<HealthStatus>,
    /// Last health check timestamp
    pub last_check: AtomicU64,
    /// Consecutive failure count
    pub failure_count: AtomicU32,
    /// Last failure timestamp
    pub last_failure: AtomicU64,
    /// Average response time in milliseconds
    pub avg_response_time: AtomicU64,
    /// Success rate (0-10000 for 0.00% to 100.00%)
    pub success_rate: AtomicU32,
    /// Circuit breaker state
    pub circuit_breaker: parking_lot::RwLock<CircuitBreakerState>,
}

/// Circuit breaker state
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CircuitBreakerState {
    /// Circuit is closed (normal operation)
    Closed,
    /// Circuit is open (failing fast)
    Open,
    /// Circuit is half-open (testing recovery)
    HalfOpen,
}

/// Deployment runtime metrics
#[derive(Debug)]
pub struct DeploymentMetrics {
    /// Total requests processed
    pub total_requests: AtomicU64,
    /// Successful requests
    pub successful_requests: AtomicU64,
    /// Failed requests
    pub failed_requests: AtomicU64,
    /// Total tokens processed
    pub total_tokens: AtomicU64,
    /// Total cost incurred
    pub total_cost: parking_lot::RwLock<f64>,
    /// Active connections
    pub active_connections: AtomicU32,
    /// Queue size
    pub queue_size: AtomicU32,
    /// Last request timestamp
    pub last_request: AtomicU64,
    /// Request rate (requests per minute)
    pub request_rate: AtomicU32,
    /// Token rate (tokens per minute)
    pub token_rate: AtomicU32,
    /// Average response time
    pub avg_response_time: AtomicU64,
    /// P95 response time
    pub p95_response_time: AtomicU64,
    /// P99 response time
    pub p99_response_time: AtomicU64,
}

/// Deployment rate limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentRateLimits {
    /// Requests per minute
    pub rpm: Option<u32>,
    /// Tokens per minute
    pub tpm: Option<u32>,
    /// Requests per day
    pub rpd: Option<u32>,
    /// Tokens per day
    pub tpd: Option<u32>,
    /// Concurrent requests
    pub concurrent: Option<u32>,
    /// Burst allowance
    pub burst: Option<u32>,
}

/// Deployment cost configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentCostConfig {
    /// Cost per input token
    pub input_cost_per_token: Option<f64>,
    /// Cost per output token
    pub output_cost_per_token: Option<f64>,
    /// Cost per request
    pub cost_per_request: Option<f64>,
    /// Cost per image
    pub cost_per_image: Option<f64>,
    /// Cost per audio second
    pub cost_per_audio_second: Option<f64>,
    /// Currency
    pub currency: String,
    /// Billing model
    pub billing_model: BillingModel,
}

/// Billing model
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BillingModel {
    /// Pay-per-use billing
    PayPerUse,
    /// Subscription billing
    Subscription,
    /// Prepaid billing
    Prepaid,
    /// Free billing
    Free,
}

/// Deployment snapshot for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentSnapshot {
    /// Deployment ID
    pub id: Uuid,
    /// Provider name
    pub name: String,
    /// Provider type
    pub provider_type: String,
    /// Model name
    pub model: String,
    /// Current state
    pub state: DeploymentState,
    /// Health status
    pub health_status: HealthStatus,
    /// Weight
    pub weight: f32,
    /// Tags
    pub tags: Vec<String>,
    /// Metrics snapshot
    pub metrics: DeploymentMetricsSnapshot,
    /// Last updated
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Deployment metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentMetricsSnapshot {
    /// Total requests
    pub total_requests: u64,
    /// Successful requests
    pub successful_requests: u64,
    /// Failed requests
    pub failed_requests: u64,
    /// Success rate percentage
    pub success_rate: f64,
    /// Total tokens
    pub total_tokens: u64,
    /// Total cost
    pub total_cost: f64,
    /// Active connections
    pub active_connections: u32,
    /// Average response time in milliseconds
    pub avg_response_time: u64,
    /// P95 response time in milliseconds
    pub p95_response_time: u64,
    /// P99 response time in milliseconds
    pub p99_response_time: u64,
    /// Request rate (RPM)
    pub request_rate: u32,
    /// Token rate (TPM)
    pub token_rate: u32,
}

impl Deployment {
    /// Create a new deployment
    pub fn new(config: ProviderConfig) -> Self {
        let weight = config.weight;
        let tags = config.tags.clone();

        Self {
            metadata: Metadata::new(),
            config,
            health: Arc::new(DeploymentHealth::new()),
            metrics: Arc::new(DeploymentMetrics::new()),
            state: DeploymentState::Active,
            tags,
            weight,
            rate_limits: None,
            cost_config: None,
        }
    }

    /// Get deployment ID
    pub fn id(&self) -> Uuid {
        self.metadata.id
    }

    /// Get deployment name
    pub fn name(&self) -> &str {
        &self.config.name
    }

    /// Get provider type
    pub fn provider_type(&self) -> &str {
        &self.config.provider_type
    }

    /// Check if deployment is available for requests
    pub fn is_available(&self) -> bool {
        matches!(
            self.state,
            DeploymentState::Active | DeploymentState::Degraded
        ) && !matches!(
            *self.health.circuit_breaker.read(),
            CircuitBreakerState::Open
        )
    }

    /// Get current health status
    pub fn health_status(&self) -> HealthStatus {
        *self.health.status.read()
    }

    /// Update health status
    pub fn update_health(&self, status: HealthStatus, response_time_ms: Option<u64>) {
        *self.health.status.write() = status;
        self.health
            .last_check
            .store(chrono::Utc::now().timestamp() as u64, Ordering::Relaxed);

        if let Some(response_time) = response_time_ms {
            self.health
                .avg_response_time
                .store(response_time, Ordering::Relaxed);
        }

        match status {
            HealthStatus::Healthy => {
                self.health.failure_count.store(0, Ordering::Relaxed);
            }
            HealthStatus::Unhealthy => {
                self.health.failure_count.fetch_add(1, Ordering::Relaxed);
                self.health
                    .last_failure
                    .store(chrono::Utc::now().timestamp() as u64, Ordering::Relaxed);
            }
            _ => {}
        }
    }

    /// Record request metrics
    pub fn record_request(&self, success: bool, tokens: u32, cost: f64, response_time_ms: u64) {
        self.metrics.total_requests.fetch_add(1, Ordering::Relaxed);

        if success {
            self.metrics
                .successful_requests
                .fetch_add(1, Ordering::Relaxed);
        } else {
            self.metrics.failed_requests.fetch_add(1, Ordering::Relaxed);
        }

        self.metrics
            .total_tokens
            .fetch_add(tokens as u64, Ordering::Relaxed);

        {
            let mut total_cost = self.metrics.total_cost.write();
            *total_cost += cost;
        }

        self.metrics
            .last_request
            .store(chrono::Utc::now().timestamp() as u64, Ordering::Relaxed);

        // Update response time metrics (simplified)
        self.metrics
            .avg_response_time
            .store(response_time_ms, Ordering::Relaxed);
    }

    /// Get current metrics snapshot
    pub fn metrics_snapshot(&self) -> DeploymentMetricsSnapshot {
        let total_requests = self.metrics.total_requests.load(Ordering::Relaxed);
        let successful_requests = self.metrics.successful_requests.load(Ordering::Relaxed);
        let failed_requests = self.metrics.failed_requests.load(Ordering::Relaxed);

        let success_rate = if total_requests > 0 {
            (successful_requests as f64 / total_requests as f64) * 100.0
        } else {
            0.0
        };

        DeploymentMetricsSnapshot {
            total_requests,
            successful_requests,
            failed_requests,
            success_rate,
            total_tokens: self.metrics.total_tokens.load(Ordering::Relaxed),
            total_cost: *self.metrics.total_cost.read(),
            active_connections: self.metrics.active_connections.load(Ordering::Relaxed),
            avg_response_time: self.metrics.avg_response_time.load(Ordering::Relaxed),
            p95_response_time: self.metrics.p95_response_time.load(Ordering::Relaxed),
            p99_response_time: self.metrics.p99_response_time.load(Ordering::Relaxed),
            request_rate: self.metrics.request_rate.load(Ordering::Relaxed),
            token_rate: self.metrics.token_rate.load(Ordering::Relaxed),
        }
    }

    /// Create deployment snapshot
    pub fn snapshot(&self) -> DeploymentSnapshot {
        DeploymentSnapshot {
            id: self.id(),
            name: self.name().to_string(),
            provider_type: self.provider_type().to_string(),
            model: self.config.api_key.clone(), // This should be model name
            state: self.state.clone(),
            health_status: self.health_status(),
            weight: self.weight,
            tags: self.tags.clone(),
            metrics: self.metrics_snapshot(),
            updated_at: chrono::Utc::now(),
        }
    }
}

impl Default for DeploymentHealth {
    fn default() -> Self {
        Self::new()
    }
}

impl DeploymentHealth {
    /// Create new deployment health
    pub fn new() -> Self {
        Self {
            status: parking_lot::RwLock::new(HealthStatus::Unknown),
            last_check: AtomicU64::new(0),
            failure_count: AtomicU32::new(0),
            last_failure: AtomicU64::new(0),
            avg_response_time: AtomicU64::new(0),
            success_rate: AtomicU32::new(10000), // 100.00%
            circuit_breaker: parking_lot::RwLock::new(CircuitBreakerState::Closed),
        }
    }
}

impl Default for DeploymentMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl DeploymentMetrics {
    /// Create new deployment metrics
    pub fn new() -> Self {
        Self {
            total_requests: AtomicU64::new(0),
            successful_requests: AtomicU64::new(0),
            failed_requests: AtomicU64::new(0),
            total_tokens: AtomicU64::new(0),
            total_cost: parking_lot::RwLock::new(0.0),
            active_connections: AtomicU32::new(0),
            queue_size: AtomicU32::new(0),
            last_request: AtomicU64::new(0),
            request_rate: AtomicU32::new(0),
            token_rate: AtomicU32::new(0),
            avg_response_time: AtomicU64::new(0),
            p95_response_time: AtomicU64::new(0),
            p99_response_time: AtomicU64::new(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ProviderConfig;
    use std::collections::HashMap;

    #[test]
    fn test_deployment_creation() {
        let config = ProviderConfig {
            name: "test-provider".to_string(),
            provider_type: "openai".to_string(),
            api_key: "test-key".to_string(),
            base_url: None,
            models: vec!["gpt-4".to_string()],
            timeout: 30,
            max_retries: 3,
            organization: None,
            api_version: None,
            project: None,
            weight: 1.0,
            rpm: 1000,
            tpm: 10000,
            enabled: true,
            max_concurrent_requests: 10,
            retry: crate::config::RetryConfig::default(),
            health_check: crate::config::HealthCheckConfig::default(),
            settings: HashMap::new(),
            tags: vec!["test".to_string()],
        };

        let deployment = Deployment::new(config);
        assert_eq!(deployment.name(), "test-provider");
        assert_eq!(deployment.provider_type(), "openai");
        assert!(deployment.is_available());
    }

    #[test]
    fn test_metrics_recording() {
        let config = ProviderConfig {
            name: "test-provider".to_string(),
            provider_type: "openai".to_string(),
            api_key: "test-key".to_string(),
            base_url: None,
            api_version: None,
            organization: None,
            project: None,
            weight: 1.0,
            rpm: 1000,
            tpm: 10000,
            max_concurrent_requests: 10,
            timeout: 30,
            max_retries: 3,
            retry: crate::config::RetryConfig::default(),
            health_check: crate::config::HealthCheckConfig::default(),
            settings: HashMap::new(),
            models: vec![],
            tags: vec![],
            enabled: true,
        };

        let deployment = Deployment::new(config);
        deployment.record_request(true, 100, 0.01, 250);

        let snapshot = deployment.metrics_snapshot();
        assert_eq!(snapshot.total_requests, 1);
        assert_eq!(snapshot.successful_requests, 1);
        assert_eq!(snapshot.total_tokens, 100);
        assert_eq!(snapshot.total_cost, 0.01);
        assert_eq!(snapshot.success_rate, 100.0);
    }
}
