//! Load balancer tests

use crate::core::router::load_balancer::{DeploymentInfo, FallbackConfig, LoadBalancer};
use crate::core::router::strategy::types::RoutingStrategy;
use crate::core::providers::unified_provider::ProviderError;

#[test]
fn test_fallback_config_builder() {
    let mut config = FallbackConfig::new();
    config
        .add_general_fallback("gpt-4", vec!["gpt-3.5-turbo".to_string()])
        .add_context_window_fallback("gpt-4", vec!["gpt-4-32k".to_string()])
        .add_content_policy_fallback("gpt-4", vec!["claude-3-opus".to_string()])
        .add_rate_limit_fallback("gpt-4", vec!["gpt-4-turbo".to_string()]);

    assert_eq!(
        config.general_fallbacks.get("gpt-4"),
        Some(&vec!["gpt-3.5-turbo".to_string()])
    );
    assert_eq!(
        config.context_window_fallbacks.get("gpt-4"),
        Some(&vec!["gpt-4-32k".to_string()])
    );
    assert_eq!(
        config.content_policy_fallbacks.get("gpt-4"),
        Some(&vec!["claude-3-opus".to_string()])
    );
    assert_eq!(
        config.rate_limit_fallbacks.get("gpt-4"),
        Some(&vec!["gpt-4-turbo".to_string()])
    );
}

#[tokio::test]
async fn test_load_balancer_creation() {
    let lb = LoadBalancer::new(RoutingStrategy::RoundRobin).await;
    assert!(lb.is_ok());
}

#[tokio::test]
async fn test_load_balancer_with_fallbacks() {
    let mut config = FallbackConfig::new();
    config.add_general_fallback("gpt-4", vec!["gpt-3.5-turbo".to_string()]);

    let lb = LoadBalancer::with_fallbacks(RoutingStrategy::RoundRobin, config).await;
    assert!(lb.is_ok());

    let lb = lb.unwrap();
    assert_eq!(
        lb.fallback_config().general_fallbacks.get("gpt-4"),
        Some(&vec!["gpt-3.5-turbo".to_string()])
    );
}

#[tokio::test]
async fn test_select_fallback_models_context_length() {
    let mut config = FallbackConfig::new();
    config
        .add_context_window_fallback(
            "gpt-4",
            vec!["gpt-4-32k".to_string(), "gpt-4-turbo".to_string()],
        )
        .add_general_fallback("gpt-4", vec!["gpt-3.5-turbo".to_string()]);

    let lb = LoadBalancer::with_fallbacks(RoutingStrategy::RoundRobin, config)
        .await
        .unwrap();

    // Context length error should use context_window_fallbacks
    let error = ProviderError::ContextLengthExceeded {
        provider: "openai",
        max: 8192,
        actual: 10000,
    };
    let fallbacks = lb.select_fallback_models(&error, "gpt-4");
    assert_eq!(
        fallbacks,
        Some(vec!["gpt-4-32k".to_string(), "gpt-4-turbo".to_string()])
    );
}

#[tokio::test]
async fn test_select_fallback_models_content_filtered() {
    let mut config = FallbackConfig::new();
    config
        .add_content_policy_fallback("gpt-4", vec!["claude-3-opus".to_string()])
        .add_general_fallback("gpt-4", vec!["gpt-3.5-turbo".to_string()]);

    let lb = LoadBalancer::with_fallbacks(RoutingStrategy::RoundRobin, config)
        .await
        .unwrap();

    // Content filter error should use content_policy_fallbacks
    let error = ProviderError::ContentFiltered {
        provider: "openai",
        reason: "Violated content policy".to_string(),
        policy_violations: None,
        potentially_retryable: Some(false),
    };
    let fallbacks = lb.select_fallback_models(&error, "gpt-4");
    assert_eq!(fallbacks, Some(vec!["claude-3-opus".to_string()]));
}

