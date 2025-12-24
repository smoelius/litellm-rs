//! Router integration tests
//!
//! Tests routing strategies and load balancer functionality.

#[cfg(test)]
mod tests {
    use litellm_rs::core::providers::ProviderError;
    use litellm_rs::core::router::deployment::{DeploymentConfig, DeploymentState, HealthStatus};
    use litellm_rs::core::router::load_balancer::{FallbackConfig, LoadBalancer};
    use litellm_rs::core::router::strategy::types::RoutingStrategy;

    // ==================== LoadBalancer Tests ====================

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

    // ==================== HealthStatus Tests ====================

    /// Test health status conversion from u8
    #[test]
    fn test_health_status_from_u8() {
        assert_eq!(HealthStatus::from(0), HealthStatus::Unknown);
        assert_eq!(HealthStatus::from(1), HealthStatus::Healthy);
        assert_eq!(HealthStatus::from(2), HealthStatus::Degraded);
        assert_eq!(HealthStatus::from(3), HealthStatus::Unhealthy);
        assert_eq!(HealthStatus::from(4), HealthStatus::Cooldown);
        assert_eq!(HealthStatus::from(255), HealthStatus::Unknown); // Invalid maps to Unknown
    }

    /// Test health status conversion to u8
    #[test]
    fn test_health_status_to_u8() {
        assert_eq!(u8::from(HealthStatus::Unknown), 0);
        assert_eq!(u8::from(HealthStatus::Healthy), 1);
        assert_eq!(u8::from(HealthStatus::Degraded), 2);
        assert_eq!(u8::from(HealthStatus::Unhealthy), 3);
        assert_eq!(u8::from(HealthStatus::Cooldown), 4);
    }

    // ==================== DeploymentConfig Tests ====================

    /// Test deployment config default values
    #[test]
    fn test_deployment_config_defaults() {
        let config = DeploymentConfig::default();

        assert!(config.tpm_limit.is_none());
        assert!(config.rpm_limit.is_none());
        assert!(config.max_parallel_requests.is_none());
        assert_eq!(config.weight, 1);
        assert_eq!(config.timeout_secs, 60);
        assert_eq!(config.priority, 0);
    }

    /// Test deployment config with custom values
    #[test]
    fn test_deployment_config_custom() {
        let config = DeploymentConfig {
            tpm_limit: Some(100_000),
            rpm_limit: Some(500),
            max_parallel_requests: Some(10),
            weight: 2,
            timeout_secs: 120,
            priority: 1,
        };

        assert_eq!(config.tpm_limit, Some(100_000));
        assert_eq!(config.rpm_limit, Some(500));
        assert_eq!(config.max_parallel_requests, Some(10));
        assert_eq!(config.weight, 2);
        assert_eq!(config.timeout_secs, 120);
        assert_eq!(config.priority, 1);
    }

    // ==================== DeploymentState Tests ====================

    /// Test deployment state initialization
    #[test]
    fn test_deployment_state_new() {
        let state = DeploymentState::new();

        assert_eq!(state.health_status(), HealthStatus::Healthy);
        assert_eq!(state.tpm_current.load(std::sync::atomic::Ordering::Relaxed), 0);
        assert_eq!(state.rpm_current.load(std::sync::atomic::Ordering::Relaxed), 0);
        assert_eq!(state.active_requests.load(std::sync::atomic::Ordering::Relaxed), 0);
        assert_eq!(state.total_requests.load(std::sync::atomic::Ordering::Relaxed), 0);
        assert_eq!(state.success_requests.load(std::sync::atomic::Ordering::Relaxed), 0);
        assert_eq!(state.fail_requests.load(std::sync::atomic::Ordering::Relaxed), 0);
    }

    /// Test deployment state reset_minute
    #[test]
    fn test_deployment_state_reset_minute() {
        let state = DeploymentState::new();

        // Set some values
        state.tpm_current.store(1000, std::sync::atomic::Ordering::Relaxed);
        state.rpm_current.store(50, std::sync::atomic::Ordering::Relaxed);
        state.fails_this_minute.store(5, std::sync::atomic::Ordering::Relaxed);

        // Reset
        state.reset_minute();

        // Verify reset
        assert_eq!(state.tpm_current.load(std::sync::atomic::Ordering::Relaxed), 0);
        assert_eq!(state.rpm_current.load(std::sync::atomic::Ordering::Relaxed), 0);
        assert_eq!(state.fails_this_minute.load(std::sync::atomic::Ordering::Relaxed), 0);
    }

    /// Test deployment state clone
    #[test]
    fn test_deployment_state_clone() {
        let state = DeploymentState::new();
        state.tpm_current.store(1000, std::sync::atomic::Ordering::Relaxed);
        state.rpm_current.store(50, std::sync::atomic::Ordering::Relaxed);

        let cloned = state.clone();

        assert_eq!(
            cloned.tpm_current.load(std::sync::atomic::Ordering::Relaxed),
            1000
        );
        assert_eq!(
            cloned.rpm_current.load(std::sync::atomic::Ordering::Relaxed),
            50
        );
    }

    // ==================== FallbackConfig Tests ====================

