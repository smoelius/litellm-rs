//! Module

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClientConfig {
    /// Default
    pub default_provider: Option<String>,
    /// Configuration
    pub providers: Vec<ProviderConfig>,
    /// Settings
    pub settings: ClientSettings,
}

/// Settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientSettings {
    /// Request
    pub timeout: u64,
    /// Number of retries
    pub max_retries: u32,
    /// Request
    pub max_concurrent_requests: u32,
    /// Request
    pub enable_logging: bool,
    /// Enable metrics collection
    pub enable_metrics: bool,
}

impl Default for ClientSettings {
    fn default() -> Self {
        Self {
            timeout: 30,
            max_retries: 3,
            max_concurrent_requests: 100,
            enable_logging: true,
            enable_metrics: true,
        }
    }
}

/// Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Provider unique ID
    pub id: String,
    /// Provider type
    pub provider_type: ProviderType,
    /// Display name
    pub name: String,
    /// API key
    pub api_key: String,
    /// Base URL (optional)
    pub base_url: Option<String>,
    /// Model
    pub models: Vec<String>,
    /// Enabled status
    pub enabled: bool,
    /// Weight (for load balancing)
    pub weight: f32,
    /// Request
    pub rate_limit_rpm: Option<u32>,
    /// Token limit per minute
    pub rate_limit_tpm: Option<u32>,
    /// Settings
    pub settings: HashMap<String, serde_json::Value>,
}

/// Provider type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderType {
    /// OpenAI provider (GPT models)
    OpenAI,
    /// Anthropic provider (Claude models)
    Anthropic,
    /// Azure OpenAI provider
    Azure,
    /// Google provider (PaLM, Gemini models)
    Google,
    /// Cohere provider
    Cohere,
    /// Hugging Face provider
    HuggingFace,
    /// Ollama provider (local models)
    Ollama,
    /// AWS Bedrock provider
    AwsBedrock,
    /// Google Vertex AI provider
    GoogleVertex,
    /// Mistral provider
    Mistral,
    /// Custom provider with specified name
    Custom(String),
}

impl From<&str> for ProviderType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "openai" => ProviderType::OpenAI,
            "anthropic" => ProviderType::Anthropic,
            "azure" => ProviderType::Azure,
            "google" => ProviderType::Google,
            "cohere" => ProviderType::Cohere,
            "huggingface" => ProviderType::HuggingFace,
            "ollama" => ProviderType::Ollama,
            "aws_bedrock" => ProviderType::AwsBedrock,
            "google_vertex" => ProviderType::GoogleVertex,
            "mistral" => ProviderType::Mistral,
            _ => ProviderType::Custom(s.to_string()),
        }
    }
}

/// Configuration
pub struct ConfigBuilder {
    config: ClientConfig,
}

impl ConfigBuilder {
    /// Create a new configuration builder
    pub fn new() -> Self {
        Self {
            config: ClientConfig::default(),
        }
    }

    /// Default
    pub fn default_provider(mut self, provider_id: &str) -> Self {
        self.config.default_provider = Some(provider_id.to_string());
        self
    }

    /// Add provider
    pub fn add_provider(mut self, provider: ProviderConfig) -> Self {
        self.config.providers.push(provider);
        self
    }

    /// Add OpenAI provider
    pub fn add_openai(self, id: &str, api_key: &str) -> Self {
        self.add_provider(ProviderConfig {
            id: id.to_string(),
            provider_type: ProviderType::OpenAI,
            name: "OpenAI".to_string(),
            api_key: api_key.to_string(),
            base_url: None,
            models: vec!["gpt-4".to_string(), "gpt-3.5-turbo".to_string()],
            enabled: true,
            weight: 1.0,
            rate_limit_rpm: Some(3000),
            rate_limit_tpm: Some(250000),
            settings: HashMap::new(),
        })
    }

    /// Add Anthropic provider
    pub fn add_anthropic(self, id: &str, api_key: &str) -> Self {
        self.add_provider(ProviderConfig {
            id: id.to_string(),
            provider_type: ProviderType::Anthropic,
            name: "Anthropic".to_string(),
            api_key: api_key.to_string(),
            base_url: None,
            models: vec![
                "claude-3-opus-20240229".to_string(),
                "claude-3-sonnet-20240229".to_string(),
            ],
            enabled: true,
            weight: 1.0,
            rate_limit_rpm: Some(1000),
            rate_limit_tpm: Some(100000),
            settings: HashMap::new(),
        })
    }

    /// Settings
    pub fn timeout(mut self, timeout: u64) -> Self {
        self.config.settings.timeout = timeout;
        self
    }

    /// Settings
    pub fn max_retries(mut self, retries: u32) -> Self {
        self.config.settings.max_retries = retries;
        self
    }

    /// Configuration
    pub fn build(self) -> ClientConfig {
        self.config
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration
impl ClientConfig {
    /// Configuration
    pub fn from_env() -> crate::sdk::errors::Result<Self> {
        let mut builder = ConfigBuilder::new();

        // Configuration
        if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
            builder = builder.add_openai("openai", &api_key);
        }

        // Configuration
        if let Ok(api_key) = std::env::var("ANTHROPIC_API_KEY") {
            builder = builder.add_anthropic("anthropic", &api_key);
        }

        let config = builder.build();

        if config.providers.is_empty() {
            return Err(crate::sdk::errors::SDKError::ConfigError(
                "No providers configured. Please set OPENAI_API_KEY or ANTHROPIC_API_KEY environment variables.".to_string()
            ));
        }

        Ok(config)
    }

    /// Configuration
    pub fn from_file(path: &str) -> crate::sdk::errors::Result<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            crate::sdk::errors::SDKError::ConfigError(format!(
                "Failed to read config file {}: {}",
                path, e
            ))
        })?;

        serde_yaml::from_str(&content).map_err(|e| {
            crate::sdk::errors::SDKError::ConfigError(format!(
                "Failed to parse config file {}: {}",
                path, e
            ))
        })
    }
}
