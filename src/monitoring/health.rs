//! Health checking system
//!
//! This module provides comprehensive health checking for all system components.

#![allow(dead_code)]

use crate::storage::StorageLayer;
use crate::utils::error::Result;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, error};

/// Health checker for monitoring system component health
#[derive(Debug)]
pub struct HealthChecker {
    /// Storage layer for health data
    storage: Arc<StorageLayer>,
    /// Consolidated health data - single lock for related data
    health_data: Arc<RwLock<HealthData>>,
    /// Whether health checking is active - using AtomicBool for lock-free access
    active: AtomicBool,
}

/// Consolidated health data - single lock for all health-related state
#[derive(Debug)]
struct HealthData {
    /// Component health status
    components: HashMap<String, ComponentHealth>,
    /// Overall health status
    overall: HealthStatus,
}

/// Overall system health status
#[derive(Debug, Clone, serde::Serialize)]
pub struct HealthStatus {
    /// Whether the system is overall healthy
    pub overall_healthy: bool,
    /// Timestamp of last health check
    pub last_check: chrono::DateTime<chrono::Utc>,
    /// Individual component health
    pub components: HashMap<String, ComponentHealth>,
    /// System uptime
    pub uptime_seconds: u64,
    /// Health check summary
    pub summary: HealthSummary,
}

/// Individual component health
#[derive(Debug, Clone, serde::Serialize)]
pub struct ComponentHealth {
    /// Component name
    pub name: String,
    /// Whether the component is healthy
    pub healthy: bool,
    /// Health status message
    pub status: String,
    /// Last check timestamp
    pub last_check: chrono::DateTime<chrono::Utc>,
    /// Response time for health check
    pub response_time_ms: u64,
    /// Error message (if unhealthy)
    pub error: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Health check summary
#[derive(Debug, Clone, serde::Serialize)]
pub struct HealthSummary {
    /// Total number of components
    pub total_components: usize,
    /// Number of healthy components
    pub healthy_components: usize,
    /// Number of unhealthy components
    pub unhealthy_components: usize,
    /// Health percentage
    pub health_percentage: f64,
}

/// Health check configuration for a component
#[derive(Debug, Clone)]
pub struct HealthCheckConfig {
    /// Component name
    pub name: String,
    /// Check interval
    pub interval: Duration,
    /// Timeout for health check
    pub timeout: Duration,
    /// Number of retries
    pub retries: u32,
    /// Whether this component is critical
    pub critical: bool,
}

impl HealthChecker {
    /// Create a new health checker
    pub async fn new(storage: Arc<StorageLayer>) -> Result<Self> {
        let initial_health = HealthStatus {
            overall_healthy: true,
            last_check: chrono::Utc::now(),
            components: HashMap::new(),
            uptime_seconds: 0,
            summary: HealthSummary {
                total_components: 0,
                healthy_components: 0,
                unhealthy_components: 0,
                health_percentage: 100.0,
            },
        };

        Ok(Self {
            storage,
            health_data: Arc::new(RwLock::new(HealthData {
                components: HashMap::new(),
                overall: initial_health,
            })),
            active: AtomicBool::new(false),
        })
    }

    /// Start health checking
    pub async fn start(&self) -> Result<()> {
        debug!("Starting health checker");

        self.active.store(true, Ordering::Release);

        // Start health check tasks
        self.start_health_check_tasks().await;

        Ok(())
    }

    /// Stop health checking
    pub async fn stop(&self) -> Result<()> {
        debug!("Stopping health checker");
        self.active.store(false, Ordering::Release);
        Ok(())
    }

