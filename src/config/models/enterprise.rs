//! Enterprise configuration

use serde::{Deserialize, Serialize};

/// Enterprise configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EnterpriseConfig {
    /// Enable enterprise features
    #[serde(default)]
    pub enabled: bool,
    /// SSO configuration
    pub sso: Option<SsoConfig>,
    /// Enable audit logging
    #[serde(default)]
    pub audit_logging: bool,
    /// Enable advanced analytics
    #[serde(default)]
    pub advanced_analytics: bool,
}

#[allow(dead_code)]
impl EnterpriseConfig {
    /// Merge enterprise configurations
    pub fn merge(mut self, other: Self) -> Self {
        if other.enabled {
            self.enabled = other.enabled;
        }
        if other.sso.is_some() {
            self.sso = other.sso;
        }
        if other.audit_logging {
            self.audit_logging = other.audit_logging;
        }
        if other.advanced_analytics {
            self.advanced_analytics = other.advanced_analytics;
        }
        self
    }
}

/// SSO configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsoConfig {
    /// SSO provider
    pub provider: String,
    /// Client ID
    pub client_id: String,
    /// Client secret
    pub client_secret: String,
    /// Redirect URL
    pub redirect_url: String,
    /// Additional settings
    #[serde(default)]
    pub settings: std::collections::HashMap<String, serde_json::Value>,
}
