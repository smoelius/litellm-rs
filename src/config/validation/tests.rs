//! Tests for configuration validation
//!
//! This module contains all tests for the validation logic.

#[cfg(test)]
mod tests {
    use super::super::ssrf::validate_url_against_ssrf;
    use super::super::trait_def::Validate;
    use crate::config::models::*;

    #[test]
    fn test_server_config_validation() {
        let mut config = ServerConfig::default();
        // Use Validate::validate to call the trait method (not the inherent method)
        assert!(Validate::validate(&config).is_ok());

        config.port = 0;
        assert!(Validate::validate(&config).is_err());

        config.port = 8080;
        config.host = "".to_string();
        assert!(Validate::validate(&config).is_err());
    }

    #[test]
    fn test_provider_config_validation() {
        let mut config = ProviderConfig {
            name: "test".to_string(),
            provider_type: "openai".to_string(),
            api_key: "test-key".to_string(),
            ..Default::default()
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
        let mut config = AuthConfig {
            jwt_secret: "a-very-long-secret-key-for-testing-purposes".to_string(),
            ..Default::default()
        };
        assert!(config.validate().is_ok());

        config.jwt_secret = "short".to_string();
        assert!(config.validate().is_err());

        config.jwt_secret = "".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_ssrf_validation_valid_urls() {
        // Valid public URLs should pass
        assert!(validate_url_against_ssrf("https://api.openai.com/v1", "test").is_ok());
        assert!(validate_url_against_ssrf("https://api.anthropic.com", "test").is_ok());
        assert!(validate_url_against_ssrf("http://example.com:8080/api", "test").is_ok());
    }

    #[test]
    fn test_ssrf_validation_localhost() {
        // Localhost should be blocked
        assert!(validate_url_against_ssrf("http://localhost/api", "test").is_err());
        assert!(validate_url_against_ssrf("http://localhost:8080/api", "test").is_err());
        assert!(validate_url_against_ssrf("http://LOCALHOST/api", "test").is_err());
    }

    #[test]
    fn test_ssrf_validation_loopback() {
        // Loopback addresses should be blocked
        assert!(validate_url_against_ssrf("http://127.0.0.1/api", "test").is_err());
        assert!(validate_url_against_ssrf("http://127.0.0.1:8080/api", "test").is_err());
        assert!(validate_url_against_ssrf("http://[::1]/api", "test").is_err());
    }

    #[test]
    fn test_ssrf_validation_private_ip() {
        // Private IP ranges should be blocked
        assert!(validate_url_against_ssrf("http://10.0.0.1/api", "test").is_err());
        assert!(validate_url_against_ssrf("http://172.16.0.1/api", "test").is_err());
        assert!(validate_url_against_ssrf("http://192.168.1.1/api", "test").is_err());
    }

    #[test]
    fn test_ssrf_validation_metadata_endpoints() {
        // Cloud metadata endpoints should be blocked
        assert!(validate_url_against_ssrf("http://169.254.169.254/latest/meta-data", "test").is_err());
        assert!(validate_url_against_ssrf("http://metadata.google.internal/computeMetadata", "test").is_err());
    }

    #[test]
    fn test_ssrf_validation_encoded_ip() {
        // Decimal-encoded IP addresses should be blocked
        // 2130706433 = 127.0.0.1
        assert!(validate_url_against_ssrf("http://2130706433/api", "test").is_err());
        // 167772161 = 10.0.0.1
        assert!(validate_url_against_ssrf("http://167772161/api", "test").is_err());
    }

    #[test]
    fn test_ssrf_validation_invalid_scheme() {
        // Non-HTTP schemes should be blocked
        assert!(validate_url_against_ssrf("file:///etc/passwd", "test").is_err());
        assert!(validate_url_against_ssrf("ftp://example.com", "test").is_err());
        assert!(validate_url_against_ssrf("gopher://example.com", "test").is_err());
    }

    #[test]
    fn test_provider_config_ssrf_validation() {
        let mut config = ProviderConfig {
            name: "test".to_string(),
            provider_type: "openai".to_string(),
            api_key: "test-key".to_string(),
            base_url: Some("http://localhost:8080".to_string()),
            ..Default::default()
        };

        // Should fail with localhost
        assert!(config.validate().is_err());

        // Should pass with valid public URL
        config.base_url = Some("https://api.openai.com/v1".to_string());
        assert!(config.validate().is_ok());

        // Should fail with private IP
        config.base_url = Some("http://192.168.1.1/api".to_string());
        assert!(config.validate().is_err());

        // Should fail with metadata endpoint
        config.base_url = Some("http://169.254.169.254/latest".to_string());
        assert!(config.validate().is_err());
    }
}
