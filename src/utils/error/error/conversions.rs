//! Type conversions for GatewayError

use super::types::GatewayError;
use crate::core::providers::unified_provider::ProviderError;

// Add conversion from core::providers::unified_provider::ProviderError
impl From<crate::core::providers::unified_provider::ProviderError> for GatewayError {
    fn from(err: crate::core::providers::unified_provider::ProviderError) -> Self {
        match err {
            crate::core::providers::unified_provider::ProviderError::Authentication {
                message,
                ..
            } => GatewayError::Auth(message),
            crate::core::providers::unified_provider::ProviderError::RateLimit {
                message, ..
            } => GatewayError::RateLimit(message),
            crate::core::providers::unified_provider::ProviderError::ModelNotFound {
                model,
                ..
            } => GatewayError::NotFound(format!("Model not found: {}", model)),
            crate::core::providers::unified_provider::ProviderError::InvalidRequest {
                message,
                ..
            } => GatewayError::BadRequest(message),
            crate::core::providers::unified_provider::ProviderError::Network {
                message, ..
            } => GatewayError::network(message),
            crate::core::providers::unified_provider::ProviderError::ProviderUnavailable {
                message,
                ..
            } => GatewayError::ProviderUnavailable(message),
            crate::core::providers::unified_provider::ProviderError::NotSupported {
                feature,
                provider,
            } => GatewayError::NotImplemented(format!(
                "Feature '{}' not supported by {}",
                feature, provider
            )),
            crate::core::providers::unified_provider::ProviderError::NotImplemented {
                feature,
                provider,
            } => GatewayError::NotImplemented(format!(
                "Feature '{}' not implemented for {}",
                feature, provider
            )),
            crate::core::providers::unified_provider::ProviderError::Configuration {
                message,
                ..
            } => GatewayError::Config(message),
            crate::core::providers::unified_provider::ProviderError::Serialization {
                message,
                ..
            } => GatewayError::parsing(message),
            crate::core::providers::unified_provider::ProviderError::Timeout {
                message, ..
            } => GatewayError::Timeout(message),
            crate::core::providers::unified_provider::ProviderError::QuotaExceeded {
                message,
                ..
            } => GatewayError::BadRequest(format!("Quota exceeded: {}", message)),
            crate::core::providers::unified_provider::ProviderError::Other { message, .. } => {
                GatewayError::Internal(message)
            }

            // Enhanced error variants mapping
            crate::core::providers::unified_provider::ProviderError::ContextLengthExceeded {
                max,
                actual,
                provider,
            } => GatewayError::BadRequest(format!(
                "Context length exceeded for {}: max {} tokens, got {} tokens",
                provider, max, actual
            )),
            crate::core::providers::unified_provider::ProviderError::ContentFiltered {
                reason,
                provider,
                ..
            } => GatewayError::BadRequest(format!(
                "Content filtered by {} safety systems: {}",
                provider, reason
            )),
            crate::core::providers::unified_provider::ProviderError::ApiError {
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
            crate::core::providers::unified_provider::ProviderError::TokenLimitExceeded {
                message,
                provider,
            } => GatewayError::BadRequest(format!(
                "Token limit exceeded for {}: {}",
                provider, message
            )),
            crate::core::providers::unified_provider::ProviderError::FeatureDisabled {
                feature,
                provider,
            } => GatewayError::NotImplemented(format!(
                "Feature '{}' disabled for {}",
                feature, provider
            )),
            crate::core::providers::unified_provider::ProviderError::DeploymentError {
                deployment,
                message,
                provider,
            } => GatewayError::NotFound(format!(
                "Azure deployment '{}' error for {}: {}",
                deployment, provider, message
            )),
            crate::core::providers::unified_provider::ProviderError::ResponseParsing {
                message,
                provider,
            } => GatewayError::parsing(format!(
                "Failed to parse {} response: {}",
                provider, message
            )),
            crate::core::providers::unified_provider::ProviderError::RoutingError {
                attempted_providers,
                message,
                provider,
            } => GatewayError::ProviderUnavailable(format!(
                "Routing error from {}: tried {:?}, final error: {}",
                provider, attempted_providers, message
            )),
            crate::core::providers::unified_provider::ProviderError::TransformationError {
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

// AnthropicProviderError is now replaced by unified ProviderError
// The conversion is handled by the unified ProviderError From implementations
