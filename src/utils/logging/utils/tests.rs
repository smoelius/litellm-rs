#[cfg(test)]
mod tests {
    use super::super::file_logging::FileLogging;
    use super::super::logger::Logger;
    use super::super::sanitization::Sanitization;
    use super::super::types::{LogEntry, LogLevel};
    use super::super::utils::LoggingUtils;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_level_from_string() {
        assert_eq!("DEBUG".parse::<LogLevel>().unwrap(), LogLevel::Debug);
        assert_eq!("INFO".parse::<LogLevel>().unwrap(), LogLevel::Info);
        assert_eq!("WARN".parse::<LogLevel>().unwrap(), LogLevel::Warn);
        assert_eq!("ERROR".parse::<LogLevel>().unwrap(), LogLevel::Error);
        assert!("INVALID".parse::<LogLevel>().is_err());
    }

    #[test]
    fn test_log_entry_creation() {
        let entry = LogEntry::new(LogLevel::Info, "Test message".to_string())
            .with_module("test_module".to_string())
            .with_request_id("req-123".to_string())
            .with_metadata(
                "key".to_string(),
                serde_json::Value::String("value".to_string()),
            );

        assert_eq!(entry.level, "INFO");
        assert_eq!(entry.message, "Test message");
        assert_eq!(entry.module, Some("test_module".to_string()));
        assert_eq!(entry.request_id, Some("req-123".to_string()));
        assert!(entry.metadata.contains_key("key"));
    }

    #[test]
    fn test_should_log_at_level() {
        assert!(LoggingUtils::should_log_at_level(
            &LogLevel::Debug,
            &LogLevel::Error
        ));
        assert!(!LoggingUtils::should_log_at_level(
            &LogLevel::Error,
            &LogLevel::Debug
        ));
        assert!(LoggingUtils::should_log_at_level(
            &LogLevel::Info,
            &LogLevel::Info
        ));
    }

    #[test]
    fn test_mask_sensitive_data() {
        let input = r#"{"api_key": "sk-1234567890", "model": "gpt-4"}"#;
        let masked = Sanitization::mask_sensitive_data(input);
        assert!(!masked.contains("sk-1234567890"));
        assert!(masked.contains("sk***90") || masked.contains("***"));
    }

    #[test]
    fn test_sanitize_log_data() {
        let input = "API_KEY=sk-1234567890 model=gpt-4";
        let sanitized = Sanitization::sanitize_log_data(input);
        assert!(!sanitized.contains("sk-1234567890"));
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(
            LoggingUtils::format_duration(std::time::Duration::from_millis(500)),
            "500ms"
        );
        assert_eq!(
            LoggingUtils::format_duration(std::time::Duration::from_secs(2)),
            "2.00s"
        );
        assert!(LoggingUtils::format_duration(std::time::Duration::from_secs(65)).contains("1m"));
    }

    #[test]
    fn test_logging_id_generation() {
        let id1 = LoggingUtils::get_logging_id();
        let id2 = LoggingUtils::get_logging_id();
        assert_ne!(id1, id2);
        assert!(uuid::Uuid::parse_str(&id1).is_ok());
    }

    #[test]
    fn test_file_logging() {
        let temp_file = NamedTempFile::new().unwrap();
        let writer = FileLogging::setup_file_logging(temp_file.path().to_str().unwrap()).unwrap();

        let entry = LogEntry::new(LogLevel::Info, "Test log entry".to_string());
        assert!(FileLogging::log_to_file(&writer, &entry).is_ok());
    }

    #[test]
    fn test_logger_creation() {
        let logger = Logger::new(LogLevel::Info);
        logger.info("Test info message");
        logger.debug("Test debug message"); // Should not be logged due to level
    }
}
