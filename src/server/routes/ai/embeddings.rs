//! Embeddings endpoint

use crate::core::models::openai::{EmbeddingRequest, EmbeddingResponse};
use crate::core::models::RequestContext;
use crate::core::providers::ProviderRegistry;
use crate::server::routes::ApiResponse;
use crate::server::AppState;
use crate::utils::error::GatewayError;
use actix_web::{web, HttpRequest, HttpResponse, Result as ActixResult};
use tracing::{error, info};

use super::context::get_request_context;

/// Embeddings endpoint
///
/// OpenAI-compatible embeddings API for generating text embeddings.
pub async fn embeddings(
    state: web::Data<AppState>,
    req: HttpRequest,
    request: web::Json<EmbeddingRequest>,
) -> ActixResult<HttpResponse> {
    info!("Embedding request for model: {}", request.model);

    // Get request context from middleware
    let context = get_request_context(&req)?;

    // Route request through the core router
    // TODO: Implement proper embedding routing through ProviderRegistry
    match handle_embedding_via_pool(&state.router, request.into_inner(), context).await {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(e) => {
            error!("Embedding error: {}", e);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Error".to_string())))
        }
    }
}

/// Handle embedding via provider pool
pub async fn handle_embedding_via_pool(
    _pool: &ProviderRegistry,
    _request: EmbeddingRequest,
    _context: RequestContext,
) -> Result<EmbeddingResponse, GatewayError> {
    // TODO: Implement actual embedding routing via ProviderRegistry
    Err(GatewayError::internal(
        "Embedding routing not implemented yet",
    ))
}