    /// Check if health checker is active
    #[inline]
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Acquire)
    }

    /// Get current health status
    pub async fn get_status(&self) -> Result<HealthStatus> {
        let data = self.health_data.read();
        Ok(data.overall.clone())
    }

    /// Check all components
    pub async fn check_all(&self) -> Result<HealthStatus> {
        debug!("Running comprehensive health check");

        let start_time = Instant::now();
        let mut components = HashMap::new();

        // Check storage layer
        let storage_health = self.check_storage().await;
        components.insert("storage".to_string(), storage_health);

        // Check database
        let database_health = self.check_database().await;
        components.insert("database".to_string(), database_health);

        // Check Redis
        let redis_health = self.check_redis().await;
        components.insert("redis".to_string(), redis_health);

        // Check file storage
        let file_storage_health = self.check_file_storage().await;
        components.insert("file_storage".to_string(), file_storage_health);

        // Check vector database (if configured)
        if self.storage.vector().is_some() {
            let vector_health = self.check_vector_database().await;
            components.insert("vector_database".to_string(), vector_health);
        }

        // Calculate overall health
        let healthy_components = components.values().filter(|c| c.healthy).count();
        let total_components = components.len();
        let overall_healthy = healthy_components == total_components;
        let health_percentage = (healthy_components as f64 / total_components as f64) * 100.0;

        let health_status = HealthStatus {
            overall_healthy,
            last_check: chrono::Utc::now(),
            components: components.clone(),
            uptime_seconds: start_time.elapsed().as_secs(),
            summary: HealthSummary {
                total_components,
                healthy_components,
                unhealthy_components: total_components - healthy_components,
                health_percentage,
            },
        };

        // Update stored health status - single lock for both updates
        {
            let mut data = self.health_data.write();
            data.overall = health_status.clone();
            data.components = components;
        }

        Ok(health_status)
    }

    /// Check storage layer health
    async fn check_storage(&self) -> ComponentHealth {
        let start_time = Instant::now();

        match self.storage.health_check().await {
            Ok(storage_status) => ComponentHealth {
                name: "storage".to_string(),
                healthy: storage_status.overall,
                status: if storage_status.overall {
                    "healthy"
                } else {
                    "degraded"
                }
                .to_string(),
                last_check: chrono::Utc::now(),
                response_time_ms: start_time.elapsed().as_millis() as u64,
                error: None,
                metadata: serde_json::to_value(&storage_status)
                    .unwrap_or_default()
                    .as_object()
                    .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                    .unwrap_or_default(),
            },
            Err(e) => ComponentHealth {
                name: "storage".to_string(),
                healthy: false,
                status: "unhealthy".to_string(),
                last_check: chrono::Utc::now(),
                response_time_ms: start_time.elapsed().as_millis() as u64,
                error: Some(e.to_string()),
                metadata: HashMap::new(),
            },
        }
    }

    /// Check database health
    async fn check_database(&self) -> ComponentHealth {
        let start_time = Instant::now();

        match self.storage.db().health_check().await {
            Ok(()) => {
                let stats = self.storage.db().stats();
                let mut metadata = HashMap::new();
                metadata.insert(
                    "pool_size".to_string(),
                    serde_json::Value::Number(stats.size.into()),
                );
                metadata.insert(
                    "idle_connections".to_string(),
                    serde_json::Value::Number(stats.idle.into()),
                );

                ComponentHealth {
                    name: "database".to_string(),
                    healthy: true,
                    status: "healthy".to_string(),
                    last_check: chrono::Utc::now(),
                    response_time_ms: start_time.elapsed().as_millis() as u64,
                    error: None,
                    metadata,
                }
            }
            Err(e) => ComponentHealth {
                name: "database".to_string(),
                healthy: false,
                status: "unhealthy".to_string(),
                last_check: chrono::Utc::now(),
                response_time_ms: start_time.elapsed().as_millis() as u64,
                error: Some(e.to_string()),
                metadata: HashMap::new(),
            },
        }
    }

    /// Check Redis health
    async fn check_redis(&self) -> ComponentHealth {
        let start_time = Instant::now();

        match self.storage.redis().health_check().await {
            Ok(()) => ComponentHealth {
                name: "redis".to_string(),
                healthy: true,
                status: "healthy".to_string(),
                last_check: chrono::Utc::now(),
                response_time_ms: start_time.elapsed().as_millis() as u64,
                error: None,
                metadata: HashMap::new(),
            },
            Err(e) => ComponentHealth {
                name: "redis".to_string(),
                healthy: false,
                status: "unhealthy".to_string(),
                last_check: chrono::Utc::now(),
                response_time_ms: start_time.elapsed().as_millis() as u64,
                error: Some(e.to_string()),
                metadata: HashMap::new(),
            },
        }
    }

    /// Check file storage health
    async fn check_file_storage(&self) -> ComponentHealth {
        let start_time = Instant::now();

        match self.storage.files().health_check().await {
            Ok(()) => ComponentHealth {
                name: "file_storage".to_string(),
                healthy: true,
                status: "healthy".to_string(),
                last_check: chrono::Utc::now(),
                response_time_ms: start_time.elapsed().as_millis() as u64,
                error: None,
                metadata: HashMap::new(),
            },
            Err(e) => ComponentHealth {
                name: "file_storage".to_string(),
                healthy: false,
                status: "unhealthy".to_string(),
                last_check: chrono::Utc::now(),
                response_time_ms: start_time.elapsed().as_millis() as u64,
                error: Some(e.to_string()),
                metadata: HashMap::new(),
            },
        }
    }

    /// Check vector database health
    async fn check_vector_database(&self) -> ComponentHealth {
        let start_time = Instant::now();

        if let Some(vector_store) = self.storage.vector() {
            match vector_store.health_check().await {
                Ok(()) => ComponentHealth {
                    name: "vector_database".to_string(),
                    healthy: true,
                    status: "healthy".to_string(),
                    last_check: chrono::Utc::now(),
                    response_time_ms: start_time.elapsed().as_millis() as u64,
                    error: None,
                    metadata: HashMap::new(),
                },
                Err(e) => ComponentHealth {
                    name: "vector_database".to_string(),
                    healthy: false,
                    status: "unhealthy".to_string(),
                    last_check: chrono::Utc::now(),
                    response_time_ms: start_time.elapsed().as_millis() as u64,
                    error: Some(e.to_string()),
                    metadata: HashMap::new(),
                },
            }
        } else {
            ComponentHealth {
                name: "vector_database".to_string(),
                healthy: true,
                status: "not_configured".to_string(),
                last_check: chrono::Utc::now(),
                response_time_ms: 0,
                error: None,
                metadata: HashMap::new(),
            }
        }
    }

    /// Check external provider health
    async fn check_provider(&self, provider_name: &str, provider_url: &str) -> ComponentHealth {
        let start_time = Instant::now();

        // Simple HTTP health check
        match reqwest::Client::new()
            .get(provider_url)
            .timeout(Duration::from_secs(10))
            .send()
            .await
        {
            Ok(response) => {
                let healthy = response.status().is_success();
                ComponentHealth {
                    name: provider_name.to_string(),
                    healthy,
                    status: if healthy { "healthy" } else { "degraded" }.to_string(),
                    last_check: chrono::Utc::now(),
                    response_time_ms: start_time.elapsed().as_millis() as u64,
                    error: if healthy {
                        None
                    } else {
                        Some(format!("HTTP {}", response.status()))
                    },
                    metadata: {
                        let mut metadata = HashMap::new();
                        metadata.insert(
                            "status_code".to_string(),
                            serde_json::Value::Number(response.status().as_u16().into()),
                        );
                        metadata
                    },
                }
            }
            Err(e) => ComponentHealth {
                name: provider_name.to_string(),
                healthy: false,
                status: "unhealthy".to_string(),
                last_check: chrono::Utc::now(),
                response_time_ms: start_time.elapsed().as_millis() as u64,
                error: Some(e.to_string()),
                metadata: HashMap::new(),
            },
        }
    }

    /// Start background health check tasks
    async fn start_health_check_tasks(&self) {
        let health_checker = self.clone();

        // Main health check task
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));

            loop {
                interval.tick().await;

                if !health_checker.is_active() {
                    break;
                }

                if let Err(e) = health_checker.check_all().await {
                    error!("Health check failed: {}", e);
                }
            }
        });

        // Component-specific health checks can be added here
        // with different intervals for different components
    }

    /// Get component health by name
    pub async fn get_component_health(&self, component_name: &str) -> Option<ComponentHealth> {
        let data = self.health_data.read();
        data.components.get(component_name).cloned()
    }

    /// Check if a specific component is healthy
    pub async fn is_component_healthy(&self, component_name: &str) -> bool {
        if let Some(component) = self.get_component_health(component_name).await {
            component.healthy
        } else {
            false
        }
    }

    /// Get unhealthy components
    pub async fn get_unhealthy_components(&self) -> Vec<ComponentHealth> {
        let data = self.health_data.read();
        data.components
            .values()
            .filter(|component| !component.healthy)
            .cloned()
            .collect()
    }
}

