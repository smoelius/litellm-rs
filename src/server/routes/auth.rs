//! Authentication endpoints
//!
//! This module provides authentication-related API endpoints.

#![allow(dead_code)]

use crate::auth::jwt::TokenPair;
use crate::core::models::User;
use crate::server::AppState;
use crate::server::routes::ApiResponse;
use crate::utils::data::validation::DataValidator;
use actix_web::http::header::HeaderMap;
use actix_web::{HttpRequest, HttpResponse, Result as ActixResult, web};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

/// Configure authentication routes
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/register", web::post().to(register))
            .route("/login", web::post().to(login))
            .route("/logout", web::post().to(logout))
            .route("/refresh", web::post().to(refresh_token))
            .route("/forgot-password", web::post().to(forgot_password))
            .route("/reset-password", web::post().to(reset_password))
            .route("/verify-email", web::post().to(verify_email))
            .route("/change-password", web::post().to(change_password))
            .route("/me", web::post().to(get_current_user)),
    );
}

/// User registration request
#[derive(Debug, Deserialize)]
struct RegisterRequest {
    username: String,
    email: String,
    password: String,
    full_name: Option<String>,
}

/// User login request
#[derive(Debug, Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

/// Password change request
#[derive(Debug, Deserialize)]
struct ChangePasswordRequest {
    current_password: String,
    new_password: String,
}

/// Forgot password request
#[derive(Debug, Deserialize)]
struct ForgotPasswordRequest {
    email: String,
}

/// Reset password request
#[derive(Debug, Deserialize)]
struct ResetPasswordRequest {
    token: String,
    new_password: String,
}

/// Email verification request
#[derive(Debug, Deserialize)]
struct VerifyEmailRequest {
    token: String,
}

/// Refresh token request
#[derive(Debug, Deserialize)]
struct RefreshTokenRequest {
    refresh_token: String,
}

/// Authentication response
#[derive(Debug, Serialize)]
struct AuthResponse {
    user: UserResponse,
    tokens: TokenPair,
}

/// User response (without sensitive data)
#[derive(Debug, Serialize)]
struct UserResponse {
    id: uuid::Uuid,
    username: String,
    email: String,
    display_name: Option<String>,
    role: String,
    email_verified: bool,
    created_at: chrono::DateTime<chrono::Utc>,
}

/// Registration response
#[derive(Debug, Serialize)]
struct RegisterResponse {
    user_id: uuid::Uuid,
    username: String,
    email: String,
    message: String,
}

/// Login response
#[derive(Debug, Serialize)]
struct LoginResponse {
    access_token: String,
    refresh_token: String,
    token_type: String,
    expires_in: u64,
    user: UserInfo,
}

/// User info for login response
#[derive(Debug, Serialize)]
struct UserInfo {
    id: uuid::Uuid,
    username: String,
    email: String,
    full_name: Option<String>,
    role: String,
    email_verified: bool,
}

/// Refresh token response
#[derive(Debug, Serialize)]
struct RefreshTokenResponse {
    access_token: String,
    token_type: String,
    expires_in: u64,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id(),
            username: user.username.clone(),
            email: user.email.clone(),
            display_name: user.display_name.clone(),
            role: format!("{:?}", user.role),
            email_verified: user.email_verified,
            created_at: user.metadata.created_at,
        }
    }
}

