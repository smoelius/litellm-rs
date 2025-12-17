//! Convenience functions for common configurations

use super::{ProviderConfigBuilder, ServerConfigBuilder};
use crate::utils::error::Result;
use std::time::Duration;

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
