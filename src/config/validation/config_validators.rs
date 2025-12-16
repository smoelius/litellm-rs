//! Core configuration validators
//!
//! This module provides validation implementations for the main gateway configuration
//! structures including GatewayConfig, ServerConfig, and ProviderConfig.

use super::ssrf::validate_url_against_ssrf;
use super::trait_def::Validate;
use crate::config::models::*;
use std::collections::HashSet;
use tracing::debug;

impl Validate for GatewayConfig {
    fn validate(&self) -> Result<(), String> {
        debug!("Validating gateway configuration");

        self.server.validate()?;

        // Validate providers
        if self.providers.is_empty() {
            return Err("At least one provider must be configured".to_string());
        }

        // Check for duplicate provider names
        let mut provider_names = HashSet::new();
        for provider in &self.providers {
            if !provider_names.insert(&provider.name) {
                return Err(format!("Duplicate provider name: {}", provider.name));
            }
            provider.validate()?;
        }

        self.router.validate()?;
        self.storage.validate()?;
        self.auth.validate()?;
        self.monitoring.validate()?;
        self.cache.validate()?;
        self.rate_limit.validate()?;
        self.enterprise.validate()?;

        debug!("Gateway configuration validation completed");
        Ok(())
    }
}

impl Validate for ServerConfig {
    fn validate(&self) -> Result<(), String> {
        debug!("Validating server configuration");

        if self.host.is_empty() {
            return Err("Server host cannot be empty".to_string());
        }

        if self.port == 0 {
            return Err("Server port must be greater than 0".to_string());
        }

        if self.port < 1024 && !cfg!(test) {
            return Err("Server port should be >= 1024 for non-root users".to_string());
        }

        if let Some(workers) = self.workers {
            if workers == 0 {
                return Err("Worker count must be greater than 0".to_string());
            }
            if workers > 1000 {
                return Err("Worker count seems too high (>1000)".to_string());
            }
        }

        if self.timeout == 0 {
            return Err("Server timeout must be greater than 0".to_string());
        }

        if self.timeout > 3600 {
            return Err("Server timeout should not exceed 1 hour".to_string());
        }

        if self.max_body_size == 0 {
            return Err("Max body size must be greater than 0".to_string());
        }

        if self.max_body_size > 1024 * 1024 * 100 { // 100MB
            return Err("Max body size should not exceed 100MB".to_string());
        }

        // Validate TLS configuration if present
        if let Some(tls) = &self.tls {
            if tls.cert_file.is_empty() {
                return Err("TLS cert file path cannot be empty".to_string());
            }
            if tls.key_file.is_empty() {
                return Err("TLS key file path cannot be empty".to_string());
            }
        }

        Ok(())
    }
}

impl Validate for ProviderConfig {
    fn validate(&self) -> Result<(), String> {
        debug!("Validating provider configuration: {}", self.name);

        if self.name.is_empty() {
            return Err("Provider name cannot be empty".to_string());
        }

        if self.provider_type.is_empty() {
            return Err(format!("Provider {} type cannot be empty", self.name));
        }

        // Validate supported provider types
        let supported_types = [
            "openai", "anthropic", "azure", "google", "bedrock", "cohere",
            "huggingface", "ollama", "custom"
        ];
        if !supported_types.contains(&self.provider_type.as_str()) {
            return Err(format!(
                "Unsupported provider type: {}. Supported types: {:?}",
                self.provider_type, supported_types
            ));
        }

        if self.api_key.is_empty() {
            return Err(format!("Provider {} API key cannot be empty", self.name));
        }

        if self.weight <= 0.0 {
            return Err(format!("Provider {} weight must be greater than 0", self.name));
        }

        if self.weight > 100.0 {
            return Err(format!("Provider {} weight should not exceed 100", self.name));
        }

        if self.timeout == 0 {
            return Err(format!("Provider {} timeout must be greater than 0", self.name));
        }

        if self.timeout > 300 {
            return Err(format!("Provider {} timeout should not exceed 5 minutes", self.name));
        }

        // Validate base URL if present (with SSRF protection)
        if let Some(base_url) = &self.base_url {
            validate_url_against_ssrf(
                base_url,
                &format!("Provider {} base URL", self.name),
            )?;
        }

        // Validate rate limits
        if self.rpm == 0 {
            return Err(format!("Provider {} RPM must be greater than 0", self.name));
        }

        if self.tpm == 0 {
            return Err(format!("Provider {} TPM must be greater than 0", self.name));
        }

        if self.max_concurrent_requests == 0 {
            return Err(format!("Provider {} max concurrent requests must be greater than 0", self.name));
        }

        Ok(())
    }
}
