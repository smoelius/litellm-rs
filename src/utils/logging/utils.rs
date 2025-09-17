use crate::core::providers::unified_provider::ProviderError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::sync::{Arc, Mutex};
use tracing::{Level, debug, error, info, warn};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl From<LogLevel> for Level {
    fn from(log_level: LogLevel) -> Self {
        match log_level {
            LogLevel::Debug => Level::DEBUG,
            LogLevel::Info => Level::INFO,
            LogLevel::Warn => Level::WARN,
            LogLevel::Error => Level::ERROR,
        }
    }
}

impl std::str::FromStr for LogLevel {
    type Err = ProviderError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "DEBUG" => Ok(LogLevel::Debug),
            "INFO" => Ok(LogLevel::Info),
            "WARN" | "WARNING" => Ok(LogLevel::Warn),
            "ERROR" => Ok(LogLevel::Error),
            _ => Err(ProviderError::InvalidRequest {
                provider: "unknown",
                message: format!("Invalid log level: {}", s),
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub level: String,
    pub message: String,
    pub module: Option<String>,
    pub request_id: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl LogEntry {
    pub fn new(level: LogLevel, message: String) -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            level: format!("{:?}", level).to_uppercase(),
            message,
            module: None,
            request_id: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_module(mut self, module: String) -> Self {
        self.module = Some(module);
        self
    }

    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self
    }

    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

pub struct LoggingUtils;

impl LoggingUtils {
    pub fn print_verbose(message: &str, logger_only: bool, log_level: LogLevel) {
        match log_level {
            LogLevel::Debug => debug!("{}", message),
            LogLevel::Info => info!("{}", message),
            LogLevel::Warn => warn!("{}", message),
            LogLevel::Error => error!("{}", message),
        }

        if !logger_only && Self::is_verbose_enabled() {
            // For verbose mode, also print to stdout for immediate feedback
            println!("{}", message);
        }
    }

    pub fn is_verbose_enabled() -> bool {
        env::var("LITELLM_VERBOSE")
            .map(|v| v.to_lowercase())
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false)
    }

    pub fn set_verbose(enabled: bool) {
        unsafe {
            env::set_var("LITELLM_VERBOSE", if enabled { "true" } else { "false" });
        }
    }

    pub fn get_logging_id() -> String {
        Uuid::new_v4().to_string()
    }

    pub fn get_logging_id_with_timestamp(start_time: chrono::DateTime<chrono::Utc>) -> String {
        let timestamp = start_time.timestamp();
        let uuid = Uuid::new_v4();
        format!("{}-{}", timestamp, uuid)
    }

    pub fn init_logger(log_level: Option<LogLevel>) -> Result<(), ProviderError> {
        let level = log_level.unwrap_or(LogLevel::Info);

        tracing_subscriber::fmt()
            .with_max_level(Level::from(level))
            .with_target(false)
            .with_thread_ids(true)
            .with_file(true)
            .with_line_number(true)
            .init();

        Ok(())
    }

    pub fn setup_file_logging(
        log_file_path: &str,
    ) -> Result<Arc<Mutex<BufWriter<File>>>, ProviderError> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file_path)
            .map_err(|e| ProviderError::InvalidRequest {
                provider: "unknown",
                message: format!("Failed to open log file '{}': {}", log_file_path, e),
            })?;

        let writer = BufWriter::new(file);
        Ok(Arc::new(Mutex::new(writer)))
    }

    pub fn log_to_file(
        writer: &Arc<Mutex<BufWriter<File>>>,
        entry: &LogEntry,
    ) -> Result<(), ProviderError> {
        let json_entry =
            serde_json::to_string(entry).map_err(|e| ProviderError::InvalidRequest {
                provider: "unknown",
                message: format!("Failed to serialize log entry: {}", e),
            })?;

        if let Ok(mut writer_guard) = writer.lock() {
            writeln!(writer_guard, "{}", json_entry).map_err(|e| {
                ProviderError::InvalidRequest {
                    provider: "unknown",
                    message: format!("Failed to write to log file: {}", e),
                }
            })?;

            writer_guard
                .flush()
                .map_err(|e| ProviderError::InvalidRequest {
                    provider: "unknown",
                    message: format!("Failed to flush log file: {}", e),
                })?;
        }

        Ok(())
    }

