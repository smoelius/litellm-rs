//! Router integration tests
//!
//! Tests routing strategies and load balancer functionality.

#[cfg(test)]
mod tests {
    use litellm_rs::core::providers::ProviderError;
    use litellm_rs::core::router::load_balancer::{FallbackConfig, LoadBalancer};
    use litellm_rs::core::router::strategy::RoutingStrategy;

    /// Test load balancer creation
    #[tokio::test]
    async fn test_load_balancer_creation() {
        let lb = LoadBalancer::new(RoutingStrategy::RoundRobin).await;
        assert!(lb.is_ok(), "Failed to create load balancer: {:?}", lb.err());
    }

    /// Test load balancer with custom fallback configuration
    #[tokio::test]
    async fn test_load_balancer_with_fallback_config() {
        let mut config = FallbackConfig::new();
        config.add_general_fallback("gpt-4", vec!["gpt-3.5-turbo".to_string()]);
        config.add_context_window_fallback("gpt-4", vec!["gpt-4-32k".to_string()]);

        let lb = LoadBalancer::with_fallbacks(RoutingStrategy::RoundRobin, config).await;
        assert!(lb.is_ok());

        let lb = lb.unwrap();
        assert!(lb.fallback_config().general_fallbacks.contains_key("gpt-4"));
        assert!(
            lb.fallback_config()
                .context_window_fallbacks
                .contains_key("gpt-4")
        );
    }

    /// Test fallback selection for context length errors
    #[tokio::test]
    async fn test_fallback_selection_context_length() {
        let mut config = FallbackConfig::new();
        config.add_context_window_fallback("gpt-4", vec!["gpt-4-32k".to_string()]);

        let lb = LoadBalancer::with_fallbacks(RoutingStrategy::RoundRobin, config)
            .await
            .unwrap();

        let error = ProviderError::ContextLengthExceeded {
            provider: "openai",
            max: 8192,
            actual: 10000,
        };

        let fallbacks = lb.select_fallback_models(&error, "gpt-4");
        assert_eq!(fallbacks, Some(vec!["gpt-4-32k".to_string()]));
    }

    /// Test fallback selection for rate limit errors
    #[tokio::test]
    async fn test_fallback_selection_rate_limit() {
        let mut config = FallbackConfig::new();
        config.add_rate_limit_fallback("gpt-4", vec!["gpt-4-turbo".to_string()]);

        let lb = LoadBalancer::with_fallbacks(RoutingStrategy::RoundRobin, config)
            .await
            .unwrap();

        let error = ProviderError::RateLimit {
            provider: "openai",
            message: "Rate limit exceeded".to_string(),
            retry_after: Some(60),
            rpm_limit: Some(100),
            tpm_limit: Some(10000),
            current_usage: None,
        };

        let fallbacks = lb.select_fallback_models(&error, "gpt-4");
        assert_eq!(fallbacks, Some(vec!["gpt-4-turbo".to_string()]));
    }

    /// Test fallback selection for content filter errors
    #[tokio::test]
    async fn test_fallback_selection_content_filter() {
        let mut config = FallbackConfig::new();
        config.add_content_policy_fallback("gpt-4", vec!["claude-3-opus".to_string()]);

        let lb = LoadBalancer::with_fallbacks(RoutingStrategy::RoundRobin, config)
            .await
            .unwrap();

        let error = ProviderError::ContentFiltered {
            provider: "openai",
            reason: "Content policy violation".to_string(),
            policy_violations: None,
            potentially_retryable: Some(false),
        };

        let fallbacks = lb.select_fallback_models(&error, "gpt-4");
        assert_eq!(fallbacks, Some(vec!["claude-3-opus".to_string()]));
    }

    /// Test general fallback when specific fallback not configured
    #[tokio::test]
    async fn test_fallback_to_general() {
        let mut config = FallbackConfig::new();
        config.add_general_fallback("gpt-4", vec!["gpt-3.5-turbo".to_string()]);

        let lb = LoadBalancer::with_fallbacks(RoutingStrategy::RoundRobin, config)
            .await
            .unwrap();

        // Context length error but no specific context fallback configured
        let error = ProviderError::ContextLengthExceeded {
            provider: "openai",
            max: 8192,
            actual: 10000,
        };

        // Should fall back to general fallback
        let fallbacks = lb.select_fallback_models(&error, "gpt-4");
        assert_eq!(fallbacks, Some(vec!["gpt-3.5-turbo".to_string()]));
    }

    /// Test no fallback when nothing configured
    #[tokio::test]
    async fn test_no_fallback() {
        let lb = LoadBalancer::new(RoutingStrategy::RoundRobin)
            .await
            .unwrap();

        let error = ProviderError::timeout("openai", "Request timeout");
        let fallbacks = lb.select_fallback_models(&error, "gpt-4");
        assert!(fallbacks.is_none());
    }

    /// Test routing strategy variants
    #[test]
    fn test_routing_strategy_variants() {
        // Verify all strategies can be created
        let strategies = [
            RoutingStrategy::RoundRobin,
            RoutingStrategy::Random,
            RoutingStrategy::LeastLatency,
            RoutingStrategy::LeastCost,
        ];

        assert_eq!(strategies.len(), 4);
    }
}
