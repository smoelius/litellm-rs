//! User login endpoint

use crate::server::state::AppState;
use crate::server::routes::ApiResponse;
use crate::utils::auth::crypto::password::verify_password;
use actix_web::{HttpResponse, Result as ActixResult, web};
use tracing::{error, info, warn};

use super::models::{LoginRequest, LoginResponse, UserInfo};

/// User login endpoint
pub async fn login(
    state: web::Data<AppState>,
    request: web::Json<LoginRequest>,
) -> ActixResult<HttpResponse> {
    info!("User login attempt: {}", request.username);

    // Find user by username
    let user = match state
        .storage
        .database
        .find_user_by_username(&request.username)
        .await
    {
        Ok(Some(user)) => user,
        Ok(None) => {
            warn!("Login attempt with invalid username: {}", request.username);
            return Ok(HttpResponse::Unauthorized()
                .json(ApiResponse::<()>::error("Invalid credentials".to_string())));
        }
        Err(e) => {
            error!("Database error during login: {}", e);
            return Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Database error".to_string())));
        }
    };

    // Check if user is active
    if !user.is_active() {
        warn!("Login attempt for inactive user: {}", request.username);
        return Ok(HttpResponse::Forbidden()
            .json(ApiResponse::<()>::error("Account is disabled".to_string())));
    }

    // Verify password
    let password_valid =
        match verify_password(&request.password, &user.password_hash) {
            Ok(valid) => valid,
            Err(e) => {
                error!("Password verification error: {}", e);
                return Ok(HttpResponse::InternalServerError()
                    .json(ApiResponse::<()>::error("Authentication error".to_string())));
            }
        };

    if !password_valid {
        warn!(
            "Login attempt with invalid password for user: {}",
            request.username
        );
        return Ok(HttpResponse::Unauthorized()
            .json(ApiResponse::<()>::error("Invalid credentials".to_string())));
    }

    // Update last login time
    if let Err(e) = state
        .storage
        .database
        .update_user_last_login(user.id())
        .await
    {
        warn!("Failed to update last login time: {}", e);
    }

    // Generate JWT tokens
    let access_token = match state
        .auth
        .jwt()
        .create_access_token(user.id(), user.role.to_string(), vec![], None, None)
        .await
    {
        Ok(token) => token,
        Err(e) => {
            error!("Failed to generate access token: {}", e);
            return Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                    "Token generation failed".to_string(),
                )),
            );
        }
    };

    let refresh_token = match state.auth.jwt().create_refresh_token(user.id(), None).await {
        Ok(token) => token,
        Err(e) => {
            error!("Failed to generate refresh token: {}", e);
            return Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                    "Token generation failed".to_string(),
                )),
            );
        }
    };

    info!("User logged in successfully: {}", user.username);

    let response = LoginResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: 3600, // 1 hour
        user: UserInfo {
            id: user.id(),
            username: user.username,
            email: user.email,
            full_name: user.display_name,
            role: user.role.to_string(),
            email_verified: user.email_verified,
        },
    };

    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}
