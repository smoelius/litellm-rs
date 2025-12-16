//! Security-aware logging utilities

use crate::utils::logging::logging::async_logger::async_logger;
use std::collections::HashMap;
use tracing::Level;
use uuid::Uuid;

/// Security-aware logging utilities
#[allow(dead_code)]
pub struct SecurityLogger;

#[allow(dead_code)]
impl SecurityLogger {
    /// Log authentication events
    pub fn log_auth_event(
        event_type: &str,
        user_id: Option<Uuid>,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
        success: bool,
        details: Option<&str>,
    ) {
        let mut fields = HashMap::new();
        fields.insert(
            "event_type".to_string(),
            serde_json::Value::String(event_type.to_string()),
        );
        fields.insert("success".to_string(), serde_json::Value::Bool(success));

        if let Some(ip) = ip_address {
            fields.insert(
                "ip_address".to_string(),
                serde_json::Value::String(ip.to_string()),
            );
        }

        if let Some(ua) = user_agent {
            // Truncate user agent to prevent log injection
            let safe_ua = ua.chars().take(200).collect::<String>();
            fields.insert("user_agent".to_string(), serde_json::Value::String(safe_ua));
        }

        if let Some(details) = details {
            fields.insert(
                "details".to_string(),
                serde_json::Value::String(details.to_string()),
            );
        }

        let level = if success { Level::INFO } else { Level::WARN };
        let message = format!(
            "Authentication {}: {}",
            if success { "success" } else { "failure" },
            event_type
        );

        if let Some(logger) = async_logger() {
            logger.log_structured(level, "security", &message, fields, None, user_id);
        }
    }

    /// Log authorization events
    pub fn log_authz_event(
        user_id: Uuid,
        resource: &str,
        action: &str,
        granted: bool,
        reason: Option<&str>,
    ) {
        let mut fields = HashMap::new();
        fields.insert(
            "resource".to_string(),
            serde_json::Value::String(resource.to_string()),
        );
        fields.insert(
            "action".to_string(),
            serde_json::Value::String(action.to_string()),
        );
        fields.insert("granted".to_string(), serde_json::Value::Bool(granted));

        if let Some(reason) = reason {
            fields.insert(
                "reason".to_string(),
                serde_json::Value::String(reason.to_string()),
            );
        }

        let level = if granted { Level::DEBUG } else { Level::WARN };
        let message = format!(
            "Authorization {}: {} on {}",
            if granted { "granted" } else { "denied" },
            action,
            resource
        );

        if let Some(logger) = async_logger() {
            logger.log_structured(level, "security", &message, fields, None, Some(user_id));
        }
    }

    /// Log security violations
    pub fn log_security_violation(
        violation_type: &str,
        severity: &str,
        description: &str,
        user_id: Option<Uuid>,
        ip_address: Option<&str>,
        additional_data: Option<HashMap<String, serde_json::Value>>,
    ) {
        let mut fields = HashMap::new();
        fields.insert(
            "violation_type".to_string(),
            serde_json::Value::String(violation_type.to_string()),
        );
        fields.insert(
            "severity".to_string(),
            serde_json::Value::String(severity.to_string()),
        );

        if let Some(ip) = ip_address {
            fields.insert(
                "ip_address".to_string(),
                serde_json::Value::String(ip.to_string()),
            );
        }

        if let Some(data) = additional_data {
            for (key, value) in data {
                fields.insert(key, value);
            }
        }

        let level = match severity.to_lowercase().as_str() {
            "critical" | "high" => Level::ERROR,
            "medium" => Level::WARN,
            _ => Level::INFO,
        };

        if let Some(logger) = async_logger() {
            logger.log_structured(level, "security", description, fields, None, user_id);
        }
    }
}
