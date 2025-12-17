//! Email verification endpoint

use crate::server::state::AppState;
use crate::server::routes::ApiResponse;
use actix_web::{HttpResponse, Result as ActixResult, web};
use tracing::{error, info, warn};

use super::models::VerifyEmailRequest;

/// Email verification endpoint
pub async fn verify_email(
    state: web::Data<AppState>,
    request: web::Json<VerifyEmailRequest>,
) -> ActixResult<HttpResponse> {
    info!("Email verification with token");

    // Verify email token
    match state
        .auth
        .jwt()
        .verify_email_verification_token(&request.token)
        .await
    {
        Ok(user_id) => {
            // Mark email as verified
            match state.storage.db().verify_user_email(user_id).await {
                Ok(()) => {
                    info!("Email verified successfully for user: {}", user_id);
                    Ok(HttpResponse::Ok().json(ApiResponse::success(())))
                }
                Err(e) => {
                    error!("Failed to verify email: {}", e);
                    Ok(
                        HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                            "Internal server error".to_string(),
                        )),
                    )
                }
            }
        }
        Err(e) => {
            warn!("Invalid email verification token: {}", e);
            Ok(HttpResponse::Ok().json(ApiResponse::<()>::error_for_type(
                "Invalid or expired verification token".to_string(),
            )))
        }
    }
}
