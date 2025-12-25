//! Provider health tracking
//!
//! This module provides health tracking for individual providers including
//! health history, metrics calculation, and routing weights.

use super::types::{HealthCheckResult, HealthStatus};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

/// Provider health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderHealth {
    /// Provider identifier
    pub provider_id: String,
    /// Current health status
    pub status: HealthStatus,
    /// Last health check result
    pub last_check: Option<HealthCheckResult>,
    /// Health check history (last N checks) - uses VecDeque for O(1) pop_front
    pub history: VecDeque<HealthCheckResult>,
    /// Average response time over recent checks
    pub avg_response_time_ms: f64,
    /// Success rate over recent checks
    pub success_rate: f64,
    /// Number of consecutive failures
    pub consecutive_failures: u32,
    /// When the provider was last healthy
    pub last_healthy: Option<chrono::DateTime<chrono::Utc>>,
    /// Custom health metrics
    pub metrics: HashMap<String, f64>,
}

impl ProviderHealth {
    /// Create new provider health tracking
    pub fn new(provider_id: String) -> Self {
        Self {
            provider_id,
            status: HealthStatus::Healthy,
            last_check: None,
            history: VecDeque::new(),
            avg_response_time_ms: 0.0,
            success_rate: 100.0,
            consecutive_failures: 0,
            last_healthy: Some(chrono::Utc::now()),
            metrics: HashMap::new(),
        }
    }

    /// Update with new health check result
    pub fn update(&mut self, result: HealthCheckResult) {
        // Update status
        self.status = result.status.clone();

        // Update consecutive failures
        if result.status == HealthStatus::Healthy {
            self.consecutive_failures = 0;
            self.last_healthy = Some(result.timestamp);
        } else {
            self.consecutive_failures += 1;
        }

        // Add to history (keep last 50 results)
        self.history.push_back(result.clone());
        if self.history.len() > 50 {
            self.history.pop_front();
        }

        // Update last check
        self.last_check = Some(result);

        // Recalculate metrics
        self.recalculate_metrics();
    }

    /// Recalculate aggregate metrics
    fn recalculate_metrics(&mut self) {
        if self.history.is_empty() {
            return;
        }

        // Calculate average response time
        let total_time: u64 = self.history.iter().map(|h| h.response_time_ms).sum();
        self.avg_response_time_ms = total_time as f64 / self.history.len() as f64;

        // Calculate success rate
        let successful_checks = self
            .history
            .iter()
            .filter(|h| h.status == HealthStatus::Healthy || h.status == HealthStatus::Degraded)
            .count();
        self.success_rate = (successful_checks as f64 / self.history.len() as f64) * 100.0;
    }

    /// Check if provider should be considered available for routing
    pub fn is_available(&self) -> bool {
        self.status.allows_requests() && self.consecutive_failures < 5
    }

    /// Get routing weight based on health
    pub fn routing_weight(&self) -> f64 {
        if !self.is_available() {
            return 0.0;
        }

        let status_weight = self.status.score() as f64 / 100.0;
        let success_weight = self.success_rate / 100.0;
        let latency_weight = if self.avg_response_time_ms > 0.0 {
            1.0 / (1.0 + self.avg_response_time_ms / 1000.0)
        } else {
            1.0
        };

        (status_weight + success_weight + latency_weight) / 3.0
    }
}

/// System health aggregator
pub struct SystemHealth {
    provider_health: HashMap<String, ProviderHealth>,
    last_updated: chrono::DateTime<chrono::Utc>,
}

impl SystemHealth {
    /// Create system health snapshot
    pub fn new(provider_health: HashMap<String, ProviderHealth>) -> Self {
        Self {
            provider_health,
            last_updated: chrono::Utc::now(),
        }
    }

    /// Get overall system health status
    pub fn overall_status(&self) -> HealthStatus {
        if self.provider_health.is_empty() {
            return HealthStatus::Down;
        }

        let total_providers = self.provider_health.len();
        let healthy_providers = self
            .provider_health
            .values()
            .filter(|h| h.status == HealthStatus::Healthy)
            .count();
        let available_providers = self
            .provider_health
            .values()
            .filter(|h| h.is_available())
            .count();

        if available_providers == 0 {
            HealthStatus::Down
        } else if healthy_providers == total_providers {
            HealthStatus::Healthy
        } else if available_providers >= total_providers / 2 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Unhealthy
        }
    }

    /// Get system health metrics
    pub fn metrics(&self) -> HashMap<String, f64> {
        let mut metrics = HashMap::new();

        if !self.provider_health.is_empty() {
            let total = self.provider_health.len() as f64;
            let healthy = self
                .provider_health
                .values()
                .filter(|h| h.status == HealthStatus::Healthy)
                .count() as f64;
            let available = self
                .provider_health
                .values()
                .filter(|h| h.is_available())
                .count() as f64;

            metrics.insert("total_providers".to_string(), total);
            metrics.insert("healthy_providers".to_string(), healthy);
            metrics.insert("available_providers".to_string(), available);
            metrics.insert("health_percentage".to_string(), (healthy / total) * 100.0);
            metrics.insert(
                "availability_percentage".to_string(),
                (available / total) * 100.0,
            );

            // Average response time across all providers
            let avg_response_time: f64 = self
                .provider_health
                .values()
                .map(|h| h.avg_response_time_ms)
                .sum::<f64>()
                / total;
            metrics.insert("avg_response_time_ms".to_string(), avg_response_time);
        }

        metrics
    }
}
