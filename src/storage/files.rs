//! File storage implementation
//!
//! This module provides file storage functionality with support for local and cloud storage.

use crate::config::{FileStorageConfig, S3Config};
use crate::utils::error::{GatewayError, Result};
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{debug, info};
use uuid::Uuid;

#[cfg(feature = "s3")]
use aws_config;
#[cfg(feature = "s3")]
use aws_sdk_s3 as aws_s3;

/// File storage backend
#[derive(Debug, Clone)]
pub enum FileStorage {
    /// Local file system storage
    Local(LocalStorage),
    /// Amazon S3 storage
    S3(S3Storage),
}

/// Local file storage
#[derive(Debug, Clone)]
pub struct LocalStorage {
    base_path: PathBuf,
}

/// S3 file storage
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct S3Storage {
    bucket: String,
    region: String,
    #[cfg(feature = "s3")]
    client: Option<aws_s3::Client>,
    #[cfg(not(feature = "s3"))]
    client: Option<()>, // Placeholder when S3 feature is disabled
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

#[allow(dead_code)]
impl FileStorage {
    /// Create a new file storage instance
    pub async fn new(config: &FileStorageConfig) -> Result<Self> {
        info!("Initializing file storage: {}", config.storage_type);

        match config.storage_type.as_str() {
            "local" => {
                let path = config
                    .local_path
                    .as_ref()
                    .ok_or_else(|| GatewayError::Config("Local path not specified".to_string()))?;
                Ok(FileStorage::Local(LocalStorage::new(path).await?))
            }
            "s3" => {
                let s3_config = config.s3.as_ref().ok_or_else(|| {
                    GatewayError::Config("S3 configuration not specified".to_string())
                })?;
                Ok(FileStorage::S3(S3Storage::new(s3_config).await?))
            }
            _ => Err(GatewayError::Config(format!(
                "Unsupported storage type: {}",
                config.storage_type
            ))),
        }
    }

    /// Store a file and return its ID
    pub async fn store(&self, filename: &str, content: &[u8]) -> Result<String> {
        match self {
            FileStorage::Local(storage) => storage.store(filename, content).await,
            FileStorage::S3(storage) => storage.store(filename, content).await,
        }
    }

    /// Retrieve file content by ID
    pub async fn get(&self, file_id: &str) -> Result<Vec<u8>> {
        match self {
            FileStorage::Local(storage) => storage.get(file_id).await,
            FileStorage::S3(storage) => storage.get(file_id).await,
        }
    }

    /// Delete a file by ID
    pub async fn delete(&self, file_id: &str) -> Result<()> {
        match self {
            FileStorage::Local(storage) => storage.delete(file_id).await,
            FileStorage::S3(storage) => storage.delete(file_id).await,
        }
    }

    /// Check if a file exists
    pub async fn exists(&self, file_id: &str) -> Result<bool> {
        match self {
            FileStorage::Local(storage) => storage.exists(file_id).await,
            FileStorage::S3(storage) => storage.exists(file_id).await,
        }
    }

    /// Get file metadata
    pub async fn metadata(&self, file_id: &str) -> Result<FileMetadata> {
        match self {
            FileStorage::Local(storage) => storage.metadata(file_id).await,
            FileStorage::S3(storage) => storage.metadata(file_id).await,
        }
    }

    /// List files with pagination
    pub async fn list(&self, prefix: Option<&str>, limit: Option<usize>) -> Result<Vec<String>> {
        match self {
            FileStorage::Local(storage) => storage.list(prefix, limit).await,
            FileStorage::S3(storage) => storage.list(prefix, limit).await,
        }
    }

    /// Health check
    pub async fn health_check(&self) -> Result<()> {
        match self {
            FileStorage::Local(storage) => storage.health_check().await,
            FileStorage::S3(storage) => storage.health_check().await,
        }
    }

    /// Close storage connections
    pub async fn close(&self) -> Result<()> {
        match self {
            FileStorage::Local(storage) => storage.close().await,
            FileStorage::S3(storage) => storage.close().await,
        }
    }
}

#[allow(dead_code)]
impl LocalStorage {
    /// Create a new local storage instance
    pub async fn new(base_path: &str) -> Result<Self> {
        let path = PathBuf::from(base_path);

        // Create directory if it doesn't exist
        if !path.exists() {
            fs::create_dir_all(&path).await.map_err(|e| {
                GatewayError::FileStorage(format!("Failed to create storage directory: {}", e))
            })?;
        }

        info!("Local file storage initialized at: {}", path.display());
        Ok(Self { base_path: path })
    }

    /// Store a file
    pub async fn store(&self, filename: &str, content: &[u8]) -> Result<String> {
        let file_id = Uuid::new_v4().to_string();
        let file_path = self.get_file_path(&file_id);

        // Create subdirectories if needed
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                GatewayError::FileStorage(format!("Failed to create directory: {}", e))
            })?;
        }

        // Write file content
        let mut file = fs::File::create(&file_path)
            .await
            .map_err(|e| GatewayError::FileStorage(format!("Failed to create file: {}", e)))?;

        file.write_all(content)
            .await
            .map_err(|e| GatewayError::FileStorage(format!("Failed to write file: {}", e)))?;

        // Store metadata
        let metadata = FileMetadata {
            id: file_id.clone(),
            filename: filename.to_string(),
            content_type: Self::detect_content_type(filename),
            size: content.len() as u64,
            created_at: chrono::Utc::now(),
            checksum: Self::calculate_checksum(content),
        };

        self.store_metadata(&file_id, &metadata).await?;

        debug!("File stored: {} -> {}", filename, file_id);
        Ok(file_id)
    }

    /// Retrieve file content
    pub async fn get(&self, file_id: &str) -> Result<Vec<u8>> {
        let file_path = self.get_file_path(file_id);

        if !file_path.exists() {
            return Err(GatewayError::NotFound(format!(
                "File not found: {}",
                file_id
            )));
        }

        let mut file = fs::File::open(&file_path)
            .await
            .map_err(|e| GatewayError::FileStorage(format!("Failed to open file: {}", e)))?;

        let mut content = Vec::new();
        file.read_to_end(&mut content)
            .await
            .map_err(|e| GatewayError::FileStorage(format!("Failed to read file: {}", e)))?;

        Ok(content)
    }

    /// Delete a file
    pub async fn delete(&self, file_id: &str) -> Result<()> {
        let file_path = self.get_file_path(file_id);
        let metadata_path = self.get_metadata_path(file_id);

        // Delete file
        if file_path.exists() {
            fs::remove_file(&file_path)
                .await
                .map_err(|e| GatewayError::FileStorage(format!("Failed to delete file: {}", e)))?;
        }

        // Delete metadata
        if metadata_path.exists() {
            fs::remove_file(&metadata_path).await.map_err(|e| {
                GatewayError::FileStorage(format!("Failed to delete metadata: {}", e))
            })?;
        }

        debug!("File deleted: {}", file_id);
        Ok(())
    }

    /// Check if file exists
    pub async fn exists(&self, file_id: &str) -> Result<bool> {
        let file_path = self.get_file_path(file_id);
        Ok(file_path.exists())
    }

    /// Get file metadata
    pub async fn metadata(&self, file_id: &str) -> Result<FileMetadata> {
        let metadata_path = self.get_metadata_path(file_id);

        if !metadata_path.exists() {
            return Err(GatewayError::NotFound(format!(
                "File metadata not found: {}",
                file_id
            )));
        }

        let content = fs::read_to_string(&metadata_path)
            .await
            .map_err(|e| GatewayError::FileStorage(format!("Failed to read metadata: {}", e)))?;

        let metadata: FileMetadata = serde_json::from_str(&content)
            .map_err(|e| GatewayError::FileStorage(format!("Failed to parse metadata: {}", e)))?;

        Ok(metadata)
    }

    /// List files
    pub async fn list(&self, prefix: Option<&str>, limit: Option<usize>) -> Result<Vec<String>> {
        let mut files = Vec::new();
        let mut entries = fs::read_dir(&self.base_path)
            .await
            .map_err(|e| GatewayError::FileStorage(format!("Failed to read directory: {}", e)))?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| GatewayError::FileStorage(format!("Failed to read entry: {}", e)))?
        {
            let file_name = entry.file_name().to_string_lossy().to_string();

            // Skip metadata files
            if file_name.ends_with(".meta") {
                continue;
            }

            // Apply prefix filter
            if let Some(prefix) = prefix {
                if !file_name.starts_with(prefix) {
                    continue;
                }
            }

            files.push(file_name);

            // Apply limit
            if let Some(limit) = limit {
                if files.len() >= limit {
                    break;
                }
            }
        }

        Ok(files)
    }

    /// Health check
    pub async fn health_check(&self) -> Result<()> {
        // Check if base directory is accessible
        if !self.base_path.exists() {
            return Err(GatewayError::FileStorage(
                "Storage directory does not exist".to_string(),
            ));
        }

        // Try to write a test file
        let test_file = self.base_path.join(".health_check");
        fs::write(&test_file, b"health_check")
            .await
            .map_err(|e| GatewayError::FileStorage(format!("Storage not writable: {}", e)))?;

        // Clean up test file
        let _ = fs::remove_file(&test_file).await;

        Ok(())
    }

    /// Close storage (no-op for local storage)
    pub async fn close(&self) -> Result<()> {
        Ok(())
    }

    /// Get file path for a given file ID
    fn get_file_path(&self, file_id: &str) -> PathBuf {
        // Use first two characters as subdirectory for better distribution
        let subdir = &file_id[..2.min(file_id.len())];
        self.base_path.join(subdir).join(file_id)
    }

    /// Get metadata path for a given file ID
    fn get_metadata_path(&self, file_id: &str) -> PathBuf {
        let subdir = &file_id[..2.min(file_id.len())];
        self.base_path
            .join(subdir)
            .join(format!("{}.meta", file_id))
    }

    /// Store file metadata
    async fn store_metadata(&self, file_id: &str, metadata: &FileMetadata) -> Result<()> {
        let metadata_path = self.get_metadata_path(file_id);

        if let Some(parent) = metadata_path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                GatewayError::FileStorage(format!("Failed to create metadata directory: {}", e))
            })?;
        }

        let content = serde_json::to_string_pretty(metadata).map_err(|e| {
            GatewayError::FileStorage(format!("Failed to serialize metadata: {}", e))
        })?;

        fs::write(&metadata_path, content)
            .await
            .map_err(|e| GatewayError::FileStorage(format!("Failed to write metadata: {}", e)))?;

        Ok(())
    }

    /// Detect content type from filename
    fn detect_content_type(filename: &str) -> String {
        match Path::new(filename).extension().and_then(|ext| ext.to_str()) {
            Some("txt") => "text/plain".to_string(),
            Some("json") => "application/json".to_string(),
            Some("xml") => "application/xml".to_string(),
            Some("html") => "text/html".to_string(),
            Some("css") => "text/css".to_string(),
            Some("js") => "application/javascript".to_string(),
            Some("png") => "image/png".to_string(),
            Some("jpg") | Some("jpeg") => "image/jpeg".to_string(),
            Some("gif") => "image/gif".to_string(),
            Some("pdf") => "application/pdf".to_string(),
            Some("zip") => "application/zip".to_string(),
            _ => "application/octet-stream".to_string(),
        }
    }

    /// Calculate file checksum
    fn calculate_checksum(content: &[u8]) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(content);
        hex::encode(hasher.finalize())
    }
}

