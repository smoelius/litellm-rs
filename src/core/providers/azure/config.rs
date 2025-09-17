//! Azure OpenAI Configuration
//!
//! Configuration for Azure OpenAI Service

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Azure OpenAI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureConfig {
    /// Azure API key
    pub api_key: Option<String>,
    /// Azure endpoint URL
    pub azure_endpoint: Option<String>,
    /// API version
    pub api_version: String,
    /// Azure AD token provider
    pub azure_ad_token_provider: Option<String>,
    /// Deployment name
    pub deployment_name: Option<String>,
    /// Resource group
    pub resource_group: Option<String>,
    /// Subscription ID
    pub subscription_id: Option<String>,
    /// Custom headers
    pub custom_headers: HashMap<String, String>,
}

impl Default for AzureConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            azure_endpoint: None,
            api_version: "2024-02-01".to_string(),
            azure_ad_token_provider: None,
            deployment_name: None,
            resource_group: None,
            subscription_id: None,
            custom_headers: HashMap::new(),
        }
    }
}

impl AzureConfig {
    /// Create new Azure configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set API key
    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }

    /// Set Azure endpoint
    pub fn with_azure_endpoint(mut self, endpoint: String) -> Self {
        self.azure_endpoint = Some(endpoint);
        self
    }

    /// Set API version
    pub fn with_api_version(mut self, version: String) -> Self {
        self.api_version = version;
        self
    }

    /// Set deployment name
    pub fn with_deployment_name(mut self, deployment: String) -> Self {
        self.deployment_name = Some(deployment);
        self
    }

    /// Get effective API key (from config, environment, or Azure AD)
    pub async fn get_effective_api_key(&self) -> Option<String> {
        // Priority: config -> environment -> Azure AD token
        if let Some(key) = &self.api_key {
            return Some(key.clone());
        }

        if let Ok(key) = std::env::var("AZURE_OPENAI_KEY") {
            return Some(key);
        }

        if let Ok(key) = std::env::var("AZURE_API_KEY") {
            return Some(key);
        }

        // Try Azure AD token (would need Azure AD integration)
        if self.azure_ad_token_provider.is_some() {
            // For now, return None - would implement Azure AD token acquisition
            return None;
        }

        None
    }

    /// Get effective Azure endpoint
    pub fn get_effective_azure_endpoint(&self) -> Option<String> {
        self.azure_endpoint
            .clone()
            .or_else(|| std::env::var("AZURE_OPENAI_ENDPOINT").ok())
            .or_else(|| std::env::var("AZURE_ENDPOINT").ok())
    }

    /// Get effective deployment name
    pub fn get_effective_deployment_name(&self, model: &str) -> String {
        self.deployment_name
            .clone()
            .or_else(|| std::env::var("AZURE_DEPLOYMENT_NAME").ok())
            .unwrap_or_else(|| model.to_string())
    }
}

/// Implement ProviderConfig trait for AzureConfig
impl crate::core::traits::ProviderConfig for AzureConfig {
    fn validate(&self) -> Result<(), String> {
        if self.get_effective_azure_endpoint().is_none() {
            return Err("Azure endpoint is required".to_string());
        }

        if self.api_version.is_empty() {
            return Err("API version is required".to_string());
        }

        Ok(())
    }

    fn api_key(&self) -> Option<&str> {
        self.api_key.as_deref()
    }

    fn api_base(&self) -> Option<&str> {
        self.azure_endpoint.as_deref()
    }

    fn timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(60) // Default 60 seconds timeout
    }

    fn max_retries(&self) -> u32 {
        3 // Default retry 3 times
    }
}

/// Azure model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureModelInfo {
    pub deployment_name: String,
    pub model_name: String,
    pub max_tokens: Option<u32>,
    pub supports_functions: bool,
    pub supports_streaming: bool,
    pub api_version: String,
}