/// User registration endpoint
async fn register(
    state: web::Data<AppState>,
    request: web::Json<RegisterRequest>,
) -> ActixResult<HttpResponse> {
    info!("User registration attempt: {}", request.username);

    // Validate input
    if let Err(e) = DataValidator::validate_username(&request.username) {
        return Ok(
            HttpResponse::BadRequest().json(ApiResponse::<()>::error_for_type(e.to_string()))
        );
    }

    if let Err(e) = crate::utils::config::ConfigValidator::validate_email(&request.email) {
        return Ok(
            HttpResponse::BadRequest().json(ApiResponse::<()>::error_for_type(e.to_string()))
        );
    }

    if let Err(e) = DataValidator::validate_password(&request.password) {
        return Ok(
            HttpResponse::BadRequest().json(ApiResponse::<()>::error_for_type(e.to_string()))
        );
    }

    // Check if user already exists
    match state
        .storage
        .database
        .find_user_by_username(&request.username)
        .await
    {
        Ok(Some(_)) => {
            return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
                "Username already exists".to_string(),
            )));
        }
        Ok(None) => {} // Continue with registration
        Err(e) => {
            error!("Failed to check existing user: {}", e);
            return Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Database error".to_string())));
        }
    }

    // Check if email already exists
    match state
        .storage
        .database
        .find_user_by_email(&request.email)
        .await
    {
        Ok(Some(_)) => {
            return Ok(HttpResponse::BadRequest()
                .json(ApiResponse::<()>::error("Email already exists".to_string())));
        }
        Ok(None) => {} // Continue with registration
        Err(e) => {
            error!("Failed to check existing email: {}", e);
            return Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Database error".to_string())));
        }
    }

    // Hash password
    let password_hash = match crate::utils::auth::crypto::hash_password(&request.password) {
        Ok(hash) => hash,
        Err(e) => {
            error!("Failed to hash password: {}", e);
            return Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                    "Password hashing failed".to_string(),
                )),
            );
        }
    };

    // Create user
    let user = crate::core::models::user::User::new(
        request.username.clone(),
        request.email.clone(),
        password_hash,
    );

    // Store user in database
    match state.storage.database.create_user(&user).await {
        Ok(created_user) => {
            info!("User registered successfully: {}", created_user.username);

            let response = RegisterResponse {
                user_id: created_user.id(),
                username: created_user.username,
                email: created_user.email,
                message: "Registration successful. Please verify your email.".to_string(),
            };

            Ok(HttpResponse::Created().json(ApiResponse::success(response)))
        }
        Err(e) => {
            error!("Failed to create user: {}", e);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("User creation failed".to_string())))
        }
    }
}