#[allow(dead_code)]
impl S3Storage {
    /// Create a new S3 storage instance
    pub async fn new(config: &S3Config) -> Result<Self> {
        info!(
            "S3 file storage initialized: bucket={}, region={}",
            config.bucket, config.region
        );

        #[cfg(feature = "s3")]
        {
            use aws_s3::config::Region;

            let region = Region::new(config.region.clone());
            let aws_config = aws_config::defaults(aws_config::BehaviorVersion::latest())
                .region(region)
                .load()
                .await;

            let client = aws_s3::Client::new(&aws_config);

            Ok(Self {
                bucket: config.bucket.clone(),
                region: config.region.clone(),
                client: Some(client),
            })
        }

        #[cfg(not(feature = "s3"))]
        {
            Ok(Self {
                bucket: config.bucket.clone(),
                region: config.region.clone(),
                client: None,
            })
        }
    }

    /// Store a file to S3
    #[allow(unused_variables)]
    pub async fn store(&self, filename: &str, content: &[u8]) -> Result<String> {
        #[cfg(feature = "s3")]
        {
            if let Some(client) = &self.client {
                use aws_s3::primitives::ByteStream;

                let file_id = Uuid::new_v4().to_string();
                let key = format!("{}/{}", file_id, filename);

                client
                    .put_object()
                    .bucket(&self.bucket)
                    .key(&key)
                    .body(ByteStream::from(content.to_vec()))
                    .send()
                    .await
                    .map_err(|e| GatewayError::FileStorage(format!("S3 upload failed: {}", e)))?;

                debug!("File uploaded to S3: {}", key);
                Ok(file_id)
            } else {
                Err(GatewayError::FileStorage(
                    "S3 client not initialized".to_string(),
                ))
            }
        }

        #[cfg(not(feature = "s3"))]
        {
            Err(GatewayError::FileStorage(
                "S3 feature not enabled".to_string(),
            ))
        }
    }

