//! Text completions endpoint (legacy)

use crate::core::models::openai::{CompletionRequest, CompletionResponse};
use crate::core::models::RequestContext;
use crate::core::providers::ProviderRegistry;
use crate::server::state::AppState;
use crate::utils::error::GatewayError;
use actix_web::{web, HttpRequest, HttpResponse, Result as ActixResult};
use tracing::{error, info};

use super::context::get_request_context;

/// Text completions endpoint (legacy)
///
/// OpenAI-compatible text completions API for backward compatibility.
pub async fn completions(
    state: web::Data<AppState>,
    req: HttpRequest,
    request: web::Json<CompletionRequest>,
) -> ActixResult<HttpResponse> {
    info!("Text completion request for model: {}", request.model);

    // Get request context from middleware
    let context = get_request_context(&req)?;

    // Route request through the core router
    // TODO: Implement proper completion routing through ProviderRegistry
    match handle_completion_via_pool(&state.router, request.into_inner(), context).await {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(e) => {
            error!("Text completion error: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Internal server error"
            })))
        }
    }
}

/// Handle completion via provider pool
pub async fn handle_completion_via_pool(
    _pool: &ProviderRegistry,
    _request: CompletionRequest,
    _context: RequestContext,
) -> Result<CompletionResponse, GatewayError> {
    // TODO: Implement actual completion routing via ProviderRegistry
    Err(GatewayError::internal(
        "Text completion routing not implemented yet",
    ))
}
