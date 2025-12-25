//! Provider factory integration tests
//!
//! Tests for the Provider enum and factory functions that create providers
//! from configuration. These tests verify the unified provider interface works
//! correctly across different provider types.

#[cfg(test)]
mod tests {
    use litellm_rs::core::providers::{Provider, ProviderType};
    use serde_json::json;

    /// Test creating OpenAI provider from config
    #[tokio::test]
    async fn test_openai_provider_from_config() {
        let config = json!({
            "api_key": "sk-proj-test-key-1234567890abcdef"  // Valid OpenAI format
        });

        let result = Provider::from_config_async(ProviderType::OpenAI, config).await;
        assert!(
            result.is_ok(),
            "Failed to create OpenAI provider: {:?}",
            result.err()
        );

        let provider = result.unwrap();
        assert_eq!(provider.name(), "openai");
    }

    /// Test creating Anthropic provider from config
    #[tokio::test]
    async fn test_anthropic_provider_from_config() {
        let config = json!({
            "api_key": "sk-ant-test-key"
        });

        let result = Provider::from_config_async(ProviderType::Anthropic, config).await;
        assert!(
            result.is_ok(),
            "Failed to create Anthropic provider: {:?}",
            result.err()
        );

        let provider = result.unwrap();
        assert_eq!(provider.name(), "anthropic");
    }

    /// Test creating Groq provider from config
    #[tokio::test]
    async fn test_groq_provider_from_config() {
        let config = json!({
            "api_key": "gsk-test-key"
        });

        let result = Provider::from_config_async(ProviderType::Groq, config).await;
        assert!(
            result.is_ok(),
            "Failed to create Groq provider: {:?}",
            result.err()
        );

        let provider = result.unwrap();
        assert_eq!(provider.name(), "groq");
    }

    /// Test creating XAI provider from config
    #[tokio::test]
    async fn test_xai_provider_from_config() {
        let config = json!({
            "api_key": "xai-test-key"
        });

        let result = Provider::from_config_async(ProviderType::XAI, config).await;
        assert!(
            result.is_ok(),
            "Failed to create XAI provider: {:?}",
            result.err()
        );

        let provider = result.unwrap();
        assert_eq!(provider.name(), "xai");
    }

    /// Test creating OpenRouter provider from config
    #[tokio::test]
    async fn test_openrouter_provider_from_config() {
        let config = json!({
            "api_key": "or-test-key"
        });

        let result = Provider::from_config_async(ProviderType::OpenRouter, config).await;
        assert!(
            result.is_ok(),
            "Failed to create OpenRouter provider: {:?}",
            result.err()
        );

        let provider = result.unwrap();
        assert_eq!(provider.name(), "openrouter");
    }

    /// Test creating Mistral provider from config
    #[tokio::test]
    async fn test_mistral_provider_from_config() {
        let config = json!({
            "api_key": "mistral-test-key"
        });

        let result = Provider::from_config_async(ProviderType::Mistral, config).await;
        assert!(
            result.is_ok(),
            "Failed to create Mistral provider: {:?}",
            result.err()
        );

        let provider = result.unwrap();
        assert_eq!(provider.name(), "mistral");
    }

    /// Test creating DeepSeek provider from config
    #[tokio::test]
    async fn test_deepseek_provider_from_config() {
        let config = json!({
            "api_key": "deepseek-test-key"
        });

        let result = Provider::from_config_async(ProviderType::DeepSeek, config).await;
        assert!(
            result.is_ok(),
            "Failed to create DeepSeek provider: {:?}",
            result.err()
        );

        let provider = result.unwrap();
        assert_eq!(provider.name(), "deepseek");
    }

    /// Test creating Moonshot provider from config
    #[tokio::test]
    async fn test_moonshot_provider_from_config() {
        let config = json!({
            "api_key": "moonshot-test-key"
        });

        let result = Provider::from_config_async(ProviderType::Moonshot, config).await;
        assert!(
            result.is_ok(),
            "Failed to create Moonshot provider: {:?}",
            result.err()
        );

        let provider = result.unwrap();
        assert_eq!(provider.name(), "moonshot");
    }

    /// Test creating Cloudflare provider from config
    #[tokio::test]
    async fn test_cloudflare_provider_from_config() {
        let config = json!({
            "account_id": "test-account-id",
            "api_token": "test-api-token"
        });

        let result = Provider::from_config_async(ProviderType::Cloudflare, config).await;
        assert!(
            result.is_ok(),
            "Failed to create Cloudflare provider: {:?}",
            result.err()
        );

        let provider = result.unwrap();
        assert_eq!(provider.name(), "cloudflare");
    }

    /// Test provider creation fails with missing api_key
    #[tokio::test]
    async fn test_provider_creation_fails_without_api_key() {
        let config = json!({});

        let result = Provider::from_config_async(ProviderType::OpenAI, config).await;
        assert!(result.is_err(), "Should fail without api_key");

        let err = result.unwrap_err();
        let err_str = err.to_string();
        assert!(
            err_str.contains("api_key") || err_str.contains("required"),
            "Error should mention api_key: {}",
            err_str
        );
    }

    /// Test Cloudflare provider fails without account_id
    #[tokio::test]
    async fn test_cloudflare_fails_without_account_id() {
        let config = json!({
            "api_token": "test-token"
        });

        let result = Provider::from_config_async(ProviderType::Cloudflare, config).await;
        assert!(result.is_err(), "Should fail without account_id");
    }

    /// Test provider capabilities are available
    #[tokio::test]
    async fn test_provider_capabilities() {
        let config = json!({
            "api_key": "gsk-test-key"  // Use Groq format which doesn't validate prefix
        });

        let provider = Provider::from_config_async(ProviderType::Groq, config)
            .await
            .unwrap();

        let capabilities = provider.capabilities();
        assert!(
            !capabilities.is_empty(),
            "Provider should have capabilities"
        );
    }

    /// Test provider models list
    #[tokio::test]
    async fn test_provider_models_list() {
        let config = json!({
            "api_key": "test-key"
        });

        let provider = Provider::from_config_async(ProviderType::Groq, config)
            .await
            .unwrap();

        let models = provider.list_models();
        assert!(!models.is_empty(), "Provider should have models");

        // All models should have required fields
        for model in models {
            assert!(!model.id.is_empty(), "Model should have id");
            assert!(!model.provider.is_empty(), "Model should have provider");
        }
    }

    /// Test provider type display
    #[test]
    fn test_provider_type_display() {
        assert_eq!(format!("{}", ProviderType::OpenAI), "openai");
        assert_eq!(format!("{}", ProviderType::Anthropic), "anthropic");
        assert_eq!(format!("{}", ProviderType::Groq), "groq");
        assert_eq!(format!("{}", ProviderType::XAI), "xai");
        assert_eq!(format!("{}", ProviderType::DeepSeek), "deepseek");
    }
}
