//! File storage types and enums

use super::{LocalStorage, S3Storage};

/// File storage backend
#[derive(Debug, Clone)]
pub enum FileStorage {
    /// Local file system storage
    Local(LocalStorage),
    /// Amazon S3 storage
    S3(S3Storage),
}

/// File metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileMetadata {
    /// File ID
    pub id: String,
    /// Original filename
    pub filename: String,
    /// MIME content type
    pub content_type: String,
    /// File size in bytes
    pub size: u64,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// File checksum
    pub checksum: String,
}