    /// Retrieve file content from S3
    #[allow(unused_variables)]
    pub async fn get(&self, file_id: &str) -> Result<Vec<u8>> {
        #[cfg(feature = "s3")]
        {
            if let Some(client) = &self.client {
                let result = client
                    .get_object()
                    .bucket(&self.bucket)
                    .key(file_id)
                    .send()
                    .await
                    .map_err(|e| GatewayError::FileStorage(format!("S3 download failed: {}", e)))?;

                let bytes = result.body.collect().await.map_err(|e| {
                    GatewayError::FileStorage(format!("Failed to read S3 content: {}", e))
                })?;

                Ok(bytes.to_vec())
            } else {
                Err(GatewayError::FileStorage(
                    "S3 client not initialized".to_string(),
                ))
            }
        }

        #[cfg(not(feature = "s3"))]
        {
            Err(GatewayError::FileStorage(
                "S3 feature not enabled".to_string(),
            ))
        }
    }

    /// Delete a file from S3
    #[allow(unused_variables)]
    pub async fn delete(&self, file_id: &str) -> Result<()> {
        #[cfg(feature = "s3")]
        {
            if let Some(client) = &self.client {
                client
                    .delete_object()
                    .bucket(&self.bucket)
                    .key(file_id)
                    .send()
                    .await
                    .map_err(|e| GatewayError::FileStorage(format!("S3 deletion failed: {}", e)))?;

                debug!("File deleted from S3: {}", file_id);
                Ok(())
            } else {
                Err(GatewayError::FileStorage(
                    "S3 client not initialized".to_string(),
                ))
            }
        }

        #[cfg(not(feature = "s3"))]
        {
            Err(GatewayError::FileStorage(
                "S3 feature not enabled".to_string(),
            ))
        }
    }

