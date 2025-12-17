//! Alert manager implementation

use super::channels::{NotificationChannel, SlackChannel};
use super::types::{AlertRule, AlertStats, AlertStorage};
use crate::config::AlertingConfig;
use crate::monitoring::types::{Alert, AlertSeverity};
use crate::utils::error::Result;
use parking_lot::{Mutex, RwLock};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock as TokioRwLock;
use tracing::{debug, info};

/// Alert manager for handling and dispatching alerts
#[derive(Debug)]
pub struct AlertManager {
    /// Configuration
    config: AlertingConfig,
    /// Consolidated storage for all alert-related data
    pub(super) storage: Arc<RwLock<AlertStorage>>,
    /// Pending alerts queue (separate for fast lock-free access)
    pub(super) pending_alerts: Arc<Mutex<VecDeque<Alert>>>,
    /// Notification channels - using tokio RwLock because we need to hold across await points
    pub(super) notification_channels: Arc<TokioRwLock<Vec<Box<dyn NotificationChannel>>>>,
    /// Whether the alert manager is active - using AtomicBool for lock-free access
    pub(super) active: AtomicBool,
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
            storage: Arc::new(RwLock::new(AlertStorage::default())),
            pending_alerts: Arc::new(Mutex::new(VecDeque::new())),
            notification_channels: Arc::new(TokioRwLock::new(notification_channels)),
            active: AtomicBool::new(false),
        })
    }

    /// Start the alert manager
    pub async fn start(&self) -> Result<()> {
        info!("Starting alert manager");

        self.active.store(true, Ordering::Release);

        // Start alert processing task
        self.start_alert_processing().await;

        // Start rule evaluation task
        self.start_rule_evaluation().await;

        Ok(())
    }

    /// Stop the alert manager
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping alert manager");
        self.active.store(false, Ordering::Release);
        Ok(())
    }

    /// Check if alert manager is active
    #[inline]
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Acquire)
    }

    /// Send an alert
    pub async fn send_alert(&self, alert: Alert) -> Result<()> {
        debug!("Queuing alert: {} - {}", alert.severity, alert.title);

        // Add to pending queue
        {
            self.pending_alerts.lock().push_back(alert.clone());
        }

        // Update statistics and history in a single lock
        {
            let mut storage = self.storage.write();

            // Update statistics
            storage.stats.total_alerts += 1;
            *storage
                .stats
                .alerts_by_severity
                .entry(format!("{:?}", alert.severity))
                .or_insert(0) += 1;
            *storage
                .stats
                .alerts_by_source
                .entry(alert.source.clone())
                .or_insert(0) += 1;
            storage.stats.last_alert = Some(alert.timestamp);

            // Add to history
            storage.history.push_back(alert);

            // Keep only recent alerts (last 1000)
            if storage.history.len() > 1000 {
                storage.history.pop_front();
            }
        }

        Ok(())
    }

    /// Process pending alerts
    pub async fn process_pending(&self) -> Result<()> {
        let mut alerts_to_process = Vec::new();

        // Get pending alerts - using parking_lot Mutex (no await needed)
        {
            let mut pending = self.pending_alerts.lock();
            while let Some(alert) = pending.pop_front() {
                alerts_to_process.push(alert);
            }
        }

        // Process each alert
        for alert in alerts_to_process {
            if let Err(e) = self.process_alert(&alert).await {
                tracing::error!("Failed to process alert {}: {}", alert.id, e);

                // Update failed notification count
                self.storage.write().stats.failed_notifications += 1;
            }
        }

        Ok(())
    }

    /// Add an alert rule
    pub async fn add_rule(&self, rule: AlertRule) -> Result<()> {
        info!("Adding alert rule: {}", rule.name);

        self.storage.write().rules.insert(rule.id.clone(), rule);

        Ok(())
    }

    /// Remove an alert rule
    pub async fn remove_rule(&self, rule_id: &str) -> Result<()> {
        info!("Removing alert rule: {}", rule_id);

        self.storage.write().rules.remove(rule_id);

        Ok(())
    }

    /// Get alert statistics
    pub async fn get_stats(&self) -> AlertStats {
        self.storage.read().stats.clone()
    }

    /// Get alert history
    pub async fn get_history(&self, limit: Option<usize>) -> Vec<Alert> {
        let storage = self.storage.read();
        let limit = limit.unwrap_or(100);

        storage.history.iter().rev().take(limit).cloned().collect()
    }
}

impl Clone for AlertManager {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            storage: self.storage.clone(),
            pending_alerts: self.pending_alerts.clone(),
            notification_channels: self.notification_channels.clone(),
            active: AtomicBool::new(self.active.load(Ordering::Acquire)),
        }
    }
}
