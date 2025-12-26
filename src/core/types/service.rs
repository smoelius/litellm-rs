//! Service status types

use super::health::HealthStatus;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// API version
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApiVersion {
    /// v1 version
    #[default]
    V1,
    /// v2 version (future extension)
    V2,
}

impl std::fmt::Display for ApiVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::V1 => write!(f, "v1"),
            Self::V2 => write!(f, "v2"),
        }
    }
}

/// Service status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStatus {
    /// Service name
    pub name: String,
    /// Version
    pub version: String,
    /// Status
    pub status: HealthStatus,
    /// Start time
    pub uptime: SystemTime,
    /// Active connections
    pub active_connections: u32,
    /// Requests processed
    pub requests_processed: u64,
    /// Errors count
    pub errors: u64,
    /// Average response time (ms)
    pub avg_response_time_ms: f64,
    /// Memory usage (bytes)
    pub memory_usage_bytes: u64,
    /// CPU usage rate (percentage)
    pub cpu_usage_percent: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== ApiVersion Tests ====================

    #[test]
    fn test_api_version_default() {
        let version = ApiVersion::default();
        assert_eq!(version, ApiVersion::V1);
    }

    #[test]
    fn test_api_version_v1_display() {
        let version = ApiVersion::V1;
        assert_eq!(format!("{}", version), "v1");
    }

    #[test]
    fn test_api_version_v2_display() {
        let version = ApiVersion::V2;
        assert_eq!(format!("{}", version), "v2");
    }

    #[test]
    fn test_api_version_equality() {
        assert_eq!(ApiVersion::V1, ApiVersion::V1);
        assert_eq!(ApiVersion::V2, ApiVersion::V2);
        assert_ne!(ApiVersion::V1, ApiVersion::V2);
    }

    #[test]
    fn test_api_version_serialization() {
        let v1 = ApiVersion::V1;
        let v2 = ApiVersion::V2;

        let json_v1 = serde_json::to_string(&v1).unwrap();
        let json_v2 = serde_json::to_string(&v2).unwrap();

        assert!(json_v1.contains("V1"));
        assert!(json_v2.contains("V2"));
    }

    #[test]
    fn test_api_version_deserialization() {
        let v1: ApiVersion = serde_json::from_str("\"V1\"").unwrap();
        let v2: ApiVersion = serde_json::from_str("\"V2\"").unwrap();

        assert_eq!(v1, ApiVersion::V1);
        assert_eq!(v2, ApiVersion::V2);
    }

    #[test]
    fn test_api_version_clone() {
        let version = ApiVersion::V1;
        let cloned = version.clone();
        assert_eq!(version, cloned);
    }

    // ==================== ServiceStatus Tests ====================

    #[test]
    fn test_service_status_structure() {
        let status = ServiceStatus {
            name: "gateway".to_string(),
            version: "1.0.0".to_string(),
            status: HealthStatus::Healthy,
            uptime: SystemTime::now(),
            active_connections: 100,
            requests_processed: 10000,
            errors: 5,
            avg_response_time_ms: 50.5,
            memory_usage_bytes: 1024 * 1024 * 100, // 100MB
            cpu_usage_percent: 25.5,
        };

        assert_eq!(status.name, "gateway");
        assert_eq!(status.version, "1.0.0");
        assert_eq!(status.active_connections, 100);
        assert_eq!(status.requests_processed, 10000);
    }

    #[test]
    fn test_service_status_healthy() {
        let status = ServiceStatus {
            name: "api".to_string(),
            version: "2.0.0".to_string(),
            status: HealthStatus::Healthy,
            uptime: SystemTime::now(),
            active_connections: 50,
            requests_processed: 5000,
            errors: 0,
            avg_response_time_ms: 10.0,
            memory_usage_bytes: 50 * 1024 * 1024,
            cpu_usage_percent: 5.0,
        };

        assert!(matches!(status.status, HealthStatus::Healthy));
        assert_eq!(status.errors, 0);
    }

    #[test]
    fn test_service_status_unhealthy() {
        let status = ServiceStatus {
            name: "worker".to_string(),
            version: "1.5.0".to_string(),
            status: HealthStatus::Unhealthy,
            uptime: SystemTime::now(),
            active_connections: 0,
            requests_processed: 100,
            errors: 50,
            avg_response_time_ms: 5000.0,
            memory_usage_bytes: 1024 * 1024 * 500,
            cpu_usage_percent: 95.0,
        };

        assert!(matches!(status.status, HealthStatus::Unhealthy));
    }

    #[test]
    fn test_service_status_high_load() {
        let status = ServiceStatus {
            name: "load-test".to_string(),
            version: "1.0.0".to_string(),
            status: HealthStatus::Healthy,
            uptime: SystemTime::now(),
            active_connections: 10000,
            requests_processed: 1000000,
            errors: 100,
            avg_response_time_ms: 200.0,
            memory_usage_bytes: 1024 * 1024 * 1024 * 2, // 2GB
            cpu_usage_percent: 80.0,
        };

        assert_eq!(status.active_connections, 10000);
        assert_eq!(status.requests_processed, 1000000);
    }

    #[test]
    fn test_service_status_clone() {
        let status = ServiceStatus {
            name: "clone-test".to_string(),
            version: "1.0.0".to_string(),
            status: HealthStatus::Healthy,
            uptime: SystemTime::now(),
            active_connections: 10,
            requests_processed: 100,
            errors: 1,
            avg_response_time_ms: 50.0,
            memory_usage_bytes: 1024,
            cpu_usage_percent: 10.0,
        };

        let cloned = status.clone();
        assert_eq!(status.name, cloned.name);
        assert_eq!(status.version, cloned.version);
        assert_eq!(status.active_connections, cloned.active_connections);
    }

    #[test]
    fn test_service_status_serialization() {
        let status = ServiceStatus {
            name: "ser-test".to_string(),
            version: "1.0.0".to_string(),
            status: HealthStatus::Healthy,
            uptime: SystemTime::now(),
            active_connections: 5,
            requests_processed: 50,
            errors: 0,
            avg_response_time_ms: 25.0,
            memory_usage_bytes: 2048,
            cpu_usage_percent: 5.0,
        };

        let json = serde_json::to_value(&status).unwrap();
        assert_eq!(json["name"], "ser-test");
        assert_eq!(json["version"], "1.0.0");
        assert_eq!(json["active_connections"], 5);
    }

    #[test]
    fn test_service_status_zero_values() {
        let status = ServiceStatus {
            name: "new-service".to_string(),
            version: "0.0.1".to_string(),
            status: HealthStatus::Unknown,
            uptime: SystemTime::now(),
            active_connections: 0,
            requests_processed: 0,
            errors: 0,
            avg_response_time_ms: 0.0,
            memory_usage_bytes: 0,
            cpu_usage_percent: 0.0,
        };

        assert_eq!(status.active_connections, 0);
        assert_eq!(status.requests_processed, 0);
        assert!((status.avg_response_time_ms - 0.0).abs() < f64::EPSILON);
    }
}
