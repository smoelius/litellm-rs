//! Health check types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

/// Health status
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    /// Healthy
    Healthy,
    /// Unhealthy
    Unhealthy,
    /// Unknown status
    #[default]
    Unknown,
    /// Degraded service
    Degraded,
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    /// Status
    pub status: HealthStatus,
    /// Check time
    pub checked_at: SystemTime,
    /// Latency (milliseconds)
    pub latency_ms: Option<u64>,
    /// Error message
    pub error: Option<String>,
    /// Extra details
    pub details: HashMap<String, serde_json::Value>,
}

impl Default for HealthCheckResult {
    fn default() -> Self {
        Self {
            status: HealthStatus::Unknown,
            checked_at: SystemTime::now(),
            latency_ms: None,
            error: None,
            details: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== HealthStatus Tests ====================

    #[test]
    fn test_health_status_default() {
        let status = HealthStatus::default();
        assert_eq!(status, HealthStatus::Unknown);
    }

    #[test]
    fn test_health_status_variants() {
        let healthy = HealthStatus::Healthy;
        let unhealthy = HealthStatus::Unhealthy;
        let unknown = HealthStatus::Unknown;
        let degraded = HealthStatus::Degraded;

        assert_eq!(healthy, HealthStatus::Healthy);
        assert_eq!(unhealthy, HealthStatus::Unhealthy);
        assert_eq!(unknown, HealthStatus::Unknown);
        assert_eq!(degraded, HealthStatus::Degraded);
    }

    #[test]
    fn test_health_status_serialization() {
        assert_eq!(
            serde_json::to_string(&HealthStatus::Healthy).unwrap(),
            "\"healthy\""
        );
        assert_eq!(
            serde_json::to_string(&HealthStatus::Unhealthy).unwrap(),
            "\"unhealthy\""
        );
        assert_eq!(
            serde_json::to_string(&HealthStatus::Unknown).unwrap(),
            "\"unknown\""
        );
        assert_eq!(
            serde_json::to_string(&HealthStatus::Degraded).unwrap(),
            "\"degraded\""
        );
    }

    #[test]
    fn test_health_status_deserialization() {
        let healthy: HealthStatus = serde_json::from_str("\"healthy\"").unwrap();
        assert_eq!(healthy, HealthStatus::Healthy);

        let unhealthy: HealthStatus = serde_json::from_str("\"unhealthy\"").unwrap();
        assert_eq!(unhealthy, HealthStatus::Unhealthy);
    }

    #[test]
    fn test_health_status_clone() {
        let status = HealthStatus::Healthy;
        let cloned = status.clone();
        assert_eq!(status, cloned);
    }

    #[test]
    fn test_health_status_equality() {
        assert_eq!(HealthStatus::Healthy, HealthStatus::Healthy);
        assert_ne!(HealthStatus::Healthy, HealthStatus::Unhealthy);
    }

    // ==================== HealthCheckResult Tests ====================

    #[test]
    fn test_health_check_result_default() {
        let result = HealthCheckResult::default();
        assert_eq!(result.status, HealthStatus::Unknown);
        assert!(result.latency_ms.is_none());
        assert!(result.error.is_none());
        assert!(result.details.is_empty());
    }

    #[test]
    fn test_health_check_result_healthy() {
        let result = HealthCheckResult {
            status: HealthStatus::Healthy,
            checked_at: SystemTime::now(),
            latency_ms: Some(50),
            error: None,
            details: HashMap::new(),
        };
        assert_eq!(result.status, HealthStatus::Healthy);
        assert_eq!(result.latency_ms, Some(50));
    }

    #[test]
    fn test_health_check_result_unhealthy() {
        let result = HealthCheckResult {
            status: HealthStatus::Unhealthy,
            checked_at: SystemTime::now(),
            latency_ms: None,
            error: Some("Connection refused".to_string()),
            details: HashMap::new(),
        };
        assert_eq!(result.status, HealthStatus::Unhealthy);
        assert_eq!(result.error, Some("Connection refused".to_string()));
    }

    #[test]
    fn test_health_check_result_with_details() {
        let mut details = HashMap::new();
        details.insert("database".to_string(), serde_json::json!("connected"));
        details.insert("cache".to_string(), serde_json::json!("available"));

        let result = HealthCheckResult {
            status: HealthStatus::Healthy,
            checked_at: SystemTime::now(),
            latency_ms: Some(25),
            error: None,
            details,
        };
        assert_eq!(result.details.len(), 2);
    }

    #[test]
    fn test_health_check_result_degraded() {
        let mut details = HashMap::new();
        details.insert("cache".to_string(), serde_json::json!("slow"));

        let result = HealthCheckResult {
            status: HealthStatus::Degraded,
            checked_at: SystemTime::now(),
            latency_ms: Some(500),
            error: None,
            details,
        };
        assert_eq!(result.status, HealthStatus::Degraded);
        assert!(result.latency_ms.unwrap() > 100);
    }

    #[test]
    fn test_health_check_result_clone() {
        let result = HealthCheckResult {
            status: HealthStatus::Healthy,
            checked_at: SystemTime::now(),
            latency_ms: Some(10),
            error: None,
            details: HashMap::new(),
        };
        let cloned = result.clone();
        assert_eq!(result.status, cloned.status);
        assert_eq!(result.latency_ms, cloned.latency_ms);
    }

    #[test]
    fn test_health_check_result_serialization() {
        let result = HealthCheckResult {
            status: HealthStatus::Healthy,
            checked_at: SystemTime::UNIX_EPOCH,
            latency_ms: Some(100),
            error: None,
            details: HashMap::new(),
        };
        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["status"], "healthy");
        assert_eq!(json["latency_ms"], 100);
    }
}
