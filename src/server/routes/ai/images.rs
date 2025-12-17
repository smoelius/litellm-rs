//! Image generation endpoint

use crate::core::models::openai::{ImageGenerationRequest, ImageGenerationResponse};
use crate::core::models::RequestContext;
use crate::core::providers::ProviderRegistry;
use crate::server::routes::ApiResponse;
use crate::server::state::AppState;
use crate::utils::error::GatewayError;
use actix_web::{web, HttpRequest, HttpResponse, Result as ActixResult};
use tracing::{error, info};

use super::context::get_request_context;

/// Image generation endpoint
///
/// OpenAI-compatible image generation API.
pub async fn image_generations(
    state: web::Data<AppState>,
    req: HttpRequest,
    request: web::Json<ImageGenerationRequest>,
) -> ActixResult<HttpResponse> {
    info!("Image generation request for model: {:?}", request.model);

    // Get request context from middleware
    let context = get_request_context(&req)?;

    // Route request through the core router
    // TODO: Implement proper image generation routing through ProviderRegistry
    match handle_image_generation_via_pool(&state.router, request.into_inner(), context).await {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(e) => {
            error!("Image generation error: {}", e);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Error".to_string())))
        }
    }
}

/// Handle image generation via provider pool
pub async fn handle_image_generation_via_pool(
    _pool: &ProviderRegistry,
    _request: ImageGenerationRequest,
    _context: RequestContext,
) -> Result<ImageGenerationResponse, GatewayError> {
    // TODO: Implement actual image generation routing via ProviderRegistry
    Err(GatewayError::internal(
        "Image generation routing not implemented yet",
    ))
}
