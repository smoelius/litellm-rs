//! Alert management system
//!
//! This module provides comprehensive alerting functionality for monitoring events.

use crate::config::AlertingConfig;
use crate::monitoring::{Alert, AlertSeverity};
use crate::utils::error::{GatewayError, Result};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, error, info, warn};

/// Alert manager for handling and dispatching alerts
#[derive(Debug)]
pub struct AlertManager {
    /// Configuration
    config: AlertingConfig,
    /// Pending alerts queue
    pending_alerts: Arc<Mutex<VecDeque<Alert>>>,
    /// Alert history
    alert_history: Arc<RwLock<VecDeque<Alert>>>,
    /// Alert rules
    alert_rules: Arc<RwLock<HashMap<String, AlertRule>>>,
    /// Notification channels
    notification_channels: Arc<RwLock<Vec<Box<dyn NotificationChannel>>>>,
    /// Whether the alert manager is active
    active: Arc<RwLock<bool>>,
    /// Alert statistics
    stats: Arc<RwLock<AlertStats>>,
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

/// Notification channel trait
#[async_trait::async_trait]
#[allow(dead_code)]
pub trait NotificationChannel: Send + Sync + std::fmt::Debug {
    /// Send a notification
    async fn send(&self, alert: &Alert) -> Result<()>;

    /// Get channel name
    fn name(&self) -> &str;

    /// Check if channel supports severity level
    fn supports_severity(&self, severity: AlertSeverity) -> bool;
}

/// Slack notification channel
#[derive(Debug)]
pub struct SlackChannel {
    webhook_url: String,
    channel: Option<String>,
    username: Option<String>,
    min_severity: AlertSeverity,
}

/// Email notification channel
#[derive(Debug)]
#[allow(dead_code)]
pub struct EmailChannel {
    smtp_config: SmtpConfig,
    recipients: Vec<String>,
    min_severity: AlertSeverity,
}

/// SMTP configuration
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SmtpConfig {
    pub server: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from_address: String,
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

#[allow(dead_code)]
impl AlertManager {
    /// Create a new alert manager
    pub async fn new(config: &AlertingConfig) -> Result<Self> {
        let mut notification_channels: Vec<Box<dyn NotificationChannel>> = Vec::new();

        // Add Slack channel if configured
        if let Some(webhook_url) = &config.slack_webhook {
            notification_channels.push(Box::new(SlackChannel::new(
                webhook_url.clone(),
                None,
                Some("Gateway Alert".to_string()),
                AlertSeverity::Info,
            )));
        }

        // Add email channel if configured
        // TODO: Add email configuration support

        Ok(Self {
            config: config.clone(),
            pending_alerts: Arc::new(Mutex::new(VecDeque::new())),
            alert_history: Arc::new(RwLock::new(VecDeque::new())),
            alert_rules: Arc::new(RwLock::new(HashMap::new())),
            notification_channels: Arc::new(RwLock::new(notification_channels)),
            active: Arc::new(RwLock::new(false)),
            stats: Arc::new(RwLock::new(AlertStats::default())),
        })
    }

    /// Start the alert manager
    pub async fn start(&self) -> Result<()> {
        info!("Starting alert manager");

        *self.active.write().await = true;

        // Start alert processing task
        self.start_alert_processing().await;

        // Start rule evaluation task
        self.start_rule_evaluation().await;

        Ok(())
    }

    /// Stop the alert manager
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping alert manager");
        *self.active.write().await = false;
        Ok(())
    }

    /// Send an alert
    pub async fn send_alert(&self, alert: Alert) -> Result<()> {
        debug!("Queuing alert: {} - {}", alert.severity, alert.title);

        // Add to pending queue
        {
            let mut pending = self.pending_alerts.lock().await;
            pending.push_back(alert.clone());
        }

        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.total_alerts += 1;
            *stats
                .alerts_by_severity
                .entry(format!("{:?}", alert.severity))
                .or_insert(0) += 1;
            *stats
                .alerts_by_source
                .entry(alert.source.clone())
                .or_insert(0) += 1;
            stats.last_alert = Some(alert.timestamp);
        }

        // Add to history
        {
            let mut history = self.alert_history.write().await;
            history.push_back(alert);

            // Keep only recent alerts (last 1000)
            if history.len() > 1000 {
                history.pop_front();
            }
        }

        Ok(())
    }

    /// Process pending alerts
    pub async fn process_pending(&self) -> Result<()> {
        let mut alerts_to_process = Vec::new();

        // Get pending alerts
        {
            let mut pending = self.pending_alerts.lock().await;
            while let Some(alert) = pending.pop_front() {
                alerts_to_process.push(alert);
            }
        }

        // Process each alert
        for alert in alerts_to_process {
            if let Err(e) = self.process_alert(&alert).await {
                error!("Failed to process alert {}: {}", alert.id, e);

                // Update failed notification count
                let mut stats = self.stats.write().await;
                stats.failed_notifications += 1;
            }
        }

        Ok(())
    }

