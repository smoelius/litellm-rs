//! Alert types and data structures

use crate::monitoring::AlertSeverity;
use std::collections::{HashMap, VecDeque};
use std::time::Duration;

/// Consolidated alert storage - single lock for related data
#[derive(Debug, Default)]
pub(super) struct AlertStorage {
    /// Alert history
    pub history: VecDeque<crate::monitoring::Alert>,
    /// Alert rules
    pub rules: HashMap<String, AlertRule>,
    /// Alert statistics
    pub stats: AlertStats,
}

/// Alert rule for automated alerting
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AlertRule {
    /// Rule ID
    pub id: String,
    /// Rule name
    pub name: String,
    /// Rule description
    pub description: String,
    /// Metric to monitor
    pub metric: String,
    /// Threshold value
    pub threshold: f64,
    /// Comparison operator
    pub operator: ComparisonOperator,
    /// Alert severity
    pub severity: AlertSeverity,
    /// Evaluation interval
    pub interval: Duration,
    /// Whether the rule is enabled
    pub enabled: bool,
    /// Notification channels for this rule
    pub channels: Vec<String>,
}

/// Comparison operators for alert rules
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ComparisonOperator {
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    Equal,
    NotEqual,
}

/// Alert statistics
#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct AlertStats {
    /// Total alerts sent
    pub total_alerts: u64,
    /// Alerts by severity
    pub alerts_by_severity: HashMap<String, u64>,
    /// Alerts by source
    pub alerts_by_source: HashMap<String, u64>,
    /// Failed notifications
    pub failed_notifications: u64,
    /// Last alert timestamp
    pub last_alert: Option<chrono::DateTime<chrono::Utc>>,
}
