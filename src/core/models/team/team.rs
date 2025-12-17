//! Team core model

use super::billing::TeamBilling;
use super::settings::TeamSettings;
use crate::core::models::{Metadata, UsageStats};
use crate::core::models::user::types::UserRateLimits;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Team/Organization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    /// Team metadata
    #[serde(flatten)]
    pub metadata: Metadata,
    /// Team name (unique)
    pub name: String,
    /// Team display name
    pub display_name: Option<String>,
    /// Team description
    pub description: Option<String>,
    /// Team status
    pub status: TeamStatus,
    /// Team settings
    pub settings: TeamSettings,
    /// Usage statistics
    pub usage_stats: UsageStats,
    /// Team rate limits
    pub rate_limits: Option<UserRateLimits>,
    /// Billing information
    pub billing: Option<TeamBilling>,
    /// Team metadata
    pub team_metadata: HashMap<String, serde_json::Value>,
}

/// Team status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TeamStatus {
    /// Active team
    Active,
    /// Inactive team
    Inactive,
    /// Suspended team
    Suspended,
    /// Deleted team (soft delete)
    Deleted,
}

/// Team visibility
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TeamVisibility {
    /// Public team
    Public,
    /// Private team
    #[default]
    Private,
    /// Internal team
    Internal,
}

impl Team {
    /// Create a new team
    pub fn new(name: String, display_name: Option<String>) -> Self {
        Self {
            metadata: Metadata::new(),
            name,
            display_name,
            description: None,
            status: TeamStatus::Active,
            settings: TeamSettings::default(),
            usage_stats: UsageStats::default(),
            rate_limits: None,
            billing: None,
            team_metadata: HashMap::new(),
        }
    }

    /// Get team ID
    pub fn id(&self) -> Uuid {
        self.metadata.id
    }

    /// Check if team is active
    pub fn is_active(&self) -> bool {
        matches!(self.status, TeamStatus::Active)
    }

    /// Update usage statistics
    pub fn update_usage(&mut self, requests: u64, tokens: u64, cost: f64) {
        self.usage_stats.total_requests += requests;
        self.usage_stats.total_tokens += tokens;
        self.usage_stats.total_cost += cost;

        // Update daily stats
        let today = chrono::Utc::now().date_naive();
        let last_reset = self.usage_stats.last_reset.date_naive();

        if today != last_reset {
            self.usage_stats.requests_today = 0;
            self.usage_stats.tokens_today = 0;
            self.usage_stats.cost_today = 0.0;
            self.usage_stats.last_reset = chrono::Utc::now();
        }

        self.usage_stats.requests_today += requests as u32;
        self.usage_stats.tokens_today += tokens as u32;
        self.usage_stats.cost_today += cost;

        // Update billing usage if applicable
        if let Some(billing) = &mut self.billing {
            billing.current_usage += cost;
        }

        self.metadata.touch();
    }

    /// Check if team is over budget
    pub fn is_over_budget(&self) -> bool {
        if let Some(billing) = &self.billing {
            if let Some(budget) = billing.monthly_budget {
                return billing.current_usage >= budget;
            }
        }
        false
    }

    /// Get remaining budget
    pub fn remaining_budget(&self) -> Option<f64> {
        if let Some(billing) = &self.billing {
            if let Some(budget) = billing.monthly_budget {
                return Some((budget - billing.current_usage).max(0.0));
            }
        }
        None
    }
}
