//! Azure OpenAI Client
//!
//! HTTP client wrapper for Azure OpenAI Service

use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};

use super::config::AzureConfig;
use super::error::azure_config_error;
use super::utils::{AzureEndpointType, AzureUtils};
use crate::core::providers::unified_provider::ProviderError;

/// Azure OpenAI client
#[derive(Debug, Clone)]
pub struct AzureClient {
    config: AzureConfig,
    http_client: reqwest::Client,
}

impl AzureClient {
    /// Create new Azure client
    pub fn new(config: AzureConfig) -> Result<Self, ProviderError> {
        AzureUtils::validate_config(&config)?;

        let http_client = reqwest::Client::new();

        Ok(Self {
            config,
            http_client,
        })
    }

    /// Get configuration
    pub fn get_config(&self) -> &AzureConfig {
        &self.config
    }

    /// Build request URL
    pub fn build_url(
        &self,
        deployment_name: &str,
        endpoint_type: AzureEndpointType,
    ) -> Result<String, ProviderError> {
        let endpoint = self
            .config
            .get_effective_azure_endpoint()
            .ok_or_else(|| azure_config_error("Azure endpoint not configured"))?;

        Ok(AzureUtils::build_azure_url(
            &endpoint,
            deployment_name,
            &self.config.api_version,
            endpoint_type,
        ))
    }

    /// Get HTTP client
    pub fn get_http_client(&self) -> &reqwest::Client {
        &self.http_client
    }
}

/// Default Azure configuration factory
pub struct AzureConfigFactory;

impl AzureConfigFactory {
    /// Create configuration from environment variables
    pub fn from_environment() -> AzureConfig {
        let mut config = AzureConfig::new();

        if let Ok(api_key) = std::env::var("AZURE_OPENAI_KEY") {
            config.api_key = Some(api_key);
        } else if let Ok(api_key) = std::env::var("AZURE_API_KEY") {
            config.api_key = Some(api_key);
        }

        if let Ok(endpoint) = std::env::var("AZURE_OPENAI_ENDPOINT") {
            config.azure_endpoint = Some(endpoint);
        } else if let Ok(endpoint) = std::env::var("AZURE_ENDPOINT") {
            config.azure_endpoint = Some(endpoint);
        }

        if let Ok(version) = std::env::var("AZURE_API_VERSION") {
            config.api_version = version;
        }

        if let Ok(deployment) = std::env::var("AZURE_DEPLOYMENT_NAME") {
            config.deployment_name = Some(deployment);
        }

        config
    }

    /// Create configuration for specific Azure service
    pub fn for_service(service: &str, _region: &str) -> AzureConfig {
        AzureConfig::new()
            .with_azure_endpoint(format!("https://{}.openai.azure.com", service))
            .with_api_version("2024-02-01".to_string())
    }
}

/// Azure rate limiting information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureRateLimitInfo {
    pub requests_limit: Option<u32>,
    pub requests_remaining: Option<u32>,
    pub requests_reset: Option<u64>,
    pub tokens_limit: Option<u32>,
    pub tokens_remaining: Option<u32>,
    pub tokens_reset: Option<u64>,
}

impl AzureRateLimitInfo {
    /// Extract rate limit info from headers
    pub fn from_headers(headers: &HeaderMap) -> Self {
        Self {
            requests_limit: headers
                .get("x-ratelimit-limit-requests")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse().ok()),
            requests_remaining: headers
                .get("x-ratelimit-remaining-requests")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse().ok()),
            requests_reset: headers
                .get("x-ratelimit-reset-requests")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse().ok()),
            tokens_limit: headers
                .get("x-ratelimit-limit-tokens")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse().ok()),
            tokens_remaining: headers
                .get("x-ratelimit-remaining-tokens")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse().ok()),
            tokens_reset: headers
                .get("x-ratelimit-reset-tokens")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse().ok()),
        }
    }
}
