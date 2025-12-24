//! Type conversions for GatewayError

use super::types::GatewayError;
use crate::core::providers::unified_provider::ProviderError;

// Conversion from unified ProviderError to GatewayError
impl From<ProviderError> for GatewayError {
    fn from(err: ProviderError) -> Self {
        match err {
            ProviderError::Authentication { message, .. } => GatewayError::Auth(message),
            ProviderError::RateLimit { message, .. } => GatewayError::RateLimit(message),
            ProviderError::ModelNotFound { model, .. } => {
                GatewayError::NotFound(format!("Model not found: {}", model))
            }
            ProviderError::InvalidRequest { message, .. } => GatewayError::BadRequest(message),
            ProviderError::Network { message, .. } => GatewayError::network(message),
            ProviderError::ProviderUnavailable { message, .. } => {
                GatewayError::ProviderUnavailable(message)
            }
            ProviderError::NotSupported { feature, provider } => GatewayError::NotImplemented(
                format!("Feature '{}' not supported by {}", feature, provider),
            ),
            ProviderError::NotImplemented { feature, provider } => GatewayError::NotImplemented(
                format!("Feature '{}' not implemented for {}", feature, provider),
            ),
            ProviderError::Configuration { message, .. } => GatewayError::Config(message),
            ProviderError::Serialization { message, .. } => GatewayError::parsing(message),
            ProviderError::Timeout { message, .. } => GatewayError::Timeout(message),
            ProviderError::QuotaExceeded { message, .. } => {
                GatewayError::BadRequest(format!("Quota exceeded: {}", message))
            }
            ProviderError::Other { message, .. } => GatewayError::Internal(message),

            // Enhanced error variants mapping
            ProviderError::ContextLengthExceeded {
                max,
                actual,
                provider,
            } => GatewayError::BadRequest(format!(
                "Context length exceeded for {}: max {} tokens, got {} tokens",
                provider, max, actual
            )),
            ProviderError::ContentFiltered {
                reason, provider, ..
            } => GatewayError::BadRequest(format!(
                "Content filtered by {} safety systems: {}",
                provider, reason
            )),
            ProviderError::ApiError {
                status,
                message,
                provider,
            } => match status {
                401 => GatewayError::Auth(format!("{}: {}", provider, message)),
                404 => GatewayError::NotFound(format!("{}: {}", provider, message)),
                429 => GatewayError::RateLimit(format!("{}: {}", provider, message)),
                400..=499 => GatewayError::BadRequest(format!("{}: {}", provider, message)),
                _ => GatewayError::Internal(format!("{}: {}", provider, message)),
            },
            ProviderError::TokenLimitExceeded { message, provider } => GatewayError::BadRequest(
                format!("Token limit exceeded for {}: {}", provider, message),
            ),
            ProviderError::FeatureDisabled { feature, provider } => {
                GatewayError::NotImplemented(format!("Feature '{}' disabled for {}", feature, provider))
            }
            ProviderError::DeploymentError {
                deployment,
                message,
                provider,
            } => GatewayError::NotFound(format!(
                "Azure deployment '{}' error for {}: {}",
                deployment, provider, message
            )),
            ProviderError::ResponseParsing { message, provider } => GatewayError::parsing(format!(
                "Failed to parse {} response: {}",
                provider, message
            )),
            ProviderError::RoutingError {
                attempted_providers,
                message,
                provider,
            } => GatewayError::ProviderUnavailable(format!(
                "Routing error from {}: tried {:?}, final error: {}",
                provider, attempted_providers, message
            )),
            ProviderError::TransformationError {
                from_format,
                to_format,
                message,
                provider,
            } => GatewayError::Internal(format!(
                "Transformation error for {}: from {} to {}: {}",
                provider, from_format, to_format, message
            )),
            ProviderError::Cancelled {
                provider,
                operation_type,
                ..
            } => GatewayError::BadRequest(format!(
                "Operation {} was cancelled for provider {}",
                operation_type, provider
            )),
            ProviderError::Streaming {
                provider,
                stream_type,
                ..
            } => GatewayError::Internal(format!(
                "Streaming error for provider {} on stream type {}",
                provider, stream_type
            )),
        }
    }
}