    pub fn log_request_start(request_id: &str, model: &str, provider: &str, endpoint: &str) {
        let entry = LogEntry::new(LogLevel::Info, "Request started".to_string())
            .with_request_id(request_id.to_string())
            .with_metadata(
                "model".to_string(),
                serde_json::Value::String(model.to_string()),
            )
            .with_metadata(
                "provider".to_string(),
                serde_json::Value::String(provider.to_string()),
            )
            .with_metadata(
                "endpoint".to_string(),
                serde_json::Value::String(endpoint.to_string()),
            );

        info!("{}", serde_json::to_string(&entry).unwrap_or_default());
    }

    pub fn log_request_end(
        request_id: &str,
        status_code: Option<u16>,
        duration_ms: u64,
        token_usage: Option<(usize, usize)>,
    ) {
        let mut entry = LogEntry::new(LogLevel::Info, "Request completed".to_string())
            .with_request_id(request_id.to_string())
            .with_metadata(
                "duration_ms".to_string(),
                serde_json::Value::Number(duration_ms.into()),
            );

        if let Some(status) = status_code {
            entry = entry.with_metadata(
                "status_code".to_string(),
                serde_json::Value::Number(status.into()),
            );
        }

        if let Some((input_tokens, output_tokens)) = token_usage {
            entry = entry
                .with_metadata(
                    "input_tokens".to_string(),
                    serde_json::Value::Number(input_tokens.into()),
                )
                .with_metadata(
                    "output_tokens".to_string(),
                    serde_json::Value::Number(output_tokens.into()),
                );
        }

        info!("{}", serde_json::to_string(&entry).unwrap_or_default());
    }

    pub fn log_error(
        request_id: Option<&str>,
        error: &ProviderError,
        context: Option<HashMap<String, String>>,
    ) {
        let mut entry = LogEntry::new(LogLevel::Error, error.to_string());

        if let Some(id) = request_id {
            entry = entry.with_request_id(id.to_string());
        }

        if let Some(ctx) = context {
            for (key, value) in ctx {
                entry = entry.with_metadata(key, serde_json::Value::String(value));
            }
        }

        error!("{}", serde_json::to_string(&entry).unwrap_or_default());
    }

    pub fn sanitize_log_data(data: &str) -> String {
        let sensitive_patterns = [
            r"(?i)api[_-]?key[=:\s]*['\x22]?([a-zA-Z0-9\-_]+)['\x22]?",
            r"(?i)token[=:\s]*['\x22]?([a-zA-Z0-9\-_.]+)['\x22]?",
            r"(?i)password[=:\s]*['\x22]?([^\s'\x22]+)['\x22]?",
            r"(?i)secret[=:\s]*['\x22]?([^\s'\x22]+)['\x22]?",
        ];

        let mut sanitized = data.to_string();

        for pattern in &sensitive_patterns {
            let re = regex::Regex::new(pattern).unwrap();
            sanitized = re.replace_all(&sanitized, "$1***REDACTED***").to_string();
        }

        sanitized
    }

    pub fn create_structured_log(
        level: LogLevel,
        message: &str,
        module: Option<&str>,
        metadata: Option<HashMap<String, serde_json::Value>>,
    ) -> LogEntry {
        let mut entry = LogEntry::new(level, message.to_string());

        if let Some(mod_name) = module {
            entry = entry.with_module(mod_name.to_string());
        }

        if let Some(meta) = metadata {
            for (key, value) in meta {
                entry = entry.with_metadata(key, value);
            }
        }

        entry
    }

