//! Request context and authentication helpers

use crate::core::models::ApiKey;
use crate::core::models::RequestContext;
use crate::core::models::user::types::User;
use actix_web::http::header::HeaderMap;
use actix_web::{HttpRequest, Result as ActixResult};
use tracing::debug;

/// Get request context from headers and middleware extensions
pub fn get_request_context(req: &HttpRequest) -> ActixResult<RequestContext> {
    // In a real implementation, this would extract the context from request extensions
    // that were set by the authentication middleware
    let mut context = RequestContext::new();

    // Extract request ID
    if let Some(request_id) = req.headers().get("x-request-id") {
        if let Ok(id) = request_id.to_str() {
            context.request_id = id.to_string();
        }
    }

    // Extract user agent
    if let Some(user_agent) = req.headers().get("user-agent") {
        if let Ok(agent) = user_agent.to_str() {
            context.user_agent = Some(agent.to_string());
        }
    }

    Ok(context)
}

/// Extract user from request extensions
pub fn get_authenticated_user(_headers: &HeaderMap) -> Option<User> {
    // In a real implementation, this would extract the user from request extensions
    // that were set by the authentication middleware
    None
}

/// Extract API key from request extensions
pub fn get_authenticated_api_key(_headers: &HeaderMap) -> Option<ApiKey> {
    // In a real implementation, this would extract the API key from request extensions
    // that were set by the authentication middleware
    None
}

/// Check if user has permission for the requested operation
pub fn check_permission(user: Option<&User>, api_key: Option<&ApiKey>, _operation: &str) -> bool {
    // In a real implementation, this would check permissions through the RBAC system
    // For now, assume all authenticated requests are allowed
    user.is_some() || api_key.is_some()
}

/// Log API usage for billing and analytics
pub async fn log_api_usage(
    context: &RequestContext,
    model: &str,
    tokens_used: u32,
    cost: f64,
) {
    // In a real implementation, this would log usage to the database
    debug!(
        "API usage: user_id={:?}, model={}, tokens={}, cost={}",
        context.user_id, model, tokens_used, cost
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_permission() {
        // Test with no authentication
        assert!(!check_permission(None, None, "chat"));

        // Test with user (would need actual User instance in real test)
        // assert!(check_permission(Some(&user), None, "chat"));
    }

    #[tokio::test]
    async fn test_log_api_usage() {
        // This would require actual state in a real test
        // For now, just test that the function doesn't panic
        let context = RequestContext::new();
        log_api_usage(&context, "gpt-4", 100, 0.002).await;
    }
}
