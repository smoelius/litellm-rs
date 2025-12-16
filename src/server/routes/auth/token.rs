//! Token refresh endpoint

use crate::server::AppState;
use crate::server::routes::ApiResponse;
use actix_web::{HttpResponse, Result as ActixResult, web};
use tracing::{debug, error, warn};

use super::models::RefreshTokenRequest;

/// Refresh token endpoint
pub async fn refresh_token(
    state: web::Data<AppState>,
    request: web::Json<RefreshTokenRequest>,
) -> ActixResult<HttpResponse> {
    debug!("Token refresh request");

    // Verify refresh token
    match state
        .auth
        .jwt()
        .verify_refresh_token(&request.refresh_token)
        .await
    {
        Ok(user_id) => {
            // Find user to get current role
            let user = match state.storage.database.find_user_by_id(user_id).await {
                Ok(Some(user)) => user,
                Ok(None) => {
                    warn!("Refresh token for non-existent user: {}", user_id);
                    return Ok(HttpResponse::Unauthorized()
                        .json(ApiResponse::<()>::error("Invalid token".to_string())));
                }
                Err(e) => {
                    error!("Database error during token refresh: {}", e);
                    return Ok(HttpResponse::InternalServerError()
                        .json(ApiResponse::<()>::error("Database error".to_string())));
                }
            };

            // Generate new token pair
            let user_permissions = state
                .auth
                .rbac()
                .get_user_permissions(&user)
                .await
                .unwrap_or_default();

            match state
                .auth
                .jwt()
                .create_token_pair(
                    user.id(),
                    format!("{:?}", user.role),
                    user_permissions,
                    None,
                    None,
                )
                .await
            {
                Ok(tokens) => {
                    debug!("Token refreshed successfully for user: {}", user.username);
                    Ok(HttpResponse::Ok().json(ApiResponse::success(tokens)))
                }
                Err(e) => {
                    error!("Failed to generate new tokens: {}", e);
                    Ok(
                        HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                            "Internal server error".to_string(),
                        )),
                    )
                }
            }
        }
        Err(e) => {
            warn!("Invalid refresh token: {}", e);
            Ok(
                HttpResponse::BadRequest().json(ApiResponse::<()>::error_for_type(
                    "Invalid refresh token".to_string(),
                )),
            )
        }
    }
}
