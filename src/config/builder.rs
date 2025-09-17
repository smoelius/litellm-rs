//! Configuration builder for type-safe configuration construction
//!
//! This module provides a builder pattern for creating configurations
//! with compile-time validation and better ergonomics.

#![allow(dead_code)] // Builder module - functions may be used in the future

use super::{AuthConfig, Config, GatewayConfig, ProviderConfig, ServerConfig, StorageConfig};
use crate::utils::data::type_utils::{Builder, NonEmptyString, PositiveF64};
use crate::utils::error::{GatewayError, Result};
use std::collections::HashMap;
use std::time::Duration;

/// Builder for creating gateway configurations
#[derive(Debug, Clone)]
pub struct ConfigBuilder {
    server: Option<ServerConfig>,
    auth: Option<AuthConfig>,
    storage: Option<StorageConfig>,
    providers: Vec<ProviderConfig>,
    features: HashMap<String, bool>,
}

impl ConfigBuilder {
    /// Create a new configuration builder
    pub fn new() -> Self {
        Self {
            server: None,
            auth: None,
            storage: None,
            providers: Vec::new(),
            features: HashMap::new(),
        }
    }

    /// Set the server configuration
    pub fn with_server(mut self, config: ServerConfig) -> Self {
        self.server = Some(config);
        self
    }

    /// Set the authentication configuration
    pub fn with_auth(mut self, config: AuthConfig) -> Self {
        self.auth = Some(config);
        self
    }

    /// Set the storage configuration
    pub fn with_storage(mut self, config: StorageConfig) -> Self {
        self.storage = Some(config);
        self
    }

    /// Add a provider configuration
    pub fn add_provider(mut self, config: ProviderConfig) -> Self {
        self.providers.push(config);
        self
    }

    /// Add multiple provider configurations
    pub fn add_providers(mut self, configs: Vec<ProviderConfig>) -> Self {
        self.providers.extend(configs);
        self
    }

    /// Enable a feature
    pub fn enable_feature(mut self, feature: impl Into<String>) -> Self {
        self.features.insert(feature.into(), true);
        self
    }

    /// Disable a feature
    pub fn disable_feature(mut self, feature: impl Into<String>) -> Self {
        self.features.insert(feature.into(), false);
        self
    }

    /// Build the configuration with validation
    pub fn build(self) -> Result<Config> {
        let gateway = GatewayConfig {
            server: self.server.unwrap_or_default(),
            auth: self.auth.unwrap_or_default(),
            storage: self.storage.unwrap_or_default(),
            providers: self.providers,
            router: super::RouterConfig::default(),
            monitoring: super::MonitoringConfig::default(),
            cache: super::CacheConfig::default(),
            rate_limit: super::RateLimitConfig::default(),
            enterprise: super::EnterpriseConfig::default(),
        };

        let config = Config { gateway };

        // Validate the configuration
        if let Err(e) = config.gateway.validate() {
            return Err(GatewayError::Config(e));
        }

        Ok(config)
    }