    /// Process a single alert
    async fn process_alert(&self, alert: &Alert) -> Result<()> {
        debug!("Processing alert: {}", alert.id);

        let channels = self.notification_channels.read().await;

        for channel in channels.iter() {
            if channel.supports_severity(alert.severity) {
                if let Err(e) = channel.send(alert).await {
                    warn!("Failed to send alert via {}: {}", channel.name(), e);
                } else {
                    debug!("Alert sent via {}", channel.name());
                }
            }
        }

        Ok(())
    }

    /// Add an alert rule
    pub async fn add_rule(&self, rule: AlertRule) -> Result<()> {
        info!("Adding alert rule: {}", rule.name);

        let mut rules = self.alert_rules.write().await;
        rules.insert(rule.id.clone(), rule);

        Ok(())
    }

    /// Remove an alert rule
    pub async fn remove_rule(&self, rule_id: &str) -> Result<()> {
        info!("Removing alert rule: {}", rule_id);

        let mut rules = self.alert_rules.write().await;
        rules.remove(rule_id);

        Ok(())
    }

    /// Get alert statistics
    pub async fn get_stats(&self) -> AlertStats {
        self.stats.read().await.clone()
    }

    /// Get alert history
    pub async fn get_history(&self, limit: Option<usize>) -> Vec<Alert> {
        let history = self.alert_history.read().await;
        let limit = limit.unwrap_or(100);

        history.iter().rev().take(limit).cloned().collect()
    }

    /// Start alert processing task
    async fn start_alert_processing(&self) {
        let alert_manager = self.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));

            loop {
                interval.tick().await;

                if !*alert_manager.active.read().await {
                    break;
                }

                if let Err(e) = alert_manager.process_pending().await {
                    error!("Failed to process pending alerts: {}", e);
                }
            }
        });
    }

    /// Start rule evaluation task
    async fn start_rule_evaluation(&self) {
        let alert_manager = self.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));

            loop {
                interval.tick().await;

                if !*alert_manager.active.read().await {
                    break;
                }

                if let Err(e) = alert_manager.evaluate_rules().await {
                    error!("Failed to evaluate alert rules: {}", e);
                }
            }
        });
    }

    /// Evaluate alert rules
    async fn evaluate_rules(&self) -> Result<()> {
        debug!("Evaluating alert rules");

        let rules = self.alert_rules.read().await.clone();

        for rule in rules.values() {
            if rule.enabled {
                if let Err(e) = self.evaluate_rule(rule).await {
                    warn!("Failed to evaluate rule {}: {}", rule.name, e);
                }
            }
        }

        Ok(())
    }

    /// Evaluate a single alert rule
    async fn evaluate_rule(&self, rule: &AlertRule) -> Result<()> {
        // TODO: Implement metric evaluation
        // This would involve getting the current metric value and comparing it to the threshold

        debug!("Evaluating rule: {}", rule.name);

        // Placeholder implementation
        let metric_value = 0.0; // Get actual metric value
        let threshold_exceeded = match rule.operator {
            ComparisonOperator::GreaterThan => metric_value > rule.threshold,
            ComparisonOperator::LessThan => metric_value < rule.threshold,
            ComparisonOperator::GreaterThanOrEqual => metric_value >= rule.threshold,
            ComparisonOperator::LessThanOrEqual => metric_value <= rule.threshold,
            ComparisonOperator::Equal => (metric_value - rule.threshold).abs() < f64::EPSILON,
            ComparisonOperator::NotEqual => (metric_value - rule.threshold).abs() >= f64::EPSILON,
        };

        if threshold_exceeded {
            let alert = Alert {
                id: uuid::Uuid::new_v4().to_string(),
                severity: rule.severity,
                title: format!("Alert Rule Triggered: {}", rule.name),
                description: format!(
                    "Rule '{}' triggered: {} {} {} (current value: {})",
                    rule.name,
                    rule.metric,
                    format!("{:?}", rule.operator).to_lowercase(),
                    rule.threshold,
                    metric_value
                ),
                timestamp: chrono::Utc::now(),
                source: "alert_rule".to_string(),
                metadata: serde_json::json!({
                    "rule_id": rule.id,
                    "metric": rule.metric,
                    "threshold": rule.threshold,
                    "current_value": metric_value,
                    "operator": format!("{:?}", rule.operator)
                }),
                resolved: false,
            };

            self.send_alert(alert).await?;
        }

        Ok(())
    }
}

