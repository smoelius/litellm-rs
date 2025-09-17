//! Configuration validation
//!
//! This module provides validation logic for all configuration structures.

use super::models::*;
use crate::utils::error::{GatewayError, Result};
use std::collections::HashSet;
use tracing::debug;

/// Validation trait for configuration structures
pub trait Validate {
    fn validate(&self) -> Result<()>;
}

impl Validate for GatewayConfig {
    fn validate(&self) -> Result<()> {
        debug!("Validating gateway configuration");
        
        self.server.validate()?;
        
        // Validate providers
        if self.providers.is_empty() {
            return Err(GatewayError::Config("At least one provider must be configured".to_string()));
        }
        
        // Check for duplicate provider names
        let mut provider_names = HashSet::new();
        for provider in &self.providers {
            if !provider_names.insert(&provider.name) {
                return Err(GatewayError::Config(format!("Duplicate provider name: {}", provider.name)));
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
    fn validate(&self) -> Result<()> {
        debug!("Validating server configuration");
        
        if self.host.is_empty() {
            return Err(GatewayError::Config("Server host cannot be empty".to_string()));
        }
        
        if self.port == 0 {
            return Err(GatewayError::Config("Server port must be greater than 0".to_string()));
        }
        
        if self.port < 1024 && !cfg!(test) {
            return Err(GatewayError::Config("Server port should be >= 1024 for non-root users".to_string()));
        }
        
        if let Some(workers) = self.workers {
            if workers == 0 {
                return Err(GatewayError::Config("Worker count must be greater than 0".to_string()));
            }
            if workers > 1000 {
                return Err(GatewayError::Config("Worker count seems too high (>1000)".to_string()));
            }
        }
        
        if self.timeout == 0 {
            return Err(GatewayError::Config("Server timeout must be greater than 0".to_string()));
        }
        
        if self.timeout > 3600 {
            return Err(GatewayError::Config("Server timeout should not exceed 1 hour".to_string()));
        }
        
        if self.max_body_size == 0 {
            return Err(GatewayError::Config("Max body size must be greater than 0".to_string()));
        }
        
        if self.max_body_size > 1024 * 1024 * 100 { // 100MB
            return Err(GatewayError::Config("Max body size should not exceed 100MB".to_string()));
        }
        
        // Validate TLS configuration if present
        if let Some(tls) = &self.tls {
            if tls.cert_file.is_empty() {
                return Err(GatewayError::Config("TLS cert file path cannot be empty".to_string()));
            }
            if tls.key_file.is_empty() {
                return Err(GatewayError::Config("TLS key file path cannot be empty".to_string()));
            }
        }
        
        Ok(())
    }
}

impl Validate for ProviderConfig {
    fn validate(&self) -> Result<()> {
        debug!("Validating provider configuration: {}", self.name);
        
        if self.name.is_empty() {
            return Err(GatewayError::Config("Provider name cannot be empty".to_string()));
        }
        
        if self.provider_type.is_empty() {
            return Err(GatewayError::Config(format!("Provider {} type cannot be empty", self.name)));
        }
        
        // Validate supported provider types
        let supported_types = [
            "openai", "anthropic", "azure", "google", "bedrock", "cohere",
            "huggingface", "ollama", "custom"
        ];
        if !supported_types.contains(&self.provider_type.as_str()) {
            return Err(GatewayError::Config(format!(
                "Unsupported provider type: {}. Supported types: {:?}",
                self.provider_type, supported_types
            )));
        }
        
        if self.api_key.is_empty() {
            return Err(GatewayError::Config(format!("Provider {} API key cannot be empty", self.name)));
        }
        
        if self.weight <= 0.0 {
            return Err(GatewayError::Config(format!("Provider {} weight must be greater than 0", self.name)));
        }
        
        if self.weight > 100.0 {
            return Err(GatewayError::Config(format!("Provider {} weight should not exceed 100", self.name)));
        }
        
        if let Some(timeout) = self.timeout {
            if timeout == 0 {
                return Err(GatewayError::Config(format!("Provider {} timeout must be greater than 0", self.name)));
            }
            if timeout > 300 {
                return Err(GatewayError::Config(format!("Provider {} timeout should not exceed 5 minutes", self.name)));
            }
        }
        
        // Validate API base URL if present
        if let Some(api_base) = &self.api_base {
            if !api_base.starts_with("http://") && !api_base.starts_with("https://") {
                return Err(GatewayError::Config(format!(
                    "Provider {} API base must be a valid HTTP/HTTPS URL", self.name
                )));
            }
        }
        
        // Validate rate limits if present
        if let Some(rate_limits) = &self.rate_limits {
            if let Some(rpm) = rate_limits.rpm {
                if rpm == 0 {
                    return Err(GatewayError::Config(format!("Provider {} RPM must be greater than 0", self.name)));
                }
            }
            if let Some(tpm) = rate_limits.tpm {
                if tpm == 0 {
                    return Err(GatewayError::Config(format!("Provider {} TPM must be greater than 0", self.name)));
                }
            }
            if let Some(concurrent) = rate_limits.concurrent {
                if concurrent == 0 {
                    return Err(GatewayError::Config(format!("Provider {} concurrent limit must be greater than 0", self.name)));
                }
            }
        }
        
        Ok(())
    }
}

impl Validate for RouterConfig {
    fn validate(&self) -> Result<()> {
        debug!("Validating router configuration");
        
        if self.health_check_interval == 0 {
            return Err(GatewayError::Config("Health check interval must be greater than 0".to_string()));
        }
        
        if self.health_check_interval > 3600 {
            return Err(GatewayError::Config("Health check interval should not exceed 1 hour".to_string()));
        }
        
        self.circuit_breaker.validate()?;
        self.retry.validate()?;
        
        Ok(())
    }
}

impl Validate for CircuitBreakerConfig {
    fn validate(&self) -> Result<()> {
        if self.failure_threshold == 0 {
            return Err(GatewayError::Config("Circuit breaker failure threshold must be greater than 0".to_string()));
        }
        
        if self.recovery_timeout == 0 {
            return Err(GatewayError::Config("Circuit breaker recovery timeout must be greater than 0".to_string()));
        }
        
        if self.min_requests == 0 {
            return Err(GatewayError::Config("Circuit breaker min requests must be greater than 0".to_string()));
        }
        
        Ok(())
    }
}

impl Validate for RetryConfig {
    fn validate(&self) -> Result<()> {
        if self.max_attempts == 0 {
            return Err(GatewayError::Config("Retry max attempts must be greater than 0".to_string()));
        }
        
        if self.max_attempts > 10 {
            return Err(GatewayError::Config("Retry max attempts should not exceed 10".to_string()));
        }
        
        if self.base_delay == 0 {
            return Err(GatewayError::Config("Retry base delay must be greater than 0".to_string()));
        }
        
        if self.max_delay <= self.base_delay {
            return Err(GatewayError::Config("Retry max delay must be greater than base delay".to_string()));
        }
        
        if self.backoff_multiplier <= 1.0 {
            return Err(GatewayError::Config("Retry backoff multiplier must be greater than 1.0".to_string()));
        }
        
        Ok(())
    }
}

impl Validate for StorageConfig {
    fn validate(&self) -> Result<()> {
        debug!("Validating storage configuration");
        
        self.database.validate()?;
        self.redis.validate()?;
        self.files.validate()?;
        
        if let Some(vector_db) = &self.vector_db {
            vector_db.validate()?;
        }
        
        Ok(())
    }
}

impl Validate for DatabaseConfig {
    fn validate(&self) -> Result<()> {
        if self.url.is_empty() {
            return Err(GatewayError::Config("Database URL cannot be empty".to_string()));
        }
        
        if !self.url.starts_with("postgresql://") && !self.url.starts_with("postgres://") {
            return Err(GatewayError::Config("Only PostgreSQL databases are supported".to_string()));
        }
        
        if self.max_connections == 0 {
            return Err(GatewayError::Config("Database max connections must be greater than 0".to_string()));
        }
        
        if self.max_connections > 1000 {
            return Err(GatewayError::Config("Database max connections should not exceed 1000".to_string()));
        }
        
        if self.connection_timeout == 0 {
            return Err(GatewayError::Config("Database connection timeout must be greater than 0".to_string()));
        }
        
        Ok(())
    }
}

impl Validate for RedisConfig {
    fn validate(&self) -> Result<()> {
        if self.url.is_empty() {
            return Err(GatewayError::Config("Redis URL cannot be empty".to_string()));
        }
        
        if !self.url.starts_with("redis://") && !self.url.starts_with("rediss://") {
            return Err(GatewayError::Config("Redis URL must start with redis:// or rediss://".to_string()));
        }
        
        if self.max_connections == 0 {
            return Err(GatewayError::Config("Redis max connections must be greater than 0".to_string()));
        }
        
        if self.connection_timeout == 0 {
            return Err(GatewayError::Config("Redis connection timeout must be greater than 0".to_string()));
        }

        Ok(())
    }
}

impl Validate for FileStorageConfig {
    fn validate(&self) -> Result<()> {
        let supported_types = ["local", "s3", "gcs", "azure"];
        if !supported_types.contains(&self.storage_type.as_str()) {
            return Err(GatewayError::Config(format!(
                "Unsupported file storage type: {}. Supported types: {:?}",
                self.storage_type, supported_types
            )));
        }

        match self.storage_type.as_str() {
            "local" => {
                if self.local_path.is_none() {
                    return Err(GatewayError::Config("Local storage path must be specified for local storage".to_string()));
                }
            }
            "s3" => {
                if self.s3.is_none() {
                    return Err(GatewayError::Config("S3 configuration must be specified for S3 storage".to_string()));
                }
                if let Some(s3) = &self.s3 {
                    s3.validate()?;
                }
            }
            _ => {}
        }

        Ok(())
    }
}

impl Validate for S3Config {
    fn validate(&self) -> Result<()> {
        if self.bucket.is_empty() {
            return Err(GatewayError::Config("S3 bucket name cannot be empty".to_string()));
        }

        if self.region.is_empty() {
            return Err(GatewayError::Config("S3 region cannot be empty".to_string()));
        }

        Ok(())
    }
}

impl Validate for VectorDbConfig {
    fn validate(&self) -> Result<()> {
        let supported_types = ["qdrant", "weaviate", "pinecone"];
        if !supported_types.contains(&self.db_type.as_str()) {
            return Err(GatewayError::Config(format!(
                "Unsupported vector DB type: {}. Supported types: {:?}",
                self.db_type, supported_types
            )));
        }

        if self.url.is_empty() {
            return Err(GatewayError::Config("Vector DB URL cannot be empty".to_string()));
        }

        if self.collection.is_empty() {
            return Err(GatewayError::Config("Vector DB collection name cannot be empty".to_string()));
        }

        Ok(())
    }
}

impl Validate for AuthConfig {
    fn validate(&self) -> Result<()> {
        debug!("Validating auth configuration");

        if self.jwt_secret.is_empty() {
            return Err(GatewayError::Config("JWT secret cannot be empty".to_string()));
        }

        if self.jwt_secret == "change-me-in-production" && !cfg!(test) {
            return Err(GatewayError::Config("JWT secret must be changed from default value in production".to_string()));
        }

        if self.jwt_secret.len() < 32 {
            return Err(GatewayError::Config("JWT secret should be at least 32 characters long".to_string()));
        }

        if self.jwt_expiration == 0 {
            return Err(GatewayError::Config("JWT expiration must be greater than 0".to_string()));
        }

        if self.jwt_expiration > 86400 * 30 { // 30 days
            return Err(GatewayError::Config("JWT expiration should not exceed 30 days".to_string()));
        }

        if self.api_key_header.is_empty() {
            return Err(GatewayError::Config("API key header cannot be empty".to_string()));
        }

        self.rbac.validate()?;

        Ok(())
    }
}

impl Validate for RbacConfig {
    fn validate(&self) -> Result<()> {
        if self.enabled && self.default_role.is_empty() {
            return Err(GatewayError::Config("Default role cannot be empty when RBAC is enabled".to_string()));
        }

        Ok(())
    }
}

impl Validate for MonitoringConfig {
    fn validate(&self) -> Result<()> {
        debug!("Validating monitoring configuration");

        self.metrics.validate()?;
        self.tracing.validate()?;
        self.health.validate()?;
        self.alerting.validate()?;

        Ok(())
    }
}

impl Validate for MetricsConfig {
    fn validate(&self) -> Result<()> {
        if self.enabled && self.port == 0 {
            return Err(GatewayError::Config("Metrics port must be greater than 0 when metrics are enabled".to_string()));
        }

        if self.path.is_empty() {
            return Err(GatewayError::Config("Metrics path cannot be empty".to_string()));
        }

        if !self.path.starts_with('/') {
            return Err(GatewayError::Config("Metrics path must start with '/'".to_string()));
        }

        Ok(())
    }
}

impl Validate for TracingConfig {
    fn validate(&self) -> Result<()> {
        if self.enabled && self.jaeger_endpoint.is_none() {
            return Err(GatewayError::Config("Jaeger endpoint must be specified when tracing is enabled".to_string()));
        }

        if self.service_name.is_empty() {
            return Err(GatewayError::Config("Service name cannot be empty".to_string()));
        }

        Ok(())
    }
}

impl Validate for HealthConfig {
    fn validate(&self) -> Result<()> {
        if self.path.is_empty() {
            return Err(GatewayError::Config("Health check path cannot be empty".to_string()));
        }

        if !self.path.starts_with('/') {
            return Err(GatewayError::Config("Health check path must start with '/'".to_string()));
        }

        Ok(())
    }
}

impl Validate for AlertingConfig {
    fn validate(&self) -> Result<()> {
        if self.enabled {
            if self.slack_webhook.is_none() && self.email.is_none() {
                return Err(GatewayError::Config("At least one alerting method must be configured when alerting is enabled".to_string()));
            }
        }

        if let Some(email) = &self.email {
            email.validate()?;
        }

        Ok(())
    }
}

impl Validate for EmailConfig {
    fn validate(&self) -> Result<()> {
        if self.smtp_server.is_empty() {
            return Err(GatewayError::Config("SMTP server cannot be empty".to_string()));
        }

        if self.smtp_port == 0 {
            return Err(GatewayError::Config("SMTP port must be greater than 0".to_string()));
        }

        if self.username.is_empty() {
            return Err(GatewayError::Config("SMTP username cannot be empty".to_string()));
        }

        if self.password.is_empty() {
            return Err(GatewayError::Config("SMTP password cannot be empty".to_string()));
        }

        if self.from.is_empty() {
            return Err(GatewayError::Config("Email from address cannot be empty".to_string()));
        }

        if self.to.is_empty() {
            return Err(GatewayError::Config("At least one email recipient must be specified".to_string()));
        }

        Ok(())
    }
}

impl Validate for CacheConfig {
    fn validate(&self) -> Result<()> {
        if self.ttl == 0 {
            return Err(GatewayError::Config("Cache TTL must be greater than 0".to_string()));
        }

        if self.max_size == 0 {
            return Err(GatewayError::Config("Cache max size must be greater than 0".to_string()));
        }

        if self.semantic_cache && (self.similarity_threshold <= 0.0 || self.similarity_threshold > 1.0) {
            return Err(GatewayError::Config("Semantic cache similarity threshold must be between 0 and 1".to_string()));
        }

        Ok(())
    }
}

impl Validate for RateLimitConfig {
    fn validate(&self) -> Result<()> {
        if self.default_rpm == 0 {
            return Err(GatewayError::Config("Default RPM must be greater than 0".to_string()));
        }

        if self.default_tpm == 0 {
            return Err(GatewayError::Config("Default TPM must be greater than 0".to_string()));
        }

        Ok(())
    }
}

impl Validate for EnterpriseConfig {
    fn validate(&self) -> Result<()> {
        if let Some(sso) = &self.sso {
            sso.validate()?;
        }

        Ok(())
    }
}

impl Validate for SsoConfig {
    fn validate(&self) -> Result<()> {
        let supported_providers = ["saml", "oidc", "oauth2"];
        if !supported_providers.contains(&self.provider.as_str()) {
            return Err(GatewayError::Config(format!(
                "Unsupported SSO provider: {}. Supported providers: {:?}",
                self.provider, supported_providers
            )));
        }

        if self.config.is_empty() {
            return Err(GatewayError::Config("SSO configuration cannot be empty".to_string()));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config_validation() {
        let mut config = ServerConfig::default();
        assert!(config.validate().is_ok());

        config.port = 0;
        assert!(config.validate().is_err());

        config.port = 8080;
        config.host = "".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_provider_config_validation() {
        let mut config = ProviderConfig {
            name: "test".to_string(),
            provider_type: "openai".to_string(),
            api_key: "test-key".to_string(),
            api_base: None,
            api_version: None,
            timeout: None,
            max_retries: 3,
            weight: 1.0,
            tags: vec![],
            headers: std::collections::HashMap::new(),
            rate_limits: None,
            cost: None,
        };

        assert!(config.validate().is_ok());

        config.provider_type = "unsupported".to_string();
        assert!(config.validate().is_err());

        config.provider_type = "openai".to_string();
        config.weight = 0.0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_auth_config_validation() {
        let mut config = AuthConfig::default();
        config.jwt_secret = "a-very-long-secret-key-for-testing-purposes".to_string();
        assert!(config.validate().is_ok());

        config.jwt_secret = "short".to_string();
        assert!(config.validate().is_err());

        config.jwt_secret = "".to_string();
        assert!(config.validate().is_err());
    }
}