    /// Build the configuration or panic with a descriptive message
    pub fn build_or_panic(self) -> Config {
        self.build().unwrap_or_else(|e| {
            panic!("Failed to build configuration: {}", e);
        })
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Builder<Config> for ConfigBuilder {
    fn build(self) -> Config {
        self.build().expect("Configuration validation failed")
    }
}

/// Builder for server configuration
#[derive(Debug, Clone)]
pub struct ServerConfigBuilder {
    host: Option<String>,
    port: Option<u16>,
    workers: Option<usize>,
    timeout: Option<Duration>,
    max_connections: Option<usize>,
    enable_cors: bool,
    cors_origins: Vec<String>,
}

impl ServerConfigBuilder {
    /// Create a new server configuration builder
    pub fn new() -> Self {
        Self {
            host: None,
            port: None,
            workers: None,
            timeout: None,
            max_connections: None,
            enable_cors: false,
            cors_origins: Vec::new(),
        }
    }

    /// Set the host
    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.host = Some(host.into());
        self
    }

    /// Set the port
    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    /// Set the number of workers
    pub fn workers(mut self, workers: usize) -> Self {
        self.workers = Some(workers);
        self
    }

    /// Set the request timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set the maximum number of connections
    pub fn max_connections(mut self, max_connections: usize) -> Self {
        self.max_connections = Some(max_connections);
        self
    }

    /// Enable CORS
    pub fn enable_cors(mut self) -> Self {
        self.enable_cors = true;
        self
    }

    /// Add CORS origin
    pub fn add_cors_origin(mut self, origin: impl Into<String>) -> Self {
        self.cors_origins.push(origin.into());
        self
    }

    /// Build the server configuration
    pub fn build(self) -> ServerConfig {
        ServerConfig {
            host: self.host.unwrap_or_else(|| "127.0.0.1".to_string()),
            port: self.port.unwrap_or(8080),
            workers: self.workers,
            timeout: self.timeout.map(|d| d.as_secs()).unwrap_or(30),
            max_body_size: 1024 * 1024, // 1MB default
            dev_mode: false,
            tls: None,
            cors: super::CorsConfig {
                enabled: self.enable_cors,
                allowed_origins: if self.cors_origins.is_empty() {
                    vec!["*".to_string()]
                } else {
                    self.cors_origins
                },
                allowed_methods: vec!["GET".to_string(), "POST".to_string(), "OPTIONS".to_string()],
                allowed_headers: vec!["Content-Type".to_string(), "Authorization".to_string()],
                max_age: 3600,
                allow_credentials: false,
            },
        }
    }
}

impl Default for ServerConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Builder<ServerConfig> for ServerConfigBuilder {
    fn build(self) -> ServerConfig {
        self.build()
    }
}

/// Builder for provider configuration
#[derive(Debug, Clone)]
pub struct ProviderConfigBuilder {
    name: Option<NonEmptyString>,
    provider_type: Option<NonEmptyString>,
    api_key: Option<String>,
    base_url: Option<String>,
    models: Vec<String>,
    max_requests_per_minute: Option<u32>,
    timeout: Option<Duration>,
    enabled: bool,
    weight: Option<PositiveF64>,
}

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
            retry: super::RetryConfig::default(),
            health_check: super::HealthCheckConfig::default(),
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

/// Convenience functions for common configurations
pub mod presets {
    use super::*;

    /// Create a development server configuration
    pub fn dev_server() -> ServerConfigBuilder {
        ServerConfigBuilder::new()
            .host("127.0.0.1")
            .port(8080)
            .workers(1)
            .enable_cors()
            .add_cors_origin("*")
    }

    /// Create a production server configuration
    pub fn prod_server() -> ServerConfigBuilder {
        ServerConfigBuilder::new()
            .host("0.0.0.0")
            .port(8080)
            .workers(num_cpus::get())
            .max_connections(10000)
            .timeout(Duration::from_secs(60))
    }

    /// Create an OpenAI provider configuration
    pub fn openai_provider(name: &str, api_key: &str) -> Result<ProviderConfigBuilder> {
        Ok(ProviderConfigBuilder::new()
            .name(name)?
            .provider_type("openai")?
            .api_key(api_key)
            .add_model("gpt-3.5-turbo")
            .add_model("gpt-4")
            .rate_limit(3000))
    }

    /// Create an Anthropic provider configuration
    pub fn anthropic_provider(name: &str, api_key: &str) -> Result<ProviderConfigBuilder> {
        Ok(ProviderConfigBuilder::new()
            .name(name)?
            .provider_type("anthropic")?
            .api_key(api_key)
            .add_model("claude-3-sonnet")
            .add_model("claude-3-haiku")
            .rate_limit(1000))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder() {
        let config = ConfigBuilder::new()
            .with_server(presets::dev_server().build())
            .add_provider(
                presets::openai_provider("openai", "test-key")
                    .unwrap()
                    .build()
                    .unwrap(),
            )
            .enable_feature("metrics")
            .build();

        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.gateway.server.port, 8080);
        assert_eq!(config.gateway.providers.len(), 1);
    }

    #[test]
    fn test_provider_builder() {
        let provider = ProviderConfigBuilder::new()
            .name("test-provider")
            .unwrap()
            .provider_type("openai")
            .unwrap()
            .api_key("test-key")
            .add_model("gpt-4")
            .weight(2.0)
            .unwrap()
            .build();

        assert!(provider.is_ok());
        let provider = provider.unwrap();
        assert_eq!(provider.name, "test-provider");
        assert_eq!(provider.weight, 2.0);
    }
}