    pub fn mask_sensitive_data(input: &str) -> String {
        let sensitive_keys = [
            "api_key",
            "token",
            "password",
            "secret",
            "auth",
            "credential",
        ];

        let mut result = input.to_string();

        for key in &sensitive_keys {
            let patterns = [
                format!(r#""{}"\s*:\s*"([^"]+)""#, key),
                format!(r#"'{}'\s*:\s*'([^']+)'"#, key),
                format!(r#"{}[=:]\s*([^\s,}}\]]+)"#, key),
            ];

            for pattern in &patterns {
                if let Ok(re) = regex::Regex::new(pattern) {
                    result = re
                        .replace_all(&result, |caps: &regex::Captures| {
                            let full_match = caps.get(0).unwrap().as_str();
                            let value = caps.get(1).unwrap().as_str();
                            let masked_value = if value.len() > 8 {
                                format!("{}***{}", &value[..2], &value[value.len() - 2..])
                            } else {
                                "***".to_string()
                            };
                            full_match.replace(value, &masked_value)
                        })
                        .to_string();
                }
            }
        }

        result
    }

    pub fn get_log_level_from_env() -> LogLevel {
        env::var("LITELLM_LOG_LEVEL")
            .unwrap_or_else(|_| "INFO".to_string())
            .parse()
            .unwrap_or(LogLevel::Info)
    }

    pub fn should_log_at_level(current_level: &LogLevel, target_level: &LogLevel) -> bool {
        let current_priority = match current_level {
            LogLevel::Debug => 0,
            LogLevel::Info => 1,
            LogLevel::Warn => 2,
            LogLevel::Error => 3,
        };

        let target_priority = match target_level {
            LogLevel::Debug => 0,
            LogLevel::Info => 1,
            LogLevel::Warn => 2,
            LogLevel::Error => 3,
        };

        target_priority >= current_priority
    }

    pub fn format_duration(duration: std::time::Duration) -> String {
        let total_ms = duration.as_millis();

        if total_ms < 1000 {
            format!("{}ms", total_ms)
        } else if total_ms < 60_000 {
            format!("{:.2}s", total_ms as f64 / 1000.0)
        } else {
            let minutes = total_ms / 60_000;
            let seconds = (total_ms % 60_000) as f64 / 1000.0;
            format!("{}m {:.2}s", minutes, seconds)
        }
    }

    pub fn log_performance_metrics(
        operation: &str,
        duration: std::time::Duration,
        metadata: Option<HashMap<String, String>>,
    ) {
        let mut entry = LogEntry::new(LogLevel::Info, format!("Performance: {}", operation))
            .with_metadata(
                "duration".to_string(),
                serde_json::Value::String(Self::format_duration(duration)),
            )
            .with_metadata(
                "duration_ms".to_string(),
                serde_json::Value::Number(serde_json::Number::from(duration.as_millis() as u64)),
            );

        if let Some(meta) = metadata {
            for (key, value) in meta {
                entry = entry.with_metadata(key, serde_json::Value::String(value));
            }
        }

        info!("{}", serde_json::to_string(&entry).unwrap_or_default());
    }
}

pub struct Logger {
    level: LogLevel,
    file_writer: Option<Arc<Mutex<BufWriter<File>>>>,
}

impl Logger {
    pub fn new(level: LogLevel) -> Self {
        Self {
            level,
            file_writer: None,
        }
    }

    pub fn with_file_output(mut self, file_path: &str) -> Result<Self, ProviderError> {
        self.file_writer = Some(LoggingUtils::setup_file_logging(file_path)?);
        Ok(self)
    }

    pub fn log(&self, level: LogLevel, message: &str, context: Option<HashMap<String, String>>) {
        if !LoggingUtils::should_log_at_level(&self.level, &level) {
            return;
        }

        let mut metadata = HashMap::new();
        if let Some(ctx) = context {
            for (key, value) in ctx {
                metadata.insert(key, serde_json::Value::String(value));
            }
        }

        let entry =
            LoggingUtils::create_structured_log(level.clone(), message, None, Some(metadata));

        LoggingUtils::print_verbose(message, false, level);

        if let Some(writer) = &self.file_writer {
            let _ = LoggingUtils::log_to_file(writer, &entry);
        }
    }

    pub fn debug(&self, message: &str) {
        self.log(LogLevel::Debug, message, None);
    }

    pub fn info(&self, message: &str) {
        self.log(LogLevel::Info, message, None);
    }

    pub fn warn(&self, message: &str) {
        self.log(LogLevel::Warn, message, None);
    }

    pub fn error(&self, message: &str) {
        self.log(LogLevel::Error, message, None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        let masked = LoggingUtils::mask_sensitive_data(input);
        assert!(!masked.contains("sk-1234567890"));
        assert!(masked.contains("sk***90") || masked.contains("***"));
    }

    #[test]
    fn test_sanitize_log_data() {
        let input = "API_KEY=sk-1234567890 model=gpt-4";
        let sanitized = LoggingUtils::sanitize_log_data(input);
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
        let writer = LoggingUtils::setup_file_logging(temp_file.path().to_str().unwrap()).unwrap();

        let entry = LogEntry::new(LogLevel::Info, "Test log entry".to_string());
        assert!(LoggingUtils::log_to_file(&writer, &entry).is_ok());
    }

    #[test]
    fn test_logger_creation() {
        let logger = Logger::new(LogLevel::Info);
        logger.info("Test info message");
        logger.debug("Test debug message"); // Should not be logged due to level
    }
}
