//! Error information models

use serde::{Deserialize, Serialize};

/// Error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorInfo {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
    /// Error type
    pub error_type: String,
    /// Provider error code
    pub provider_code: Option<String>,
    /// Stack trace
    pub stack_trace: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== ErrorInfo Structure Tests ====================

    #[test]
    fn test_error_info_structure() {
        let error = ErrorInfo {
            code: "ERR001".to_string(),
            message: "Something went wrong".to_string(),
            error_type: "internal_error".to_string(),
            provider_code: Some("PROVIDER_500".to_string()),
            stack_trace: Some("at function_a\nat function_b".to_string()),
        };
        assert_eq!(error.code, "ERR001");
        assert_eq!(error.message, "Something went wrong");
        assert_eq!(error.error_type, "internal_error");
        assert!(error.provider_code.is_some());
        assert!(error.stack_trace.is_some());
    }

    #[test]
    fn test_error_info_minimal() {
        let error = ErrorInfo {
            code: "E100".to_string(),
            message: "Error occurred".to_string(),
            error_type: "client_error".to_string(),
            provider_code: None,
            stack_trace: None,
        };
        assert_eq!(error.code, "E100");
        assert!(error.provider_code.is_none());
        assert!(error.stack_trace.is_none());
    }

    #[test]
    fn test_error_info_rate_limit() {
        let error = ErrorInfo {
            code: "RATE_LIMIT".to_string(),
            message: "Rate limit exceeded".to_string(),
            error_type: "rate_limit_error".to_string(),
            provider_code: Some("429".to_string()),
            stack_trace: None,
        };
        assert_eq!(error.code, "RATE_LIMIT");
        assert_eq!(error.error_type, "rate_limit_error");
        assert_eq!(error.provider_code, Some("429".to_string()));
    }

    #[test]
    fn test_error_info_auth_error() {
        let error = ErrorInfo {
            code: "AUTH_FAILED".to_string(),
            message: "Invalid API key".to_string(),
            error_type: "authentication_error".to_string(),
            provider_code: Some("401".to_string()),
            stack_trace: None,
        };
        assert_eq!(error.error_type, "authentication_error");
    }

    #[test]
    fn test_error_info_timeout() {
        let error = ErrorInfo {
            code: "TIMEOUT".to_string(),
            message: "Request timed out after 30 seconds".to_string(),
            error_type: "timeout_error".to_string(),
            provider_code: None,
            stack_trace: None,
        };
        assert_eq!(error.code, "TIMEOUT");
        assert_eq!(error.error_type, "timeout_error");
    }

    // ==================== ErrorInfo Serialization Tests ====================

    #[test]
    fn test_error_info_serialization() {
        let error = ErrorInfo {
            code: "SER001".to_string(),
            message: "Serialization test".to_string(),
            error_type: "test_error".to_string(),
            provider_code: Some("P001".to_string()),
            stack_trace: Some("stack".to_string()),
        };
        let json = serde_json::to_value(&error).unwrap();
        assert_eq!(json["code"], "SER001");
        assert_eq!(json["message"], "Serialization test");
        assert_eq!(json["error_type"], "test_error");
        assert_eq!(json["provider_code"], "P001");
        assert_eq!(json["stack_trace"], "stack");
    }

    #[test]
    fn test_error_info_serialization_with_nulls() {
        let error = ErrorInfo {
            code: "NULL".to_string(),
            message: "No optional fields".to_string(),
            error_type: "basic".to_string(),
            provider_code: None,
            stack_trace: None,
        };
        let json = serde_json::to_value(&error).unwrap();
        assert_eq!(json["code"], "NULL");
        assert!(json["provider_code"].is_null());
        assert!(json["stack_trace"].is_null());
    }

    #[test]
    fn test_error_info_deserialization() {
        let json = r#"{
            "code": "DES001",
            "message": "Deserialization test",
            "error_type": "validation_error",
            "provider_code": "VAL_ERR",
            "stack_trace": "line 1\nline 2"
        }"#;
        let error: ErrorInfo = serde_json::from_str(json).unwrap();
        assert_eq!(error.code, "DES001");
        assert_eq!(error.message, "Deserialization test");
        assert_eq!(error.error_type, "validation_error");
        assert_eq!(error.provider_code, Some("VAL_ERR".to_string()));
        assert!(error.stack_trace.unwrap().contains("line 1"));
    }

    #[test]
    fn test_error_info_deserialization_minimal() {
        let json = r#"{
            "code": "MIN",
            "message": "Minimal",
            "error_type": "minimal_error"
        }"#;
        let error: ErrorInfo = serde_json::from_str(json).unwrap();
        assert_eq!(error.code, "MIN");
        assert!(error.provider_code.is_none());
        assert!(error.stack_trace.is_none());
    }

    // ==================== ErrorInfo Clone Tests ====================

    #[test]
    fn test_error_info_clone() {
        let error = ErrorInfo {
            code: "CLONE".to_string(),
            message: "Clone test".to_string(),
            error_type: "clone_error".to_string(),
            provider_code: Some("PROV".to_string()),
            stack_trace: Some("trace".to_string()),
        };
        let cloned = error.clone();
        assert_eq!(error.code, cloned.code);
        assert_eq!(error.message, cloned.message);
        assert_eq!(error.error_type, cloned.error_type);
        assert_eq!(error.provider_code, cloned.provider_code);
        assert_eq!(error.stack_trace, cloned.stack_trace);
    }

    // ==================== ErrorInfo Edge Cases ====================

    #[test]
    fn test_error_info_long_message() {
        let long_message = "x".repeat(10000);
        let error = ErrorInfo {
            code: "LONG".to_string(),
            message: long_message.clone(),
            error_type: "long_error".to_string(),
            provider_code: None,
            stack_trace: None,
        };
        assert_eq!(error.message.len(), 10000);
    }

    #[test]
    fn test_error_info_special_characters() {
        let error = ErrorInfo {
            code: "SPECIAL".to_string(),
            message: "Error with \"quotes\" and \\ backslash".to_string(),
            error_type: "special_chars".to_string(),
            provider_code: None,
            stack_trace: None,
        };
        let json = serde_json::to_string(&error).unwrap();
        let parsed: ErrorInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(error.message, parsed.message);
    }

    #[test]
    fn test_error_info_unicode() {
        let error = ErrorInfo {
            code: "UNICODE".to_string(),
            message: "错误消息 🚨 エラー".to_string(),
            error_type: "unicode_error".to_string(),
            provider_code: None,
            stack_trace: None,
        };
        let json = serde_json::to_string(&error).unwrap();
        let parsed: ErrorInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(error.message, parsed.message);
    }
}
