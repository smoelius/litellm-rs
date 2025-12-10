//! Health monitoring system for providers and services
//!
//! This module provides comprehensive health monitoring capabilities including
//! provider health checks, service availability monitoring, and health-based routing.

use crate::utils::error::{GatewayError, Result};
use crate::utils::error_recovery::CircuitBreaker;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::time::interval;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Health status levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Service is fully operational
    Healthy,
    /// Service is operational but degraded
    Degraded,
    /// Service is unhealthy but may recover
    Unhealthy,
    /// Service is completely unavailable
    Down,
}

impl HealthStatus {
    /// Check if the status allows requests
    pub fn allows_requests(&self) -> bool {
        matches!(self, HealthStatus::Healthy | HealthStatus::Degraded)
    }

    /// Get numeric score for routing (higher is better)
    pub fn score(&self) -> u32 {
        match self {
            HealthStatus::Healthy => 100,
            HealthStatus::Degraded => 70,
            HealthStatus::Unhealthy => 30,
            HealthStatus::Down => 0,
        }
    }
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    /// Health status
    pub status: HealthStatus,
    /// Response time in milliseconds
    pub response_time_ms: u64,
    /// Timestamp of the check
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Additional details about the health check
    pub details: Option<String>,
    /// Error message if unhealthy
    pub error: Option<String>,
    /// Metrics collected during health check
    pub metrics: HashMap<String, f64>,
}

impl HealthCheckResult {
    /// Create a healthy result
    pub fn healthy(response_time_ms: u64) -> Self {
        Self {
            status: HealthStatus::Healthy,
            response_time_ms,
            timestamp: chrono::Utc::now(),
            details: None,
            error: None,
            metrics: HashMap::new(),
        }
    }

    /// Create an unhealthy result
    pub fn unhealthy(error: String, response_time_ms: u64) -> Self {
        Self {
            status: HealthStatus::Unhealthy,
            response_time_ms,
            timestamp: chrono::Utc::now(),
            details: None,
            error: Some(error),
            metrics: HashMap::new(),
        }
    }

    /// Create a degraded result
    pub fn degraded(reason: String, response_time_ms: u64) -> Self {
        Self {
            status: HealthStatus::Degraded,
            response_time_ms,
            timestamp: chrono::Utc::now(),
            details: Some(reason),
            error: None,
            metrics: HashMap::new(),
        }
    }
}

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
        let successful_checks = self.history.iter()
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

/// Health monitor configuration
#[derive(Debug, Clone)]
pub struct HealthMonitorConfig {
    /// Interval between health checks
    pub check_interval: Duration,
    /// Timeout for individual health checks
    pub check_timeout: Duration,
    /// Number of failures before marking as unhealthy
    pub failure_threshold: u32,
    /// Number of successes needed to recover from unhealthy
    pub recovery_threshold: u32,
    /// Whether to enable automatic health checks
    pub auto_check_enabled: bool,
    /// Maximum response time before considering degraded
    pub degraded_threshold_ms: u64,
}

impl Default for HealthMonitorConfig {
    fn default() -> Self {
        Self {
            check_interval: Duration::from_secs(30),
            check_timeout: Duration::from_secs(10),
            failure_threshold: 3,
            recovery_threshold: 2,
            auto_check_enabled: true,
            degraded_threshold_ms: 2000,
        }
    }
}

/// Health monitor for tracking provider and service health
pub struct HealthMonitor {
    config: HealthMonitorConfig,
    provider_health: Arc<RwLock<HashMap<String, ProviderHealth>>>,
    /// Circuit breakers stored as Arc for shared access without Clone
    circuit_breakers: Arc<RwLock<HashMap<String, Arc<CircuitBreaker>>>>,
    check_tasks: Arc<RwLock<HashMap<String, tokio::task::JoinHandle<()>>>>,
}