impl Clone for HealthChecker {
    fn clone(&self) -> Self {
        Self {
            storage: self.storage.clone(),
            health_data: self.health_data.clone(),
            active: AtomicBool::new(self.active.load(Ordering::Acquire)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_health_creation() {
        let health = ComponentHealth {
            name: "test_component".to_string(),
            healthy: true,
            status: "healthy".to_string(),
            last_check: chrono::Utc::now(),
            response_time_ms: 50,
            error: None,
            metadata: HashMap::new(),
        };

        assert!(health.healthy);
        assert_eq!(health.name, "test_component");
        assert_eq!(health.response_time_ms, 50);
    }

    #[test]
    fn test_health_summary_calculation() {
        let summary = HealthSummary {
            total_components: 5,
            healthy_components: 4,
            unhealthy_components: 1,
            health_percentage: 80.0,
        };

        assert_eq!(summary.total_components, 5);
        assert_eq!(summary.healthy_components, 4);
        assert_eq!(summary.health_percentage, 80.0);
    }

    #[test]
    fn test_health_check_config() {
        let config = HealthCheckConfig {
            name: "database".to_string(),
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(5),
            retries: 3,
            critical: true,
        };

        assert_eq!(config.name, "database");
        assert!(config.critical);
        assert_eq!(config.retries, 3);
    }
}
