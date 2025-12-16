//! GDPR compliance tools
//!
//! Tools for managing data retention, consent, and export in compliance with GDPR.

use std::collections::HashMap;

use super::types::*;

/// GDPR compliance tools
pub struct GDPRCompliance {
    /// Data retention policies
    retention_policies: HashMap<String, RetentionPolicy>,
    /// Consent management
    consent_manager: ConsentManager,
    /// Data export tools
    export_tools: DataExportTools,
}

impl GDPRCompliance {
    /// Create a new GDPR compliance instance
    pub fn new() -> Self {
        Self {
            retention_policies: HashMap::new(),
            consent_manager: ConsentManager::new(),
            export_tools: DataExportTools::new(),
        }
    }

    /// Add a retention policy
    pub fn add_retention_policy(&mut self, data_type: String, policy: RetentionPolicy) {
        self.retention_policies.insert(data_type, policy);
    }

    /// Get retention policy for data type
    pub fn get_retention_policy(&self, data_type: &str) -> Option<&RetentionPolicy> {
        self.retention_policies.get(data_type)
    }

    /// Get consent manager
    pub fn consent_manager(&self) -> &ConsentManager {
        &self.consent_manager
    }

    /// Get mutable consent manager
    pub fn consent_manager_mut(&mut self) -> &mut ConsentManager {
        &mut self.consent_manager
    }

    /// Get export tools
    pub fn export_tools(&self) -> &DataExportTools {
        &self.export_tools
    }
}

impl Default for GDPRCompliance {
    fn default() -> Self {
        Self::new()
    }
}

impl ConsentManager {
    /// Create a new consent manager
    pub fn new() -> Self {
        Self {
            consents: HashMap::new(),
        }
    }

    /// Add user consent
    pub fn add_consent(&mut self, user_id: String, consent: UserConsent) {
        self.consents.insert(user_id, consent);
    }

    /// Get user consent
    pub fn get_consent(&self, user_id: &str) -> Option<&UserConsent> {
        self.consents.get(user_id)
    }

    /// Check if user has consented
    pub fn has_consent(&self, user_id: &str) -> bool {
        self.consents
            .get(user_id)
            .map(|c| c.consented)
            .unwrap_or(false)
    }

    /// Revoke user consent
    pub fn revoke_consent(&mut self, user_id: &str) {
        if let Some(consent) = self.consents.get_mut(user_id) {
            consent.consented = false;
            consent.timestamp = chrono::Utc::now();
        }
    }
}

impl Default for ConsentManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DataExportTools {
    /// Create a new data export tools instance
    pub fn new() -> Self {
        Self {
            formats: vec![
                ExportFormat::Json,
                ExportFormat::Csv,
                ExportFormat::Xml,
                ExportFormat::Pdf,
            ],
        }
    }

    /// Get supported formats
    pub fn supported_formats(&self) -> &[ExportFormat] {
        &self.formats
    }

    /// Check if format is supported
    pub fn is_format_supported(&self, format: &ExportFormat) -> bool {
        self.formats.iter().any(|f| match (f, format) {
            (ExportFormat::Json, ExportFormat::Json) => true,
            (ExportFormat::Csv, ExportFormat::Csv) => true,
            (ExportFormat::Xml, ExportFormat::Xml) => true,
            (ExportFormat::Pdf, ExportFormat::Pdf) => true,
            _ => false,
        })
    }
}

impl Default for DataExportTools {
    fn default() -> Self {
        Self::new()
    }
}
