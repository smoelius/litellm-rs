use super::types::LogEntry;
use crate::core::providers::unified_provider::ProviderError;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::sync::{Arc, Mutex};

pub struct FileLogging;

impl FileLogging {
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
}