impl Clone for AlertManager {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            pending_alerts: self.pending_alerts.clone(),
            alert_history: self.alert_history.clone(),
            alert_rules: self.alert_rules.clone(),
            notification_channels: self.notification_channels.clone(),
            active: self.active.clone(),
            stats: self.stats.clone(),
        }
    }
}

#[allow(dead_code)]
impl SlackChannel {
    /// Create a new Slack notification channel
    pub fn new(
        webhook_url: String,
        channel: Option<String>,
        username: Option<String>,
        min_severity: AlertSeverity,
    ) -> Self {
        Self {
            webhook_url,
            channel,
            username,
            min_severity,
        }
    }
}

#[async_trait::async_trait]
impl NotificationChannel for SlackChannel {
    async fn send(&self, alert: &Alert) -> Result<()> {
        let color = match alert.severity {
            AlertSeverity::Info => "#36a64f",      // Green
            AlertSeverity::Warning => "#ff9500",   // Orange
            AlertSeverity::Critical => "#ff0000",  // Red
            AlertSeverity::Emergency => "#8b0000", // Dark Red
        };

        let payload = serde_json::json!({
            "username": self.username.as_deref().unwrap_or("Gateway Alert"),
            "channel": self.channel,
            "attachments": [{
                "color": color,
                "title": alert.title,
                "text": alert.description,
                "fields": [
                    {
                        "title": "Severity",
                        "value": format!("{:?}", alert.severity),
                        "short": true
                    },
                    {
                        "title": "Source",
                        "value": alert.source,
                        "short": true
                    },
                    {
                        "title": "Time",
                        "value": alert.timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                        "short": true
                    }
                ],
                "footer": "Gateway Monitoring",
                "ts": alert.timestamp.timestamp()
            }]
        });

        let client = reqwest::Client::new();
        let response = client
            .post(&self.webhook_url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| {
                GatewayError::Alert(format!("Failed to send Slack notification: {}", e))
            })?;

        if !response.status().is_success() {
            return Err(GatewayError::Alert(format!(
                "Slack webhook returned status: {}",
                response.status()
            )));
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "slack"
    }

    fn supports_severity(&self, severity: AlertSeverity) -> bool {
        severity as u8 >= self.min_severity as u8
    }
}

#[allow(dead_code)]
impl EmailChannel {
    /// Create a new email notification channel
    pub fn new(
        smtp_config: SmtpConfig,
        recipients: Vec<String>,
        min_severity: AlertSeverity,
    ) -> Self {
        Self {
            smtp_config,
            recipients,
            min_severity,
        }
    }
}

#[async_trait::async_trait]
impl NotificationChannel for EmailChannel {
    async fn send(&self, _alert: &Alert) -> Result<()> {
        // TODO: Implement email sending
        // This would use an SMTP library to send emails
        warn!("Email notifications not implemented yet");
        Ok(())
    }

    fn name(&self) -> &str {
        "email"
    }

    fn supports_severity(&self, severity: AlertSeverity) -> bool {
        severity as u8 >= self.min_severity as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alert_rule_creation() {
        let rule = AlertRule {
            id: "test-rule".to_string(),
            name: "High CPU Usage".to_string(),
            description: "Alert when CPU usage exceeds 80%".to_string(),
            metric: "cpu_usage".to_string(),
            threshold: 80.0,
            operator: ComparisonOperator::GreaterThan,
            severity: AlertSeverity::Warning,
            interval: Duration::from_secs(60),
            enabled: true,
            channels: vec!["slack".to_string()],
        };

        assert_eq!(rule.name, "High CPU Usage");
        assert_eq!(rule.threshold, 80.0);
        assert!(rule.enabled);
    }

    #[test]
    fn test_comparison_operators() {
        assert_eq!(
            ComparisonOperator::GreaterThan,
            ComparisonOperator::GreaterThan
        );
        assert_ne!(
            ComparisonOperator::GreaterThan,
            ComparisonOperator::LessThan
        );
    }

    #[test]
    fn test_slack_channel_creation() {
        let channel = SlackChannel::new(
            "https://hooks.slack.com/test".to_string(),
            Some("#alerts".to_string()),
            Some("Gateway".to_string()),
            AlertSeverity::Warning,
        );

        assert_eq!(channel.name(), "slack");
        assert!(channel.supports_severity(AlertSeverity::Critical));
        assert!(!channel.supports_severity(AlertSeverity::Info));
    }

    #[test]
    fn test_alert_stats() {
        let mut stats = AlertStats {
            total_alerts: 10,
            ..Default::default()
        };
        stats.alerts_by_severity.insert("Warning".to_string(), 5);
        stats.alerts_by_severity.insert("Critical".to_string(), 3);

        assert_eq!(stats.total_alerts, 10);
        assert_eq!(stats.alerts_by_severity.get("Warning"), Some(&5));
    }
}