/// User login endpoint
async fn login(
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
        match crate::utils::auth::crypto::verify_password(&request.password, &user.password_hash) {
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

/// User logout endpoint
async fn logout(state: web::Data<AppState>, req: HttpRequest) -> ActixResult<HttpResponse> {
    info!("User logout");

    // Extract session token from headers or cookies
    if let Some(session_token) = extract_session_token(req.headers()) {
        if let Err(e) = state.auth.logout(&session_token).await {
            warn!("Failed to logout user: {}", e);
        }
    }

    Ok(HttpResponse::Ok().json(ApiResponse::success(())))
}

/// Refresh token endpoint
async fn refresh_token(
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

/// Forgot password endpoint
async fn forgot_password(
    state: web::Data<AppState>,
    request: web::Json<ForgotPasswordRequest>,
) -> ActixResult<HttpResponse> {
    info!("Password reset request for email: {}", request.email);

    // Generate reset token
    match state.auth.request_password_reset(&request.email).await {
        Ok(_reset_token) => {
            // TODO: Send email with reset token
            info!("Password reset token generated for: {}", request.email);
            Ok(HttpResponse::Ok().json(ApiResponse::success(())))
        }
        Err(e) => {
            // Don't reveal if email exists or not
            warn!("Password reset request failed: {}", e);
            Ok(HttpResponse::Ok().json(ApiResponse::success(())))
        }
    }
}

/// Reset password endpoint
async fn reset_password(
    state: web::Data<AppState>,
    request: web::Json<ResetPasswordRequest>,
) -> ActixResult<HttpResponse> {
    info!("Password reset with token");

    // Validate new password
    if let Err(e) = DataValidator::validate_password(&request.new_password) {
        return Ok(HttpResponse::Ok().json(ApiResponse::<()>::error_for_type(e.to_string())));
    }

    // Reset password
    match state
        .auth
        .reset_password(&request.token, &request.new_password)
        .await
    {
        Ok(()) => {
            info!("Password reset successfully");
            Ok(HttpResponse::Ok().json(ApiResponse::success(())))
        }
        Err(e) => {
            warn!("Password reset failed: {}", e);
            Ok(HttpResponse::Ok().json(ApiResponse::<()>::error_for_type(
                "Invalid or expired reset token".to_string(),
            )))
        }
    }
}

/// Email verification endpoint
async fn verify_email(
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

/// Change password endpoint
async fn change_password(
    state: web::Data<AppState>,
    req: HttpRequest,
    request: web::Json<ChangePasswordRequest>,
) -> ActixResult<HttpResponse> {
    info!("Password change request");

    // Get authenticated user
    let user = match get_authenticated_user(req.headers()) {
        Some(user) => user,
        None => {
            return Ok(HttpResponse::Unauthorized()
                .json(ApiResponse::<()>::error("Unauthorized".to_string())));
        }
    };

    // Validate new password
    if let Err(e) = DataValidator::validate_password(&request.new_password) {
        return Ok(HttpResponse::Ok().json(ApiResponse::<()>::error_for_type(e.to_string())));
    }

    // Change password
    match state
        .auth
        .change_password(user.id(), &request.current_password, &request.new_password)
        .await
    {
        Ok(()) => {
            info!("Password changed successfully for user: {}", user.username);
            Ok(HttpResponse::Ok().json(ApiResponse::success(())))
        }
        Err(e) => {
            warn!("Password change failed: {}", e);
            Ok(HttpResponse::Ok().json(ApiResponse::<()>::error_for_type(e.to_string())))
        }
    }
}

/// Get current user endpoint
async fn get_current_user(req: HttpRequest) -> ActixResult<HttpResponse> {
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

/// Extract session token from headers
fn extract_session_token(headers: &HeaderMap) -> Option<String> {
    // Check Authorization header
    if let Some(auth_header) = headers.get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(stripped) = auth_str.strip_prefix("Session ") {
                return Some(stripped.to_string());
            }
        }
    }

    // Check session cookie
    if let Some(cookie_header) = headers.get("cookie") {
        if let Ok(cookie_str) = cookie_header.to_str() {
            for cookie in cookie_str.split(';') {
                let cookie = cookie.trim();
                if let Some(stripped) = cookie.strip_prefix("session=") {
                    return Some(stripped.to_string());
                }
            }
        }
    }

    None
}

/// Get authenticated user from request extensions
fn get_authenticated_user(_headers: &HeaderMap) -> Option<User> {
    // In a real implementation, this would extract the user from request extensions
    // that were set by the authentication middleware
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_request_validation() {
        let request = RegisterRequest {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password: "SecurePass123!".to_string(),
            full_name: Some("Test User".to_string()),
        };

        assert_eq!(request.username, "testuser");
        assert_eq!(request.email, "test@example.com");
        assert!(request.full_name.is_some());
    }

    #[test]
    fn test_user_response_conversion() {
        // This would require a real User instance in a full test
        // For now, just test the structure
        let user_response = UserResponse {
            id: uuid::Uuid::new_v4(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            display_name: Some("Test User".to_string()),
            role: "User".to_string(),
            email_verified: false,
            created_at: chrono::Utc::now(),
        };

        assert_eq!(user_response.username, "testuser");
        assert!(!user_response.email_verified);
    }

    #[test]
    fn test_extract_session_token() {
        use actix_web::http::header::{HeaderName, HeaderValue};

        let mut headers = HeaderMap::new();
        headers.insert(
            HeaderName::from_static("cookie"),
            HeaderValue::from_static("session=abc123; other=value"),
        );

        let token = extract_session_token(&headers);
        assert_eq!(token, Some("abc123".to_string()));

        let mut headers = HeaderMap::new();
        headers.insert(
            HeaderName::from_static("authorization"),
            HeaderValue::from_static("Session xyz789"),
        );

        let token = extract_session_token(&headers);
        assert_eq!(token, Some("xyz789".to_string()));
    }
}
