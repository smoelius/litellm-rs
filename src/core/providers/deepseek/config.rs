//! DeepSeek Configuration
//!
//! Configuration

use crate::core::traits::ProviderConfig;
use crate::define_provider_config;

// Configuration
define_provider_config!(DeepSeekConfig {});

impl DeepSeekConfig {
    /// Create
    pub fn from_env() -> Self {
        Self::new("deepseek")
    }
}

// implementationProviderConfig trait
impl ProviderConfig for DeepSeekConfig {
    fn validate(&self) -> Result<(), String> {
        self.base.validate("deepseek")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deepseek_config() {
        let config = DeepSeekConfig::new("deepseek");
        assert_eq!(
            config.base.api_base,
            Some("https://api.deepseek.com".to_string())
        );
        assert_eq!(config.base.timeout, 60);
    }
}
