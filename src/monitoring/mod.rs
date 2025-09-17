//! Monitoring and observability system
//!
//! This module provides comprehensive monitoring, metrics, and observability functionality.

pub mod alerts;
pub mod health;
pub mod metrics;

use crate::config::MonitoringConfig;
use crate::storage::StorageLayer;
use crate::utils::error::Result;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tracing::{debug, info, warn};

/// Main monitoring system
#[derive(Clone)]
#[allow(dead_code)]
pub struct MonitoringSystem {
    /// Monitoring configuration
    config: Arc<MonitoringConfig>,
    /// Storage layer for persistence
    storage: Arc<StorageLayer>,
    /// Metrics collector
    metrics: Arc<metrics::MetricsCollector>,
    /// Health checker
    health: Arc<health::HealthChecker>,
    /// Alert manager
    alerts: Option<Arc<alerts::AlertManager>>,
    /// System start time
    start_time: Instant,
}

/// System metrics snapshot
#[derive(Debug, Clone, serde::Serialize)]
pub struct SystemMetrics {
    /// Timestamp of the snapshot
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Request metrics
    pub requests: RequestMetrics,
    /// Provider metrics
    pub providers: ProviderMetrics,
    /// System resource metrics
    pub system: SystemResourceMetrics,
    /// Error metrics
    pub errors: ErrorMetrics,
    /// Performance metrics
    pub performance: PerformanceMetrics,
}

/// Request-related metrics
#[derive(Debug, Clone, serde::Serialize)]
pub struct RequestMetrics {
    /// Total requests processed
    pub total_requests: u64,
    /// Requests per second (current)
    pub requests_per_second: f64,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    /// 95th percentile response time
    pub p95_response_time_ms: f64,
    /// 99th percentile response time
    pub p99_response_time_ms: f64,
    /// Success rate (percentage)
    pub success_rate: f64,
    /// Requests by status code
    pub status_codes: std::collections::HashMap<u16, u64>,
    /// Requests by endpoint
    pub endpoints: std::collections::HashMap<String, u64>,
}

/// Provider-related metrics
#[derive(Debug, Clone, serde::Serialize)]
pub struct ProviderMetrics {
    /// Total provider requests
    pub total_provider_requests: u64,
    /// Provider success rates
    pub provider_success_rates: std::collections::HashMap<String, f64>,
    /// Provider response times
    pub provider_response_times: std::collections::HashMap<String, f64>,
    /// Provider error counts
    pub provider_errors: std::collections::HashMap<String, u64>,
    /// Provider usage distribution
    pub provider_usage: std::collections::HashMap<String, u64>,
    /// Token usage by provider
    pub token_usage: std::collections::HashMap<String, u64>,
    /// Cost by provider
    pub costs: std::collections::HashMap<String, f64>,
}

/// System resource metrics
#[derive(Debug, Clone, serde::Serialize)]
pub struct SystemResourceMetrics {
    /// CPU usage percentage
    pub cpu_usage: f64,
    /// Memory usage in bytes
    pub memory_usage: u64,
    /// Memory usage percentage
    pub memory_usage_percent: f64,
    /// Disk usage in bytes
    pub disk_usage: u64,
    /// Disk usage percentage
    pub disk_usage_percent: f64,
    /// Network bytes received
    pub network_bytes_in: u64,
    /// Network bytes sent
    pub network_bytes_out: u64,
    /// Active connections
    pub active_connections: u32,
    /// Database connections
    pub database_connections: u32,
    /// Redis connections
    pub redis_connections: u32,
}

/// Error-related metrics
#[derive(Debug, Clone, serde::Serialize)]
pub struct ErrorMetrics {
    /// Total errors
    pub total_errors: u64,
    /// Error rate (errors per second)
    pub error_rate: f64,
    /// Errors by type
    pub error_types: std::collections::HashMap<String, u64>,
    /// Errors by endpoint
    pub error_endpoints: std::collections::HashMap<String, u64>,
    /// Critical errors
    pub critical_errors: u64,
    /// Warning count
    pub warnings: u64,
}

/// Performance-related metrics
#[derive(Debug, Clone, serde::Serialize)]
pub struct PerformanceMetrics {
    /// Cache hit rate
    pub cache_hit_rate: f64,
    /// Cache miss rate
    pub cache_miss_rate: f64,
    /// Database query time (average)
    pub avg_db_query_time_ms: f64,
    /// Queue depth
    pub queue_depth: u32,
    /// Throughput (requests per second)
    pub throughput: f64,
    /// Latency percentiles
    pub latency_percentiles: LatencyPercentiles,
}

/// Latency percentile metrics
#[derive(Debug, Clone, serde::Serialize)]
pub struct LatencyPercentiles {
    pub p50: f64,
    pub p90: f64,
    pub p95: f64,
    pub p99: f64,
    pub p999: f64,
}

/// Alert severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
    Emergency,
}