impl HealthMonitor {
    /// Create a new health monitor
    pub fn new(config: HealthMonitorConfig) -> Self {
        Self {
            config,
            provider_health: Arc::new(RwLock::new(HashMap::new())),
            circuit_breakers: Arc::new(RwLock::new(HashMap::new())),
            check_tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a provider for health monitoring
    pub async fn register_provider(&self, provider_id: String) {
        info!("Registering provider for health monitoring: {}", provider_id);

        // Initialize provider health
        if let Ok(mut health) = self.provider_health.write() {
            health.insert(provider_id.clone(), ProviderHealth::new(provider_id.clone()));
        } else {
            error!("Failed to acquire write lock for provider health");
            return;
        }

        // Initialize circuit breaker (wrapped in Arc for shared access)
        if let Ok(mut breakers) = self.circuit_breakers.write() {
            let breaker_config = crate::utils::error_recovery::CircuitBreakerConfig::default();
            breakers.insert(provider_id.clone(), Arc::new(CircuitBreaker::new(breaker_config)));
        } else {
            error!("Failed to acquire write lock for circuit breakers");
            return;
        }

        // Start health check task if auto-check is enabled
        if self.config.auto_check_enabled {
            self.start_health_check_task(provider_id).await;
        }
    }

    /// Start health check task for a provider
    async fn start_health_check_task(&self, provider_id: String) {
        let provider_health = self.provider_health.clone();
        let check_interval = self.config.check_interval;
        let check_timeout = self.config.check_timeout;
        let degraded_threshold = self.config.degraded_threshold_ms;

        let task = tokio::spawn(async move {
            let mut interval = interval(check_interval);
            
            loop {
                interval.tick().await;
                
                debug!("Running health check for provider: {}", provider_id);
                
                let start_time = Instant::now();
                let result = match tokio::time::timeout(check_timeout, Self::perform_health_check(&provider_id)).await {
                    Ok(Ok(response_time)) => {
                        let response_time_ms = response_time.as_millis() as u64;
                        if response_time_ms > degraded_threshold {
                            HealthCheckResult::degraded(
                                format!("High latency: {}ms", response_time_ms),
                                response_time_ms
                            )
                        } else {
                            HealthCheckResult::healthy(response_time_ms)
                        }
                    }
                    Ok(Err(error)) => {
                        let elapsed = start_time.elapsed().as_millis() as u64;
                        HealthCheckResult::unhealthy(error.to_string(), elapsed)
                    }
                    Err(_) => {
                        let elapsed = start_time.elapsed().as_millis() as u64;
                        HealthCheckResult::unhealthy("Health check timeout".to_string(), elapsed)
                    }
                };

                // Update provider health
                if let Ok(mut health_map) = provider_health.write() {
                    if let Some(provider_health) = health_map.get_mut(&provider_id) {
                        provider_health.update(result);
                        debug!("Updated health for {}: {:?}", provider_id, provider_health.status);
                    }
                }
            }
        });

        // Store task handle
        if let Ok(mut tasks) = self.check_tasks.write() {
            tasks.insert(provider_id, task);
        } else {
            error!("Failed to acquire write lock for check tasks");
        }
    }

    /// Perform actual health check for a provider
    async fn perform_health_check(provider_id: &str) -> Result<Duration> {
        let start_time = Instant::now();
        
        // In a real implementation, this would call the provider's health endpoint
        // For now, simulate a health check with variable response times
        let delay = match provider_id {
            id if id.contains("openai") => Duration::from_millis(100 + rand::random::<u64>() % 200),
            id if id.contains("anthropic") => Duration::from_millis(150 + rand::random::<u64>() % 300),
            _ => Duration::from_millis(50 + rand::random::<u64>() % 100),
        };

        tokio::time::sleep(delay).await;

        // Simulate occasional failures
        if rand::random::<f64>() < 0.05 {
            return Err(GatewayError::External("Simulated health check failure".to_string()));
        }

        Ok(start_time.elapsed())
    }

    /// Get health status for a provider
    pub fn get_provider_health(&self, provider_id: &str) -> Option<ProviderHealth> {
        self.provider_health
            .read()
            .ok()
            .and_then(|health| health.get(provider_id).cloned())
    }

    /// Get health status for all providers
    pub fn get_all_provider_health(&self) -> HashMap<String, ProviderHealth> {
        self.provider_health
            .read()
            .map(|health| health.clone())
            .unwrap_or_default()
    }

    /// Get healthy providers sorted by routing weight
    pub fn get_healthy_providers(&self) -> Vec<(String, f64)> {
        let health_map = match self.provider_health.read() {
            Ok(map) => map,
            Err(_) => return Vec::new(),
        };
        let mut providers: Vec<_> = health_map
            .iter()
            .filter(|(_, health)| health.is_available())
            .map(|(id, health)| (id.clone(), health.routing_weight()))
            .collect();

        providers.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        providers
    }

    /// Manually update provider health
    pub fn update_provider_health(&self, provider_id: &str, result: HealthCheckResult) {
        if let Ok(mut health_map) = self.provider_health.write() {
            if let Some(provider_health) = health_map.get_mut(provider_id) {
                provider_health.update(result);
                info!("Manually updated health for {}: {:?}", provider_id, provider_health.status);
            }
        }
    }

    /// Get circuit breaker for a provider
    pub fn get_circuit_breaker(&self, provider_id: &str) -> Option<Arc<CircuitBreaker>> {
        self.circuit_breakers
            .read()
            .ok()
            .and_then(|breakers| breakers.get(provider_id).cloned())
    }

    /// Shutdown health monitoring for all providers
    pub async fn shutdown(&self) {
        info!("Shutting down health monitoring");

        // Cancel all health check tasks
        let tasks = match self.check_tasks.write() {
            Ok(mut task_map) => task_map.drain().map(|(_, task)| task).collect::<Vec<_>>(),
            Err(_) => {
                error!("Failed to acquire write lock for check tasks during shutdown");
                return;
            }
        };

        for task in tasks {
            task.abort();
        }

        info!("Health monitoring shutdown complete");
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
        let healthy_providers = self.provider_health.values()
            .filter(|h| h.status == HealthStatus::Healthy)
            .count();
        let available_providers = self.provider_health.values()
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
            let healthy = self.provider_health.values()
                .filter(|h| h.status == HealthStatus::Healthy)
                .count() as f64;
            let available = self.provider_health.values()
                .filter(|h| h.is_available())
                .count() as f64;
            
            metrics.insert("total_providers".to_string(), total);
            metrics.insert("healthy_providers".to_string(), healthy);
            metrics.insert("available_providers".to_string(), available);
            metrics.insert("health_percentage".to_string(), (healthy / total) * 100.0);
            metrics.insert("availability_percentage".to_string(), (available / total) * 100.0);

            // Average response time across all providers
            let avg_response_time: f64 = self.provider_health.values()
                .map(|h| h.avg_response_time_ms)
                .sum::<f64>() / total;
            metrics.insert("avg_response_time_ms".to_string(), avg_response_time);
        }

        metrics
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status_scoring() {
        assert_eq!(HealthStatus::Healthy.score(), 100);
        assert_eq!(HealthStatus::Degraded.score(), 70);
        assert_eq!(HealthStatus::Unhealthy.score(), 30);
        assert_eq!(HealthStatus::Down.score(), 0);

        assert!(HealthStatus::Healthy.allows_requests());
        assert!(HealthStatus::Degraded.allows_requests());
        assert!(!HealthStatus::Unhealthy.allows_requests());
        assert!(!HealthStatus::Down.allows_requests());
    }

    #[test]
    fn test_provider_health_update() {
        let mut provider = ProviderHealth::new("test-provider".to_string());
        
        // Start healthy
        assert_eq!(provider.status, HealthStatus::Healthy);
        assert_eq!(provider.consecutive_failures, 0);
        
        // Add unhealthy result
        let unhealthy_result = HealthCheckResult::unhealthy("test error".to_string(), 1000);
        provider.update(unhealthy_result);
        
        assert_eq!(provider.status, HealthStatus::Unhealthy);
        assert_eq!(provider.consecutive_failures, 1);
        
        // Add healthy result
        let healthy_result = HealthCheckResult::healthy(500);
        provider.update(healthy_result);
        
        assert_eq!(provider.status, HealthStatus::Healthy);
        assert_eq!(provider.consecutive_failures, 0);
    }

    #[test]
    fn test_provider_routing_weight() {
        let mut provider = ProviderHealth::new("test-provider".to_string());
        
        // Healthy provider should have high weight
        let healthy_result = HealthCheckResult::healthy(100);
        provider.update(healthy_result);
        let weight = provider.routing_weight();
        assert!(weight > 0.8);
        
        // Unhealthy provider should have zero weight
        provider.status = HealthStatus::Down;
        let weight = provider.routing_weight();
        assert_eq!(weight, 0.0);
    }

    #[tokio::test]
    async fn test_health_monitor_registration() {
        let config = HealthMonitorConfig {
            auto_check_enabled: false,
            ..Default::default()
        };
        let monitor = HealthMonitor::new(config);
        
        monitor.register_provider("test-provider".to_string()).await;
        
        let health = monitor.get_provider_health("test-provider");
        assert!(health.is_some());
        assert_eq!(health.unwrap().provider_id, "test-provider");
    }

    #[test]
    fn test_system_health() {
        let mut providers = HashMap::new();
        providers.insert("provider1".to_string(), ProviderHealth::new("provider1".to_string()));
        
        let mut provider2 = ProviderHealth::new("provider2".to_string());
        provider2.status = HealthStatus::Unhealthy;
        providers.insert("provider2".to_string(), provider2);
        
        let system_health = SystemHealth::new(providers);
        assert_eq!(system_health.overall_status(), HealthStatus::Degraded);
        
        let metrics = system_health.metrics();
        assert_eq!(metrics.get("total_providers"), Some(&2.0));
        assert_eq!(metrics.get("healthy_providers"), Some(&1.0));
    }
}