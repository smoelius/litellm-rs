#[cfg(test)]
mod tests {
    use super::super::types::{HttpClientConfig, RequestMetrics, RetryConfig};
    use super::super::utils::ClientUtils;
    use std::time::Duration;

    #[test]
    fn test_retry_logic() {
        assert!(ClientUtils::should_retry_request(429, 0, 3));
        assert!(ClientUtils::should_retry_request(500, 0, 3));
        assert!(ClientUtils::should_retry_request(502, 1, 3));
        assert!(!ClientUtils::should_retry_request(400, 0, 3));
        assert!(!ClientUtils::should_retry_request(429, 3, 3));
    }

    #[test]
    fn test_timeout_for_provider() {
        assert_eq!(
            ClientUtils::get_timeout_for_provider("openai"),
            Duration::from_secs(120)
        );
        assert_eq!(
            ClientUtils::get_timeout_for_provider("anthropic"),
            Duration::from_secs(180)
        );
        assert_eq!(
            ClientUtils::get_timeout_for_provider("unknown"),
            Duration::from_secs(60)
        );
    }

    #[test]
    fn test_add_path_to_api_base() {
        assert_eq!(
            ClientUtils::add_path_to_api_base("https://api.openai.com", "/v1/chat/completions"),
            "https://api.openai.com/v1/chat/completions"
        );

        assert_eq!(
            ClientUtils::add_path_to_api_base("https://api.openai.com/", "v1/chat/completions"),
            "https://api.openai.com/v1/chat/completions"
        );
    }

    #[test]
    fn test_url_validation() {
        assert!(ClientUtils::validate_url("https://api.openai.com").is_ok());
        assert!(ClientUtils::validate_url("http://localhost:8080").is_ok());
        assert!(ClientUtils::validate_url("ftp://example.com").is_err());
        assert!(ClientUtils::validate_url("not-a-url").is_err());
    }

    #[test]
    fn test_supports_httpx_timeout() {
        assert!(ClientUtils::supports_httpx_timeout("openai"));
        assert!(ClientUtils::supports_httpx_timeout("anthropic"));
        assert!(!ClientUtils::supports_httpx_timeout("unknown"));
    }

    #[test]
    fn test_user_agent_for_provider() {
        assert_eq!(
            ClientUtils::get_user_agent_for_provider("openai"),
            "litellm-rust-openai/1.0"
        );
        assert_eq!(
            ClientUtils::get_user_agent_for_provider("unknown"),
            "litellm-rust/1.0"
        );
    }

    #[test]
    fn test_parse_content_type() {
        let (media_type, params) =
            ClientUtils::parse_content_type("text/html; charset=utf-8; boundary=something");
        assert_eq!(media_type, "text/html");
        assert_eq!(params.get("charset"), Some(&"utf-8".to_string()));
        assert_eq!(params.get("boundary"), Some(&"something".to_string()));
    }

    #[test]
    fn test_request_metrics() {
        let mut metrics = RequestMetrics::new("openai".to_string(), "gpt-4".to_string());
        assert_eq!(metrics.retry_count, 0);
        assert!(metrics.end_time.is_none());

        metrics.increment_retry();
        assert_eq!(metrics.retry_count, 1);

        metrics.finish(Some(200));
        assert!(metrics.end_time.is_some());
        assert_eq!(metrics.status_code, Some(200));
    }
}
