//! Tests for error handling

#[cfg(test)]
mod tests {
    use super::super::types::GatewayError;
    use crate::core::providers::unified_provider::ProviderError;

    #[test]
    fn test_error_creation() {
        let error = GatewayError::auth("Invalid token");
        assert!(matches!(error, GatewayError::Auth(_)));

        let error = GatewayError::bad_request("Missing parameter");
        assert!(matches!(error, GatewayError::BadRequest(_)));
    }

    #[test]
    fn test_provider_error_creation() {
        let error = ProviderError::other("openai", "Bad request");
        assert!(matches!(error, ProviderError::Other { .. }));

        let error = ProviderError::rate_limit("openai", Some(60));
        assert!(matches!(error, ProviderError::RateLimit { .. }));
    }
}
