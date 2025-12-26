//! Monitoring configuration

use super::*;
use serde::{Deserialize, Serialize};

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MonitoringConfig {
    /// Metrics configuration
    #[serde(default)]
    pub metrics: MetricsConfig,
    /// Tracing configuration
    #[serde(default)]
    pub tracing: TracingConfig,
    /// Health check configuration
    #[serde(default)]
    pub health: HealthConfig,
}

#[allow(dead_code)]
impl MonitoringConfig {
    /// Merge monitoring configurations
    pub fn merge(mut self, other: Self) -> Self {
        self.metrics = self.metrics.merge(other.metrics);
        self.tracing = self.tracing.merge(other.tracing);
        self.health = self.health.merge(other.health);
        self
    }
}

/// Metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Enable metrics
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Metrics port
    #[serde(default = "default_metrics_port")]
    pub port: u16,
    /// Metrics path
    #[serde(default = "default_metrics_path")]
    pub path: String,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            port: default_metrics_port(),
            path: default_metrics_path(),
        }
    }
}

#[allow(dead_code)]
impl MetricsConfig {
    /// Merge metrics configurations
    pub fn merge(mut self, other: Self) -> Self {
        if !other.enabled {
            self.enabled = other.enabled;
        }
        if other.port != default_metrics_port() {
            self.port = other.port;
        }
        if other.path != default_metrics_path() {
            self.path = other.path;
        }
        self
    }
}

/// Tracing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingConfig {
    /// Enable tracing
    #[serde(default)]
    pub enabled: bool,
    /// Tracing endpoint
    pub endpoint: Option<String>,
    /// Service name
    #[serde(default = "default_service_name")]
    pub service_name: String,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            endpoint: None,
            service_name: default_service_name(),
        }
    }
}

#[allow(dead_code)]
impl TracingConfig {
    /// Merge tracing configurations
    pub fn merge(mut self, other: Self) -> Self {
        if other.enabled {
            self.enabled = other.enabled;
        }
        if other.endpoint.is_some() {
            self.endpoint = other.endpoint;
        }
        if other.service_name != default_service_name() {
            self.service_name = other.service_name;
        }
        self
    }
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthConfig {
    /// Health check path
    #[serde(default = "default_health_path")]
    pub path: String,
    /// Enable detailed health checks
    #[serde(default = "default_true")]
    pub detailed: bool,
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            path: default_health_path(),
            detailed: true,
        }
    }
}

#[allow(dead_code)]
impl HealthConfig {
    /// Merge health configurations
    pub fn merge(mut self, other: Self) -> Self {
        if other.path != default_health_path() {
            self.path = other.path;
        }
        if !other.detailed {
            self.detailed = other.detailed;
        }
        self
    }
}

fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== MetricsConfig Tests ====================

    #[test]
    fn test_metrics_config_default() {
        let config = MetricsConfig::default();
        assert!(config.enabled);
        assert_eq!(config.port, 9090);
        assert_eq!(config.path, "/metrics");
    }

    #[test]
    fn test_metrics_config_structure() {
        let config = MetricsConfig {
            enabled: false,
            port: 8080,
            path: "/prometheus".to_string(),
        };
        assert!(!config.enabled);
        assert_eq!(config.port, 8080);
        assert_eq!(config.path, "/prometheus");
    }

    #[test]
    fn test_metrics_config_serialization() {
        let config = MetricsConfig {
            enabled: true,
            port: 9100,
            path: "/stats".to_string(),
        };
        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["enabled"], true);
        assert_eq!(json["port"], 9100);
        assert_eq!(json["path"], "/stats");
    }

    #[test]
    fn test_metrics_config_deserialization() {
        let json = r#"{"enabled": false, "port": 3000, "path": "/api/metrics"}"#;
        let config: MetricsConfig = serde_json::from_str(json).unwrap();
        assert!(!config.enabled);
        assert_eq!(config.port, 3000);
    }

    #[test]
    fn test_metrics_config_merge_disabled() {
        let base = MetricsConfig::default();
        let other = MetricsConfig {
            enabled: false,
            port: 9090,
            path: "/metrics".to_string(),
        };
        let merged = base.merge(other);
        assert!(!merged.enabled);
    }

    #[test]
    fn test_metrics_config_merge_port() {
        let base = MetricsConfig::default();
        let other = MetricsConfig {
            enabled: true,
            port: 8888,
            path: "/metrics".to_string(),
        };
        let merged = base.merge(other);
        assert_eq!(merged.port, 8888);
    }

    #[test]
    fn test_metrics_config_clone() {
        let config = MetricsConfig::default();
        let cloned = config.clone();
        assert_eq!(config.enabled, cloned.enabled);
        assert_eq!(config.port, cloned.port);
    }

    // ==================== TracingConfig Tests ====================

    #[test]
    fn test_tracing_config_default() {
        let config = TracingConfig::default();
        assert!(!config.enabled);
        assert!(config.endpoint.is_none());
        assert_eq!(config.service_name, "litellm-rs");
    }

    #[test]
    fn test_tracing_config_structure() {
        let config = TracingConfig {
            enabled: true,
            endpoint: Some("http://jaeger:14268".to_string()),
            service_name: "my-gateway".to_string(),
        };
        assert!(config.enabled);
        assert_eq!(config.endpoint, Some("http://jaeger:14268".to_string()));
    }

    #[test]
    fn test_tracing_config_serialization() {
        let config = TracingConfig {
            enabled: true,
            endpoint: Some("http://otel:4317".to_string()),
            service_name: "api-gateway".to_string(),
        };
        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["enabled"], true);
        assert_eq!(json["endpoint"], "http://otel:4317");
    }

    #[test]
    fn test_tracing_config_deserialization() {
        let json = r#"{"enabled": true, "endpoint": "http://zipkin:9411", "service_name": "tracer"}"#;
        let config: TracingConfig = serde_json::from_str(json).unwrap();
        assert!(config.enabled);
        assert_eq!(config.service_name, "tracer");
    }

    #[test]
    fn test_tracing_config_merge_enabled() {
        let base = TracingConfig::default();
        let other = TracingConfig {
            enabled: true,
            endpoint: None,
            service_name: "litellm-gateway".to_string(),
        };
        let merged = base.merge(other);
        assert!(merged.enabled);
    }

    #[test]
    fn test_tracing_config_merge_endpoint() {
        let base = TracingConfig::default();
        let other = TracingConfig {
            enabled: false,
            endpoint: Some("http://collector:4317".to_string()),
            service_name: "litellm-gateway".to_string(),
        };
        let merged = base.merge(other);
        assert_eq!(merged.endpoint, Some("http://collector:4317".to_string()));
    }

    #[test]
    fn test_tracing_config_clone() {
        let config = TracingConfig::default();
        let cloned = config.clone();
        assert_eq!(config.enabled, cloned.enabled);
        assert_eq!(config.service_name, cloned.service_name);
    }

    // ==================== HealthConfig Tests ====================

    #[test]
    fn test_health_config_default() {
        let config = HealthConfig::default();
        assert_eq!(config.path, "/health");
        assert!(config.detailed);
    }

    #[test]
    fn test_health_config_structure() {
        let config = HealthConfig {
            path: "/api/health".to_string(),
            detailed: false,
        };
        assert_eq!(config.path, "/api/health");
        assert!(!config.detailed);
    }

    #[test]
    fn test_health_config_serialization() {
        let config = HealthConfig {
            path: "/status".to_string(),
            detailed: true,
        };
        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["path"], "/status");
        assert_eq!(json["detailed"], true);
    }

    #[test]
    fn test_health_config_deserialization() {
        let json = r#"{"path": "/ready", "detailed": false}"#;
        let config: HealthConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.path, "/ready");
        assert!(!config.detailed);
    }

    #[test]
    fn test_health_config_merge_path() {
        let base = HealthConfig::default();
        let other = HealthConfig {
            path: "/healthz".to_string(),
            detailed: true,
        };
        let merged = base.merge(other);
        assert_eq!(merged.path, "/healthz");
    }

    #[test]
    fn test_health_config_merge_detailed() {
        let base = HealthConfig::default();
        let other = HealthConfig {
            path: "/health".to_string(),
            detailed: false,
        };
        let merged = base.merge(other);
        assert!(!merged.detailed);
    }

    #[test]
    fn test_health_config_clone() {
        let config = HealthConfig::default();
        let cloned = config.clone();
        assert_eq!(config.path, cloned.path);
        assert_eq!(config.detailed, cloned.detailed);
    }

    // ==================== MonitoringConfig Tests ====================

    #[test]
    fn test_monitoring_config_default() {
        let config = MonitoringConfig::default();
        assert!(config.metrics.enabled);
        assert!(!config.tracing.enabled);
        assert!(config.health.detailed);
    }

    #[test]
    fn test_monitoring_config_structure() {
        let config = MonitoringConfig {
            metrics: MetricsConfig::default(),
            tracing: TracingConfig::default(),
            health: HealthConfig::default(),
        };
        assert_eq!(config.metrics.port, 9090);
    }

    #[test]
    fn test_monitoring_config_serialization() {
        let config = MonitoringConfig::default();
        let json = serde_json::to_value(&config).unwrap();
        assert!(json["metrics"].is_object());
        assert!(json["tracing"].is_object());
        assert!(json["health"].is_object());
    }

    #[test]
    fn test_monitoring_config_merge() {
        let base = MonitoringConfig::default();
        let other = MonitoringConfig {
            metrics: MetricsConfig {
                enabled: false,
                port: 9090,
                path: "/metrics".to_string(),
            },
            tracing: TracingConfig::default(),
            health: HealthConfig::default(),
        };
        let merged = base.merge(other);
        assert!(!merged.metrics.enabled);
    }

    #[test]
    fn test_monitoring_config_clone() {
        let config = MonitoringConfig::default();
        let cloned = config.clone();
        assert_eq!(config.metrics.enabled, cloned.metrics.enabled);
    }
}
