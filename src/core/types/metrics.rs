//! Metrics types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

/// Metric data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricDataPoint {
    /// Metric name
    pub name: String,
    /// Metric value
    pub value: f64,
    /// Labels
    pub labels: HashMap<String, String>,
    /// Timestamp
    pub timestamp: SystemTime,
}

/// Metric type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MetricType {
    /// Counter
    Counter,
    /// Gauge
    Gauge,
    /// Histogram
    Histogram,
    /// Summary
    Summary,
}

/// Metric definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricDefinition {
    /// Metric name
    pub name: String,
    /// Metric type
    pub metric_type: MetricType,
    /// Description
    pub description: String,
    /// Unit
    pub unit: Option<String>,
    /// Labels
    pub labels: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== MetricDataPoint Tests ====================

    #[test]
    fn test_metric_data_point_structure() {
        let mut labels = HashMap::new();
        labels.insert("provider".to_string(), "openai".to_string());

        let point = MetricDataPoint {
            name: "request_count".to_string(),
            value: 100.0,
            labels,
            timestamp: SystemTime::now(),
        };

        assert_eq!(point.name, "request_count");
        assert!((point.value - 100.0).abs() < f64::EPSILON);
        assert_eq!(point.labels.get("provider"), Some(&"openai".to_string()));
    }

    #[test]
    fn test_metric_data_point_empty_labels() {
        let point = MetricDataPoint {
            name: "gauge".to_string(),
            value: 42.5,
            labels: HashMap::new(),
            timestamp: SystemTime::now(),
        };

        assert!(point.labels.is_empty());
    }

    #[test]
    fn test_metric_data_point_multiple_labels() {
        let mut labels = HashMap::new();
        labels.insert("provider".to_string(), "anthropic".to_string());
        labels.insert("model".to_string(), "claude-3".to_string());
        labels.insert("region".to_string(), "us-east-1".to_string());

        let point = MetricDataPoint {
            name: "latency".to_string(),
            value: 150.5,
            labels,
            timestamp: SystemTime::now(),
        };

        assert_eq!(point.labels.len(), 3);
    }

    #[test]
    fn test_metric_data_point_negative_value() {
        let point = MetricDataPoint {
            name: "temperature".to_string(),
            value: -10.5,
            labels: HashMap::new(),
            timestamp: SystemTime::now(),
        };

        assert!(point.value < 0.0);
    }

    #[test]
    fn test_metric_data_point_clone() {
        let point = MetricDataPoint {
            name: "test".to_string(),
            value: 1.0,
            labels: HashMap::new(),
            timestamp: SystemTime::now(),
        };

        let cloned = point.clone();
        assert_eq!(point.name, cloned.name);
        assert!((point.value - cloned.value).abs() < f64::EPSILON);
    }

    // ==================== MetricType Tests ====================

    #[test]
    fn test_metric_type_counter_serialization() {
        let t = MetricType::Counter;
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, "\"counter\"");
    }

    #[test]
    fn test_metric_type_gauge_serialization() {
        let t = MetricType::Gauge;
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, "\"gauge\"");
    }

    #[test]
    fn test_metric_type_histogram_serialization() {
        let t = MetricType::Histogram;
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, "\"histogram\"");
    }

    #[test]
    fn test_metric_type_summary_serialization() {
        let t = MetricType::Summary;
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, "\"summary\"");
    }

    #[test]
    fn test_metric_type_deserialization() {
        let counter: MetricType = serde_json::from_str("\"counter\"").unwrap();
        let gauge: MetricType = serde_json::from_str("\"gauge\"").unwrap();
        let histogram: MetricType = serde_json::from_str("\"histogram\"").unwrap();
        let summary: MetricType = serde_json::from_str("\"summary\"").unwrap();

        assert_eq!(counter, MetricType::Counter);
        assert_eq!(gauge, MetricType::Gauge);
        assert_eq!(histogram, MetricType::Histogram);
        assert_eq!(summary, MetricType::Summary);
    }

    #[test]
    fn test_metric_type_equality() {
        assert_eq!(MetricType::Counter, MetricType::Counter);
        assert_ne!(MetricType::Counter, MetricType::Gauge);
    }

    #[test]
    fn test_metric_type_clone() {
        let t = MetricType::Histogram;
        let cloned = t.clone();
        assert_eq!(t, cloned);
    }

    // ==================== MetricDefinition Tests ====================

    #[test]
    fn test_metric_definition_structure() {
        let def = MetricDefinition {
            name: "http_requests_total".to_string(),
            metric_type: MetricType::Counter,
            description: "Total HTTP requests".to_string(),
            unit: Some("requests".to_string()),
            labels: vec!["method".to_string(), "status".to_string()],
        };

        assert_eq!(def.name, "http_requests_total");
        assert_eq!(def.metric_type, MetricType::Counter);
        assert_eq!(def.labels.len(), 2);
    }

    #[test]
    fn test_metric_definition_no_unit() {
        let def = MetricDefinition {
            name: "queue_size".to_string(),
            metric_type: MetricType::Gauge,
            description: "Current queue size".to_string(),
            unit: None,
            labels: vec![],
        };

        assert!(def.unit.is_none());
        assert!(def.labels.is_empty());
    }

    #[test]
    fn test_metric_definition_serialization() {
        let def = MetricDefinition {
            name: "latency".to_string(),
            metric_type: MetricType::Histogram,
            description: "Request latency".to_string(),
            unit: Some("ms".to_string()),
            labels: vec!["endpoint".to_string()],
        };

        let json = serde_json::to_value(&def).unwrap();
        assert_eq!(json["name"], "latency");
        assert_eq!(json["metric_type"], "histogram");
        assert_eq!(json["unit"], "ms");
    }

    #[test]
    fn test_metric_definition_deserialization() {
        let json = r#"{
            "name": "test_metric",
            "metric_type": "counter",
            "description": "Test description",
            "unit": null,
            "labels": ["label1"]
        }"#;

        let def: MetricDefinition = serde_json::from_str(json).unwrap();
        assert_eq!(def.name, "test_metric");
        assert_eq!(def.metric_type, MetricType::Counter);
        assert!(def.unit.is_none());
    }

    #[test]
    fn test_metric_definition_clone() {
        let def = MetricDefinition {
            name: "clone_test".to_string(),
            metric_type: MetricType::Summary,
            description: "Clone test".to_string(),
            unit: Some("seconds".to_string()),
            labels: vec!["quantile".to_string()],
        };

        let cloned = def.clone();
        assert_eq!(def.name, cloned.name);
        assert_eq!(def.metric_type, cloned.metric_type);
    }
}
