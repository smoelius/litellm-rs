//! Provider configuration builder implementation

use super::types::ProviderConfigBuilder;
use crate::config::ProviderConfig;
use crate::utils::data::type_utils::{NonEmptyString, PositiveF64};
use crate::utils::error::{GatewayError, Result};
use std::time::Duration;

impl ProviderConfigBuilder {
    /// Create a new provider configuration builder
    pub fn new() -> Self {
        Self {
            name: None,
            provider_type: None,
            api_key: None,
            base_url: None,
            models: Vec::new(),
            max_requests_per_minute: None,
            timeout: None,
            enabled: true,
            weight: None,
        }
    }

    /// Set the provider name
    pub fn name(mut self, name: impl TryInto<NonEmptyString>) -> Result<Self> {
        self.name = Some(
            name.try_into()
                .map_err(|_| GatewayError::Config("Provider name cannot be empty".to_string()))?,
        );
        Ok(self)
    }

    /// Set the provider type
    pub fn provider_type(mut self, provider_type: impl TryInto<NonEmptyString>) -> Result<Self> {
        self.provider_type = Some(
            provider_type
                .try_into()
                .map_err(|_| GatewayError::Config("Provider type cannot be empty".to_string()))?,
        );
        Ok(self)
    }

    /// Set the API key
    pub fn api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Set the base URL
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    /// Add a supported model
    pub fn add_model(mut self, model: impl Into<String>) -> Self {
        self.models.push(model.into());
        self
    }

    /// Set the rate limit
    pub fn rate_limit(mut self, requests_per_minute: u32) -> Self {
        self.max_requests_per_minute = Some(requests_per_minute);
        self
    }

    /// Set the timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Enable the provider
    pub fn enable(mut self) -> Self {
        self.enabled = true;
        self
    }

    /// Disable the provider
    pub fn disable(mut self) -> Self {
        self.enabled = false;
        self
    }

    /// Set the provider weight for load balancing
    pub fn weight(mut self, weight: f64) -> Result<Self> {
        self.weight =
            Some(PositiveF64::new(weight).map_err(|_| {
                GatewayError::Config("Provider weight must be positive".to_string())
            })?);
        Ok(self)
    }

    /// Build the provider configuration
    pub fn build(self) -> Result<ProviderConfig> {
        let name = self
            .name
            .ok_or_else(|| GatewayError::Config("Provider name is required".to_string()))?;

        let provider_type = self
            .provider_type
            .ok_or_else(|| GatewayError::Config("Provider type is required".to_string()))?;

        Ok(ProviderConfig {
            name: name.into_string(),
            provider_type: provider_type.into_string(),
            api_key: self.api_key.unwrap_or_default(),
            base_url: self.base_url,
            api_version: None,
            organization: None,
            project: None,
            weight: self.weight.map(|w| w.get() as f32).unwrap_or(1.0),
            rpm: self.max_requests_per_minute.unwrap_or(1000),
            tpm: 100000, // Default TPM
            max_concurrent_requests: 10,
            timeout: self.timeout.map(|d| d.as_secs()).unwrap_or(30),
            max_retries: 3,
            retry: crate::config::RetryConfig::default(),
            health_check: crate::config::HealthCheckConfig::default(),
            settings: std::collections::HashMap::new(),
            models: self.models,
            enabled: self.enabled,
            tags: Vec::new(),
        })
    }
}

impl Default for ProviderConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}