impl std::fmt::Display for AlertSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlertSeverity::Info => write!(f, "INFO"),
            AlertSeverity::Warning => write!(f, "WARNING"),
            AlertSeverity::Critical => write!(f, "CRITICAL"),
            AlertSeverity::Emergency => write!(f, "EMERGENCY"),
        }
    }
}

/// Alert information
#[derive(Debug, Clone, serde::Serialize)]
pub struct Alert {
    /// Alert ID
    pub id: String,
    /// Alert severity
    pub severity: AlertSeverity,
    /// Alert title
    pub title: String,
    /// Alert description
    pub description: String,
    /// Alert timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Alert source
    pub source: String,
    /// Alert metadata
    pub metadata: serde_json::Value,
    /// Whether the alert is resolved
    pub resolved: bool,
}

#[allow(dead_code)]
impl MonitoringSystem {
    /// Create a new monitoring system
    pub async fn new(config: &MonitoringConfig, storage: Arc<StorageLayer>) -> Result<Self> {
        info!("Initializing monitoring system");

        let config = Arc::new(config.clone());

        // Initialize metrics collector
        let metrics = Arc::new(metrics::MetricsCollector::new(&config).await?);

        // Initialize health checker
        let health = Arc::new(health::HealthChecker::new(storage.clone()).await?);

        // Initialize alert manager (if enabled)
        let alerts = None; // TODO: Add alerting config to MonitoringConfig

        info!("Monitoring system initialized successfully");

        Ok(Self {
            config,
            storage,
            metrics,
            health,
            alerts,
            start_time: Instant::now(),
        })
    }

    /// Start the monitoring system
    pub async fn start(&self) -> Result<()> {
        info!("Starting monitoring system");

        // Start metrics collection
        self.metrics.start().await?;

        // Start health checking
        self.health.start().await?;

        // Start alert manager
        if let Some(alerts) = &self.alerts {
            alerts.start().await?;
        }

        // Start background tasks
        self.start_background_tasks().await?;

        info!("Monitoring system started successfully");
        Ok(())
    }

    /// Stop the monitoring system
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping monitoring system");

        // Stop metrics collection
        self.metrics.stop().await?;

        // Stop health checking
        self.health.stop().await?;

        // Stop alert manager
        if let Some(alerts) = &self.alerts {
            alerts.stop().await?;
        }

