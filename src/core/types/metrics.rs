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
