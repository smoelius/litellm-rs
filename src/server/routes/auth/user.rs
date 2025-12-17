//! Current user endpoint and helpers

use crate::core::models::user::types::User;
use crate::server::routes::ApiResponse;
use actix_web::http::header::HeaderMap;
use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};
use tracing::debug;

/// Get current user endpoint
pub async fn get_current_user(req: HttpRequest) -> ActixResult<HttpResponse> {
    debug!("Get current user request");

    // Get authenticated user
    let user = match get_authenticated_user(req.headers()) {
        Some(user) => user,
        None => {
            return Ok(HttpResponse::Unauthorized()
                .json(ApiResponse::<()>::error("Unauthorized".to_string())));
        }
    };

    Ok(HttpResponse::Ok().json(ApiResponse::success(user)))
}

/// Get authenticated user from request extensions
pub fn get_authenticated_user(_headers: &HeaderMap) -> Option<User> {
    // In a real implementation, this would extract the user from request extensions
    // that were set by the authentication middleware
    None
}
