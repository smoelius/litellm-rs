//! File storage configuration

use serde::{Deserialize, Serialize};

/// File storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileStorageConfig {
    /// Storage type (local, s3, etc.)
    pub storage_type: String,
    /// Local storage path
    pub local_path: Option<String>,
    /// S3 configuration
    pub s3: Option<S3Config>,
}

impl Default for FileStorageConfig {
    fn default() -> Self {
        Self {
            storage_type: "local".to_string(),
            local_path: Some("./data".to_string()),
            s3: None,
        }
    }
}

/// S3 configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3Config {
    /// S3 bucket name
    pub bucket: String,
    /// AWS region
    pub region: String,
    /// Access key ID
    pub access_key_id: String,
    /// Secret access key
    pub secret_access_key: String,
    /// Endpoint URL (for S3-compatible services)
    pub endpoint: Option<String>,
}

/// Vector database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorDbConfig {
    /// Vector DB type (pinecone, weaviate, etc.)
    pub db_type: String,
    /// Connection URL
    pub url: String,
    /// API key
    pub api_key: String,
    /// Index name
    pub index_name: String,
}

impl Default for VectorDbConfig {
    fn default() -> Self {
        Self {
            db_type: "pinecone".to_string(),
            url: String::new(),
            api_key: String::new(),
            index_name: "default".to_string(),
        }
    }
}

/// Alerting configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AlertingConfig {
    /// Enable alerting
    #[serde(default)]
    pub enabled: bool,
    /// Slack webhook URL
    pub slack_webhook: Option<String>,
    /// Email configuration
    pub email: Option<EmailConfig>,
}

/// Email configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    /// SMTP server
    pub smtp_server: String,
    /// SMTP port
    pub smtp_port: u16,
    /// Username
    pub username: String,
    /// Password
    pub password: String,
    /// From address
    pub from_address: String,
    /// To addresses
    pub to_addresses: Vec<String>,
}
