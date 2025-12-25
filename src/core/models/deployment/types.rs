//! Core deployment type definitions
//!
//! This module defines the main deployment structure and related types.

use crate::config::ProviderConfig;
use crate::core::models::{HealthStatus, Metadata};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::core::models::deployment::health::DeploymentHealth;
use crate::core::models::deployment::metrics::DeploymentMetrics;

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