    /// Check if file exists (placeholder implementation)
    pub async fn exists(&self, _file_id: &str) -> Result<bool> {
        Err(GatewayError::FileStorage(
            "S3 storage not implemented yet".to_string(),
        ))
    }

    /// Get file metadata (placeholder implementation)
    pub async fn metadata(&self, _file_id: &str) -> Result<FileMetadata> {
        Err(GatewayError::FileStorage(
            "S3 storage not implemented yet".to_string(),
        ))
    }

    /// List files (placeholder implementation)
    pub async fn list(&self, _prefix: Option<&str>, _limit: Option<usize>) -> Result<Vec<String>> {
        Err(GatewayError::FileStorage(
            "S3 storage not implemented yet".to_string(),
        ))
    }

    /// Health check (placeholder implementation)
    pub async fn health_check(&self) -> Result<()> {
        Err(GatewayError::FileStorage(
            "S3 storage not implemented yet".to_string(),
        ))
    }

    /// Close storage (placeholder implementation)
    pub async fn close(&self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_local_storage() {
        let temp_dir = TempDir::new().unwrap();
        let storage = LocalStorage::new(temp_dir.path().to_str().unwrap())
            .await
            .unwrap();

        // Test store
        let content = b"Hello, World!";
        let file_id = storage.store("test.txt", content).await.unwrap();
        assert!(!file_id.is_empty());

        // Test exists
        assert!(storage.exists(&file_id).await.unwrap());

        // Test get
        let retrieved = storage.get(&file_id).await.unwrap();
        assert_eq!(retrieved, content);

        // Test metadata
        let metadata = storage.metadata(&file_id).await.unwrap();
        assert_eq!(metadata.filename, "test.txt");
        assert_eq!(metadata.size, content.len() as u64);

        // Test delete
        storage.delete(&file_id).await.unwrap();
        assert!(!storage.exists(&file_id).await.unwrap());
    }

    #[test]
    fn test_content_type_detection() {
        assert_eq!(LocalStorage::detect_content_type("test.txt"), "text/plain");
        assert_eq!(
            LocalStorage::detect_content_type("data.json"),
            "application/json"
        );
        assert_eq!(LocalStorage::detect_content_type("image.png"), "image/png");
        assert_eq!(
            LocalStorage::detect_content_type("unknown"),
            "application/octet-stream"
        );
    }
}