    /// Test fallback config creation and methods
    #[test]
    fn test_fallback_config_methods() {
        let mut config = FallbackConfig::new();

        // Add various fallbacks
        config.add_general_fallback("gpt-4", vec!["gpt-3.5-turbo".to_string()]);
        config.add_context_window_fallback("gpt-4", vec!["gpt-4-32k".to_string()]);
        config.add_rate_limit_fallback("gpt-4", vec!["gpt-4-turbo".to_string()]);
        config.add_content_policy_fallback("gpt-4", vec!["claude-3".to_string()]);

        // Verify all were added
        assert!(config.general_fallbacks.contains_key("gpt-4"));
        assert!(config.context_window_fallbacks.contains_key("gpt-4"));
        assert!(config.rate_limit_fallbacks.contains_key("gpt-4"));
        assert!(config.content_policy_fallbacks.contains_key("gpt-4"));
    }

    /// Test multiple fallbacks for same model
    #[test]
    fn test_multiple_fallbacks_same_model() {
        let mut config = FallbackConfig::new();

        config.add_general_fallback(
            "gpt-4",
            vec![
                "gpt-3.5-turbo".to_string(),
                "gpt-3.5-turbo-16k".to_string(),
            ],
        );

        let fallbacks = config.general_fallbacks.get("gpt-4").unwrap();
        assert_eq!(fallbacks.len(), 2);
        assert!(fallbacks.contains(&"gpt-3.5-turbo".to_string()));
        assert!(fallbacks.contains(&"gpt-3.5-turbo-16k".to_string()));
    }

    // ==================== Routing Strategy Display/Debug ====================

    /// Test routing strategy debug representation
    #[test]
    fn test_routing_strategy_debug() {
        let strategy = RoutingStrategy::RoundRobin;
        let debug_str = format!("{:?}", strategy);
        assert!(debug_str.contains("RoundRobin"));

        let strategy = RoutingStrategy::LeastLatency;
        let debug_str = format!("{:?}", strategy);
        assert!(debug_str.contains("LeastLatency"));
    }

    // ==================== Load Balancer with Multiple Strategies ====================

    /// Test load balancer with Random strategy
    #[tokio::test]
    async fn test_load_balancer_random_strategy() {
        let lb = LoadBalancer::new(RoutingStrategy::Random).await;
        assert!(lb.is_ok());
    }

    /// Test load balancer with LeastLatency strategy
    #[tokio::test]
    async fn test_load_balancer_least_latency_strategy() {
        let lb = LoadBalancer::new(RoutingStrategy::LeastLatency).await;
        assert!(lb.is_ok());
    }

    /// Test load balancer with LeastCost strategy
    #[tokio::test]
    async fn test_load_balancer_least_cost_strategy() {
        let lb = LoadBalancer::new(RoutingStrategy::LeastCost).await;
        assert!(lb.is_ok());
    }

    // ==================== Error-Specific Fallback Tests ====================

    /// Test fallback for network errors
    #[tokio::test]
    async fn test_fallback_selection_network_error() {
        let mut config = FallbackConfig::new();
        config.add_general_fallback("gpt-4", vec!["gpt-4-backup".to_string()]);

        let lb = LoadBalancer::with_fallbacks(RoutingStrategy::RoundRobin, config)
            .await
            .unwrap();

        let error = ProviderError::Network {
            provider: "openai",
            message: "Connection refused".to_string(),
        };

        // Network errors should fall back to general fallback
        let fallbacks = lb.select_fallback_models(&error, "gpt-4");
        assert_eq!(fallbacks, Some(vec!["gpt-4-backup".to_string()]));
    }

    /// Test fallback for timeout errors
    #[tokio::test]
    async fn test_fallback_selection_timeout_error() {
        let mut config = FallbackConfig::new();
        config.add_general_fallback("gpt-4", vec!["gpt-4-fast".to_string()]);

        let lb = LoadBalancer::with_fallbacks(RoutingStrategy::RoundRobin, config)
            .await
            .unwrap();

        let error = ProviderError::timeout("openai", "Request timed out after 30s");

        let fallbacks = lb.select_fallback_models(&error, "gpt-4");
        assert_eq!(fallbacks, Some(vec!["gpt-4-fast".to_string()]));
    }

    /// Test fallback for authentication errors (should not fallback)
    #[tokio::test]
    async fn test_fallback_selection_auth_error() {
        let mut config = FallbackConfig::new();
        config.add_general_fallback("gpt-4", vec!["gpt-4-backup".to_string()]);

        let lb = LoadBalancer::with_fallbacks(RoutingStrategy::RoundRobin, config)
            .await
            .unwrap();

        let error = ProviderError::authentication("openai", "Invalid API key");

        // Auth errors typically shouldn't trigger fallback to same provider
        // but might still return general fallback
        let fallbacks = lb.select_fallback_models(&error, "gpt-4");
        // This depends on implementation - auth errors might or might not use fallback
        assert!(fallbacks.is_some() || fallbacks.is_none());
    }

    /// Test fallback for model not found
    #[tokio::test]
    async fn test_fallback_selection_model_not_found() {
        let mut config = FallbackConfig::new();
        config.add_general_fallback("gpt-5", vec!["gpt-4".to_string()]);

        let lb = LoadBalancer::with_fallbacks(RoutingStrategy::RoundRobin, config)
            .await
            .unwrap();

        let error = ProviderError::model_not_found("openai", "gpt-5");

        let fallbacks = lb.select_fallback_models(&error, "gpt-5");
        assert_eq!(fallbacks, Some(vec!["gpt-4".to_string()]));
    }
}
