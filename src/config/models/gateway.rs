//! Main gateway configuration

#![allow(missing_docs)]

use super::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main gateway configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GatewayConfig {
    /// Server configuration
    pub server: ServerConfig,
    /// Provider configurations
    pub providers: Vec<ProviderConfig>,
    /// Router configuration
    pub router: RouterConfig,
    /// Storage configuration
    pub storage: StorageConfig,
    /// Authentication configuration
    pub auth: AuthConfig,
    /// Monitoring configuration
    pub monitoring: MonitoringConfig,
    /// Caching configuration
    #[serde(default)]
    pub cache: CacheConfig,
    /// Rate limiting configuration
    #[serde(default)]
    pub rate_limit: RateLimitConfig,
    /// Enterprise features configuration
    #[serde(default)]
    pub enterprise: EnterpriseConfig,
}

#[allow(dead_code)]
impl GatewayConfig {
    pub fn from_env() -> crate::utils::error::Result<Self> {
        Ok(Self {
            server: ServerConfig::default(),
            providers: vec![],
            router: RouterConfig::default(),
            storage: StorageConfig::default(),
            auth: AuthConfig::default(),
            monitoring: MonitoringConfig::default(),
            cache: CacheConfig::default(),
            rate_limit: RateLimitConfig::default(),
            enterprise: EnterpriseConfig::default(),
        })
    }
}

#[allow(dead_code)]
impl GatewayConfig {
    /// Merge two configurations, with other taking precedence
    pub fn merge(mut self, other: Self) -> Self {
        self.server = self.server.merge(other.server);

        // Merge providers (other takes precedence for same names)
        let mut provider_map: HashMap<String, ProviderConfig> = self
            .providers
            .into_iter()
            .map(|p| (p.name.clone(), p))
            .collect();

        for provider in other.providers {
            provider_map.insert(provider.name.clone(), provider);
        }

        self.providers = provider_map.into_values().collect();
        self.router = self.router.merge(other.router);
        self.storage = self.storage.merge(other.storage);
        self.auth = self.auth.merge(other.auth);
        self.monitoring = self.monitoring.merge(other.monitoring);
        self.cache = self.cache.merge(other.cache);
        self.rate_limit = self.rate_limit.merge(other.rate_limit);
        self.enterprise = self.enterprise.merge(other.enterprise);

        self
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate server config
        if self.server.port == 0 {
            return Err("Server port cannot be 0".to_string());
        }

        // Validate providers
        if self.providers.is_empty() {
            return Err("At least one provider must be configured".to_string());
        }

        let mut provider_names = std::collections::HashSet::new();
        for provider in &self.providers {
            if provider.name.is_empty() {
                return Err("Provider name cannot be empty".to_string());
            }
            if !provider_names.insert(&provider.name) {
                return Err(format!("Duplicate provider name: {}", provider.name));
            }
            if provider.api_key.is_empty() {
                return Err(format!(
                    "API key is required for provider: {}",
                    provider.name
                ));
            }
        }

        // Validate storage config
        if self.storage.database.url.is_empty() {
            return Err("Database URL is required".to_string());
        }

        // Validate auth config
        if self.auth.jwt_secret.is_empty() {
            return Err("JWT secret is required".to_string());
        }

        Ok(())
    }

    /// Get provider by name
    pub fn get_provider(&self, name: &str) -> Option<&ProviderConfig> {
        self.providers.iter().find(|p| p.name == name)
    }

    /// Get providers by type
    pub fn get_providers_by_type(&self, provider_type: &str) -> Vec<&ProviderConfig> {
        self.providers
            .iter()
            .filter(|p| p.provider_type == provider_type)
            .collect()
    }

    /// Get providers by tag
    pub fn get_providers_by_tag(&self, tag: &str) -> Vec<&ProviderConfig> {
        self.providers
            .iter()
            .filter(|p| p.tags.contains(&tag.to_string()))
            .collect()
    }

    /// Check if a feature is enabled
    pub fn is_feature_enabled(&self, feature: &str) -> bool {
        match feature {
            "jwt_auth" => self.auth.enable_jwt,
            "api_key_auth" => self.auth.enable_api_key,
            "rbac" => self.auth.rbac.enabled,
            "metrics" => self.monitoring.metrics.enabled,
            "tracing" => self.monitoring.tracing.enabled,
            "health_checks" => true, // Always enabled
            "caching" => self.cache.enabled,
            "semantic_cache" => self.cache.semantic_cache,
            "rate_limiting" => self.rate_limit.enabled,
            "enterprise" => self.enterprise.enabled,
            "sso" => self.enterprise.sso.is_some(),
            "audit_logging" => self.enterprise.audit_logging,
            "advanced_analytics" => self.enterprise.advanced_analytics,
            _ => false,
        }
    }

    /// Get environment-specific configuration
    pub fn for_environment(&self, env: &str) -> Self {
        let mut config = self.clone();

        match env {
            "development" => {
                config.server.dev_mode = true;
                config.monitoring.tracing.enabled = true;
            }
            "production" => {
                config.server.dev_mode = false;
                config.monitoring.metrics.enabled = true;
                config.monitoring.tracing.enabled = true;
            }
            "testing" => {
                config.server.dev_mode = true;
                config.cache.enabled = false;
                config.rate_limit.enabled = false;
            }
            _ => {}
        }

        config
    }
}