        info!("Monitoring system stopped");
        Ok(())
    }

    /// Get current system metrics
    pub async fn get_metrics(&self) -> Result<SystemMetrics> {
        debug!("Collecting system metrics");

        let timestamp = chrono::Utc::now();

        // Collect metrics from various sources
        let requests = self.collect_request_metrics().await?;
        let providers = self.collect_provider_metrics().await?;
        let system = self.collect_system_metrics().await?;
        let errors = self.collect_error_metrics().await?;
        let performance = self.collect_performance_metrics().await?;

        Ok(SystemMetrics {
            timestamp,
            requests,
            providers,
            system,
            errors,
            performance,
        })
    }

    /// Record a request metric
    pub async fn record_request(
        &self,
        method: &str,
        path: &str,
        status_code: u16,
        response_time: Duration,
        user_id: Option<uuid::Uuid>,
        api_key_id: Option<uuid::Uuid>,
    ) -> Result<()> {
        self.metrics
            .record_request(
                method,
                path,
                status_code,
                response_time,
                user_id,
                api_key_id,
            )
            .await
    }

    /// Record a provider request metric
    pub async fn record_provider_request(
        &self,
        provider: &str,
        model: &str,
        tokens_used: u32,
        cost: f64,
        response_time: Duration,
        success: bool,
    ) -> Result<()> {
        self.metrics
            .record_provider_request(provider, model, tokens_used, cost, response_time, success)
            .await
    }

    /// Record an error
    pub async fn record_error(
        &self,
        error_type: &str,
        error_message: &str,
        context: Option<serde_json::Value>,
    ) -> Result<()> {
        self.metrics
            .record_error(error_type, error_message, context)
            .await
    }

    /// Send an alert
    pub async fn send_alert(&self, alert: Alert) -> Result<()> {
        if let Some(alerts) = &self.alerts {
            alerts.send_alert(alert).await
        } else {
            warn!("Alert manager not configured, skipping alert");
            Ok(())
        }
    }

    /// Get system uptime
    pub fn get_uptime(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Get health status
    pub async fn get_health_status(&self) -> Result<health::HealthStatus> {
        self.health.get_status().await
    }

    /// Start background monitoring tasks
    async fn start_background_tasks(&self) -> Result<()> {
        let monitoring = self.clone();

        // Metrics aggregation task
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                if let Err(e) = monitoring.aggregate_metrics().await {
                    warn!("Failed to aggregate metrics: {}", e);
                }
            }
        });

        // Health check task
        let monitoring = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            loop {
                interval.tick().await;
                if let Err(e) = monitoring.run_health_checks().await {
                    warn!("Health check failed: {}", e);
                }
            }
        });

        // Alert processing task
        if self.alerts.is_some() {
            let monitoring = self.clone();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_secs(10));
                loop {
                    interval.tick().await;
                    if let Err(e) = monitoring.process_alerts().await {
                        warn!("Failed to process alerts: {}", e);
                    }
                }
            });
        }

        Ok(())
    }

    /// Aggregate metrics for storage
    async fn aggregate_metrics(&self) -> Result<()> {
        debug!("Aggregating metrics");

        let _metrics = self.get_metrics().await?;

        // Store metrics in database
        // TODO: SystemMetrics and RequestMetrics are different types, need to convert or use different method
        // self.storage.db().store_metrics(&metrics).await?;

        // Store metrics in time series database (if configured)
        // TODO: Implement time series storage

        Ok(())
    }

    /// Run health checks
    async fn run_health_checks(&self) -> Result<()> {
        debug!("Running health checks");

        let health_status = self.health.check_all().await?;

        // Check for unhealthy components and send alerts
        if !health_status.overall_healthy {
            let alert = Alert {
                id: uuid::Uuid::new_v4().to_string(),
                severity: AlertSeverity::Critical,
                title: "System Health Check Failed".to_string(),
                description: format!(
                    "One or more system components are unhealthy: {:?}",
                    health_status
                ),
                timestamp: chrono::Utc::now(),
                source: "health_checker".to_string(),
                metadata: serde_json::to_value(&health_status).unwrap_or_default(),
                resolved: false,
            };

            self.send_alert(alert).await?;
        }

        Ok(())
    }

    /// Process pending alerts
    async fn process_alerts(&self) -> Result<()> {
        if let Some(alerts) = &self.alerts {
            alerts.process_pending().await?;
        }
        Ok(())
    }

    /// Collect request metrics
    async fn collect_request_metrics(&self) -> Result<RequestMetrics> {
        self.metrics.get_request_metrics().await
    }

    /// Collect provider metrics
    async fn collect_provider_metrics(&self) -> Result<ProviderMetrics> {
        self.metrics.get_provider_metrics().await
    }

    /// Collect system resource metrics
    async fn collect_system_metrics(&self) -> Result<SystemResourceMetrics> {
        self.metrics.get_system_metrics().await
    }

    /// Collect error metrics
    async fn collect_error_metrics(&self) -> Result<ErrorMetrics> {
        self.metrics.get_error_metrics().await
    }

    /// Collect performance metrics
    async fn collect_performance_metrics(&self) -> Result<PerformanceMetrics> {
        self.metrics.get_performance_metrics().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alert_creation() {
        let alert = Alert {
            id: "test-alert".to_string(),
            severity: AlertSeverity::Warning,
            title: "Test Alert".to_string(),
            description: "This is a test alert".to_string(),
            timestamp: chrono::Utc::now(),
            source: "test".to_string(),
            metadata: serde_json::json!({"test": true}),
            resolved: false,
        };

        assert_eq!(alert.severity, AlertSeverity::Warning);
        assert!(!alert.resolved);
    }

    #[test]
    fn test_system_metrics_structure() {
        let metrics = SystemMetrics {
            timestamp: chrono::Utc::now(),
            requests: RequestMetrics {
                total_requests: 1000,
                requests_per_second: 10.5,
                avg_response_time_ms: 150.0,
                p95_response_time_ms: 300.0,
                p99_response_time_ms: 500.0,
                success_rate: 99.5,
                status_codes: std::collections::HashMap::new(),
                endpoints: std::collections::HashMap::new(),
            },
            providers: ProviderMetrics {
                total_provider_requests: 800,
                provider_success_rates: std::collections::HashMap::new(),
                provider_response_times: std::collections::HashMap::new(),
                provider_errors: std::collections::HashMap::new(),
                provider_usage: std::collections::HashMap::new(),
                token_usage: std::collections::HashMap::new(),
                costs: std::collections::HashMap::new(),
            },
            system: SystemResourceMetrics {
                cpu_usage: 45.2,
                memory_usage: 1024 * 1024 * 512, // 512MB
                memory_usage_percent: 25.0,
                disk_usage: 1024 * 1024 * 1024 * 10, // 10GB
                disk_usage_percent: 50.0,
                network_bytes_in: 1024 * 1024,
                network_bytes_out: 1024 * 512,
                active_connections: 100,
                database_connections: 10,
                redis_connections: 5,
            },
            errors: ErrorMetrics {
                total_errors: 5,
                error_rate: 0.1,
                error_types: std::collections::HashMap::new(),
                error_endpoints: std::collections::HashMap::new(),
                critical_errors: 1,
                warnings: 4,
            },
            performance: PerformanceMetrics {
                cache_hit_rate: 85.5,
                cache_miss_rate: 14.5,
                avg_db_query_time_ms: 25.0,
                queue_depth: 0,
                throughput: 10.5,
                latency_percentiles: LatencyPercentiles {
                    p50: 100.0,
                    p90: 200.0,
                    p95: 300.0,
                    p99: 500.0,
                    p999: 800.0,
                },
            },
        };

        assert_eq!(metrics.requests.total_requests, 1000);
        assert_eq!(metrics.system.cpu_usage, 45.2);
        assert_eq!(metrics.errors.critical_errors, 1);
    }
}
