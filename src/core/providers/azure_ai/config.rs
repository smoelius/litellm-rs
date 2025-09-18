//! Azure AI Configuration
//!
//! Configuration

// use serde::{Deserialize, Serialize};  // Not needed with the macro
use std::collections::HashMap;

use crate::core::traits::ProviderConfig;
use crate::define_provider_config;

// Configuration
define_provider_config!(AzureAIConfig {});

impl AzureAIConfig {
    /// Create
    pub fn from_env() -> Self {
        let mut config = Self::new("azure_ai");

        // Default
        if config.base.api_base.is_none() {
            if let Ok(api_base) = std::env::var("AZURE_AI_API_BASE") {
                config.base.api_base = Some(api_base);
            } else if let Ok(endpoint) = std::env::var("AZURE_AI_ENDPOINT") {
                config.base.api_base = Some(endpoint);
            }
        }

        // Settings
        if config.base.api_key.is_none() {
            if let Ok(api_key) = std::env::var("AZURE_AI_API_KEY") {
                config.base.api_key = Some(api_key);
            } else if let Ok(api_key) = std::env::var("AZURE_API_KEY") {
                config.base.api_key = Some(api_key);
            }
        }

        config
    }

    /// Build
    pub fn build_endpoint_url(&self, path: &str) -> Result<String, String> {
        let base_url = self
            .base
            .api_base
            .as_ref()
            .ok_or("Azure AI API base URL not set")?;

        // Ensure base URL ends with '/' and path doesn't start with '/'
        let base = base_url.trim_end_matches('/');
        let endpoint_path = path.trim_start_matches('/');

        Ok(format!("{}/{}", base, endpoint_path))
    }

    /// Default
    pub fn create_default_headers(&self) -> Result<HashMap<String, String>, String> {
        let mut headers = HashMap::new();

        // Settings
        if let Some(api_key) = &self.base.api_key {
            headers.insert("Authorization".to_string(), format!("Bearer {}", api_key));
        } else {
            return Err("Azure AI API key not set".to_string());
        }

        // Settings
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        // Settings
        headers.insert("User-Agent".to_string(), "litellm-rust/0.1.0".to_string());

        // Settings
        headers.insert("api-version".to_string(), "2024-05-01-preview".to_string());

        Ok(headers)
    }

    /// Configuration
    pub fn timeout(&self) -> std::time::Duration {
        self.base.timeout_duration()
    }

    /// Configuration
    pub fn validate(&self) -> Result<(), String> {
        self.base.validate("azure_ai")
    }
}

// Implementation of ProviderConfig trait
impl ProviderConfig for AzureAIConfig {
    fn validate(&self) -> Result<(), String> {
        self.base.validate("azure_ai")
    }

    fn api_key(&self) -> Option<&str> {
        self.base.api_key.as_deref()
    }

    fn api_base(&self) -> Option<&str> {
        self.base.api_base.as_deref()
    }

    fn timeout(&self) -> std::time::Duration {
        self.base.timeout_duration()
    }

    fn max_retries(&self) -> u32 {
        self.base.max_retries
    }
}

/// Azure AI endpoint type enumeration
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AzureAIEndpointType {
    /// Chat completions endpoint
    ChatCompletions,
    /// Embeddings endpoint
    Embeddings,
    /// Image embeddings endpoint (multimodal)
    ImageEmbeddings,
    /// Image generation endpoint
    ImageGeneration,
    /// Rerank endpoint
    Rerank,
}

impl AzureAIEndpointType {
    /// Get
    pub fn as_path(&self) -> &'static str {
        match self {
            AzureAIEndpointType::ChatCompletions => "chat/completions",
            AzureAIEndpointType::Embeddings => "embeddings",
            AzureAIEndpointType::ImageEmbeddings => "images/embeddings",
            AzureAIEndpointType::ImageGeneration => "images/generations",
            AzureAIEndpointType::Rerank => "rerank",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_azure_ai_config() {
        let config = AzureAIConfig::new("azure_ai");
        assert_eq!(config.base.max_retries, 3);
        assert_eq!(config.base.timeout, 60);
    }

    #[test]
    fn test_endpoint_types() {
        assert_eq!(
            AzureAIEndpointType::ChatCompletions.as_path(),
            "chat/completions"
        );
        assert_eq!(AzureAIEndpointType::Embeddings.as_path(), "embeddings");
        assert_eq!(
            AzureAIEndpointType::ImageGeneration.as_path(),
            "images/generations"
        );
        assert_eq!(AzureAIEndpointType::Rerank.as_path(), "rerank");
    }

    #[test]
    fn test_build_endpoint_url() {
        let mut config = AzureAIConfig::new("azure_ai");
        config.base.api_base = Some("https://test.ai.azure.com".to_string());

        let url = config.build_endpoint_url("chat/completions").unwrap();
        assert_eq!(url, "https://test.ai.azure.com/chat/completions");

        // Test with trailing slash
        config.base.api_base = Some("https://test.ai.azure.com/".to_string());
        let url = config.build_endpoint_url("/chat/completions").unwrap();
        assert_eq!(url, "https://test.ai.azure.com/chat/completions");
    }
}