#[tokio::test]
async fn test_select_fallback_models_rate_limit() {
    let mut config = FallbackConfig::new();
    config
        .add_rate_limit_fallback("gpt-4", vec!["gpt-4-turbo".to_string()])
        .add_general_fallback("gpt-4", vec!["gpt-3.5-turbo".to_string()]);

    let lb = LoadBalancer::with_fallbacks(RoutingStrategy::RoundRobin, config)
        .await
        .unwrap();

    // Rate limit error should use rate_limit_fallbacks
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

#[tokio::test]
async fn test_select_fallback_models_falls_back_to_general() {
    let mut config = FallbackConfig::new();
    // Only general fallback configured
    config.add_general_fallback("gpt-4", vec!["gpt-3.5-turbo".to_string()]);

    let lb = LoadBalancer::with_fallbacks(RoutingStrategy::RoundRobin, config)
        .await
        .unwrap();

    // Any error should fall back to general
    let error = ProviderError::timeout("openai", "Request timeout");
    let fallbacks = lb.select_fallback_models(&error, "gpt-4");
    assert_eq!(fallbacks, Some(vec!["gpt-3.5-turbo".to_string()]));
}

#[tokio::test]
async fn test_select_fallback_models_no_config() {
    let lb = LoadBalancer::new(RoutingStrategy::RoundRobin)
        .await
        .unwrap();

    let error = ProviderError::timeout("openai", "Request timeout");
    let fallbacks = lb.select_fallback_models(&error, "gpt-4");
    assert_eq!(fallbacks, None);
}

// Tag/Group routing tests

#[test]
fn test_deployment_info_builder() {
    let info = DeploymentInfo::new()
        .with_tag("fast")
        .with_tag("high-quality")
        .with_group("gpt-4-group")
        .with_priority(1);

    assert_eq!(info.tags, vec!["fast", "high-quality"]);
    assert_eq!(info.model_group, Some("gpt-4-group".to_string()));
    assert_eq!(info.priority, 1);
}

#[test]
fn test_deployment_info_with_tags() {
    let info = DeploymentInfo::new().with_tags(["fast", "cheap", "reliable"]);

    assert_eq!(info.tags.len(), 3);
    assert!(info.tags.contains(&"fast".to_string()));
    assert!(info.tags.contains(&"cheap".to_string()));
    assert!(info.tags.contains(&"reliable".to_string()));
}

#[test]
fn test_deployment_info_has_all_tags() {
    let info = DeploymentInfo::new().with_tags(["fast", "cheap", "reliable"]);

    // Should match when all tags present
    assert!(info.has_all_tags(&["fast".to_string(), "cheap".to_string()]));

    // Should not match when a tag is missing
    assert!(!info.has_all_tags(&["fast".to_string(), "expensive".to_string()]));

    // Empty tags should always match
    assert!(info.has_all_tags(&[]));
}

#[test]
fn test_deployment_info_has_any_tag() {
    let info = DeploymentInfo::new().with_tags(["fast", "cheap"]);

    // Should match when any tag present
    assert!(info.has_any_tag(&["fast".to_string(), "expensive".to_string()]));

    // Should not match when no tags present
    assert!(!info.has_any_tag(&["expensive".to_string(), "slow".to_string()]));

    // Empty tags should not match
    assert!(!info.has_any_tag(&[]));
}

#[tokio::test]
async fn test_load_balancer_with_deployment_info() {
    let lb = LoadBalancer::new(RoutingStrategy::RoundRobin)
        .await
        .unwrap();

    let deployment = DeploymentInfo::new()
        .with_tag("fast")
        .with_group("test-group");

    // Use update_deployment_info since we don't have a real Provider
    lb.update_deployment_info("test_provider", deployment);

    let info = lb.get_deployment_info("test_provider");
    assert!(info.is_some());

    let info = info.unwrap();
    assert!(info.tags.contains(&"fast".to_string()));
    assert_eq!(info.model_group, Some("test-group".to_string()));
}

#[tokio::test]
async fn test_get_providers_by_tag() {
    let lb = LoadBalancer::new(RoutingStrategy::RoundRobin)
        .await
        .unwrap();

    // Add deployment info directly
    lb.update_deployment_info(
        "provider_a",
        DeploymentInfo::new().with_tags(["fast", "cheap"]),
    );
    lb.update_deployment_info(
        "provider_b",
        DeploymentInfo::new().with_tags(["fast", "quality"]),
    );
    lb.update_deployment_info("provider_c", DeploymentInfo::new().with_tag("quality"));

    let fast_providers = lb.get_providers_by_tag("fast");
    assert_eq!(fast_providers.len(), 2);
    assert!(fast_providers.contains(&"provider_a".to_string()));
    assert!(fast_providers.contains(&"provider_b".to_string()));

    let quality_providers = lb.get_providers_by_tag("quality");
    assert_eq!(quality_providers.len(), 2);
    assert!(quality_providers.contains(&"provider_b".to_string()));
    assert!(quality_providers.contains(&"provider_c".to_string()));

    let cheap_providers = lb.get_providers_by_tag("cheap");
    assert_eq!(cheap_providers.len(), 1);
    assert!(cheap_providers.contains(&"provider_a".to_string()));
}

#[tokio::test]
async fn test_get_providers_by_group() {
    let lb = LoadBalancer::new(RoutingStrategy::RoundRobin)
        .await
        .unwrap();

    lb.update_deployment_info("provider_a", DeploymentInfo::new().with_group("gpt-4-group"));
    lb.update_deployment_info("provider_b", DeploymentInfo::new().with_group("gpt-4-group"));
    lb.update_deployment_info("provider_c", DeploymentInfo::new().with_group("claude-group"));

    let gpt4_providers = lb.get_providers_by_group("gpt-4-group");
    assert_eq!(gpt4_providers.len(), 2);
    assert!(gpt4_providers.contains(&"provider_a".to_string()));
    assert!(gpt4_providers.contains(&"provider_b".to_string()));

    let claude_providers = lb.get_providers_by_group("claude-group");
    assert_eq!(claude_providers.len(), 1);
    assert!(claude_providers.contains(&"provider_c".to_string()));
}

#[tokio::test]
async fn test_get_all_tags() {
    let lb = LoadBalancer::new(RoutingStrategy::RoundRobin)
        .await
        .unwrap();

    lb.update_deployment_info(
        "provider_a",
        DeploymentInfo::new().with_tags(["fast", "cheap"]),
    );
    lb.update_deployment_info(
        "provider_b",
        DeploymentInfo::new().with_tags(["fast", "quality"]),
    );

    let all_tags = lb.get_all_tags();
    assert_eq!(all_tags.len(), 3); // cheap, fast, quality (sorted, deduplicated)
    assert_eq!(all_tags, vec!["cheap", "fast", "quality"]);
}

#[tokio::test]
async fn test_get_all_groups() {
    let lb = LoadBalancer::new(RoutingStrategy::RoundRobin)
        .await
        .unwrap();

    lb.update_deployment_info("provider_a", DeploymentInfo::new().with_group("gpt-4-group"));
    lb.update_deployment_info("provider_b", DeploymentInfo::new().with_group("gpt-4-group"));
    lb.update_deployment_info("provider_c", DeploymentInfo::new().with_group("claude-group"));
    lb.update_deployment_info("provider_d", DeploymentInfo::new()); // No group

    let all_groups = lb.get_all_groups();
    assert_eq!(all_groups.len(), 2); // claude-group, gpt-4-group (sorted, deduplicated)
    assert_eq!(all_groups, vec!["claude-group", "gpt-4-group"]);
}
