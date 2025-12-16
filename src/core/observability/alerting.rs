//! Alert management system

use super::destinations::AlertChannel;
use super::destinations::AlertRule;
use super::types::AlertState;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Alert manager for notifications
pub struct AlertManager {
    /// Alert channels
    channels: Vec<AlertChannel>,
    /// Alert rules
    rules: Vec<AlertRule>,
    /// Alert state tracking
    alert_states: Arc<RwLock<HashMap<String, AlertState>>>,
}

impl Default for AlertManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AlertManager {
    /// Create a new alert manager
    pub fn new() -> Self {
        Self {
            channels: Vec::new(),
            rules: Vec::new(),
            alert_states: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add an alert channel
    pub fn add_channel(&mut self, channel: AlertChannel) {
        self.channels.push(channel);
    }

    /// Add an alert rule
    pub fn add_rule(&mut self, rule: AlertRule) {
        self.rules.push(rule);
    }

    /// Get all rules
    pub fn rules(&self) -> &[AlertRule] {
        &self.rules
    }

    /// Get all channels
    pub fn channels(&self) -> &[AlertChannel] {
        &self.channels
    }

    /// Get alert state for a rule
    pub async fn get_alert_state(&self, rule_id: &str) -> Option<AlertState> {
        let states = self.alert_states.read().await;
        states.get(rule_id).cloned()
    }

    /// Update alert state
    pub async fn update_alert_state(&self, rule_id: String, state: AlertState) {
        let mut states = self.alert_states.write().await;
        states.insert(rule_id, state);
    }
}
