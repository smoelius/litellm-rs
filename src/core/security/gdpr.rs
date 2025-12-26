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
        self.formats.iter().any(|f| {
            matches!(
                (f, format),
                (ExportFormat::Json, ExportFormat::Json)
                    | (ExportFormat::Csv, ExportFormat::Csv)
                    | (ExportFormat::Xml, ExportFormat::Xml)
                    | (ExportFormat::Pdf, ExportFormat::Pdf)
            )
        })
    }
}

impl Default for DataExportTools {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    // ==================== GDPRCompliance Tests ====================

    #[test]
    fn test_gdpr_compliance_new() {
        let gdpr = GDPRCompliance::new();
        assert!(gdpr.retention_policies.is_empty());
    }

    #[test]
    fn test_gdpr_compliance_default() {
        let gdpr = GDPRCompliance::default();
        assert!(gdpr.retention_policies.is_empty());
    }

    #[test]
    fn test_gdpr_compliance_add_retention_policy() {
        let mut gdpr = GDPRCompliance::new();
        let policy = RetentionPolicy {
            data_type: "logs".to_string(),
            retention_days: 30,
            auto_delete: true,
            anonymization: None,
        };
        gdpr.add_retention_policy("logs".to_string(), policy);
        assert!(gdpr.get_retention_policy("logs").is_some());
    }

    #[test]
    fn test_gdpr_compliance_get_retention_policy_not_found() {
        let gdpr = GDPRCompliance::new();
        assert!(gdpr.get_retention_policy("nonexistent").is_none());
    }

    #[test]
    fn test_gdpr_compliance_multiple_policies() {
        let mut gdpr = GDPRCompliance::new();
        gdpr.add_retention_policy(
            "logs".to_string(),
            RetentionPolicy {
                data_type: "logs".to_string(),
                retention_days: 30,
                auto_delete: true,
                anonymization: None,
            },
        );
        gdpr.add_retention_policy(
            "metrics".to_string(),
            RetentionPolicy {
                data_type: "metrics".to_string(),
                retention_days: 90,
                auto_delete: false,
                anonymization: None,
            },
        );
        assert_eq!(gdpr.get_retention_policy("logs").unwrap().retention_days, 30);
        assert_eq!(gdpr.get_retention_policy("metrics").unwrap().retention_days, 90);
    }

    #[test]
    fn test_gdpr_compliance_consent_manager() {
        let gdpr = GDPRCompliance::new();
        let _ = gdpr.consent_manager();
    }

    #[test]
    fn test_gdpr_compliance_consent_manager_mut() {
        let mut gdpr = GDPRCompliance::new();
        let consent_mgr = gdpr.consent_manager_mut();
        consent_mgr.add_consent(
            "user1".to_string(),
            UserConsent {
                user_id: "user1".to_string(),
                consented: true,
                timestamp: Utc::now(),
                version: "1.0".to_string(),
                permissions: vec!["data_processing".to_string()],
            },
        );
        assert!(gdpr.consent_manager().has_consent("user1"));
    }

    #[test]
    fn test_gdpr_compliance_export_tools() {
        let gdpr = GDPRCompliance::new();
        let tools = gdpr.export_tools();
        assert!(!tools.supported_formats().is_empty());
    }

    // ==================== ConsentManager Tests ====================

    #[test]
    fn test_consent_manager_new() {
        let manager = ConsentManager::new();
        assert!(!manager.has_consent("unknown_user"));
    }

    #[test]
    fn test_consent_manager_default() {
        let manager = ConsentManager::default();
        assert!(!manager.has_consent("unknown_user"));
    }

    #[test]
    fn test_consent_manager_add_consent() {
        let mut manager = ConsentManager::new();
        let consent = UserConsent {
            user_id: "user123".to_string(),
            consented: true,
            timestamp: Utc::now(),
            version: "2.0".to_string(),
            permissions: vec!["analytics".to_string()],
        };
        manager.add_consent("user123".to_string(), consent);
        assert!(manager.has_consent("user123"));
    }

    #[test]
    fn test_consent_manager_get_consent() {
        let mut manager = ConsentManager::new();
        let consent = UserConsent {
            user_id: "user456".to_string(),
            consented: true,
            timestamp: Utc::now(),
            version: "1.5".to_string(),
            permissions: vec!["marketing".to_string()],
        };
        manager.add_consent("user456".to_string(), consent);
        let retrieved = manager.get_consent("user456").unwrap();
        assert_eq!(retrieved.version, "1.5");
    }

    #[test]
    fn test_consent_manager_get_consent_not_found() {
        let manager = ConsentManager::new();
        assert!(manager.get_consent("unknown").is_none());
    }

    #[test]
    fn test_consent_manager_has_consent_false() {
        let mut manager = ConsentManager::new();
        let consent = UserConsent {
            user_id: "user789".to_string(),
            consented: false,
            timestamp: Utc::now(),
            version: "1.0".to_string(),
            permissions: vec![],
        };
        manager.add_consent("user789".to_string(), consent);
        assert!(!manager.has_consent("user789"));
    }

    #[test]
    fn test_consent_manager_revoke_consent() {
        let mut manager = ConsentManager::new();
        let consent = UserConsent {
            user_id: "user_revoke".to_string(),
            consented: true,
            timestamp: Utc::now(),
            version: "1.0".to_string(),
            permissions: vec!["all".to_string()],
        };
        manager.add_consent("user_revoke".to_string(), consent);
        assert!(manager.has_consent("user_revoke"));
        manager.revoke_consent("user_revoke");
        assert!(!manager.has_consent("user_revoke"));
    }

    #[test]
    fn test_consent_manager_revoke_nonexistent() {
        let mut manager = ConsentManager::new();
        manager.revoke_consent("nonexistent");
        // Should not panic
    }

    // ==================== DataExportTools Tests ====================

    #[test]
    fn test_data_export_tools_new() {
        let tools = DataExportTools::new();
        assert_eq!(tools.supported_formats().len(), 4);
    }

    #[test]
    fn test_data_export_tools_default() {
        let tools = DataExportTools::default();
        assert_eq!(tools.supported_formats().len(), 4);
    }

    #[test]
    fn test_data_export_tools_json_supported() {
        let tools = DataExportTools::new();
        assert!(tools.is_format_supported(&ExportFormat::Json));
    }

    #[test]
    fn test_data_export_tools_csv_supported() {
        let tools = DataExportTools::new();
        assert!(tools.is_format_supported(&ExportFormat::Csv));
    }

    #[test]
    fn test_data_export_tools_xml_supported() {
        let tools = DataExportTools::new();
        assert!(tools.is_format_supported(&ExportFormat::Xml));
    }

    #[test]
    fn test_data_export_tools_pdf_supported() {
        let tools = DataExportTools::new();
        assert!(tools.is_format_supported(&ExportFormat::Pdf));
    }
}
