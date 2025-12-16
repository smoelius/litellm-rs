//! Session management endpoints

use crate::server::AppState;
use crate::server::routes::ApiResponse;
use actix_web::http::header::HeaderMap;
use actix_web::{HttpRequest, HttpResponse, Result as ActixResult, web};
use tracing::{info, warn};

/// User logout endpoint
pub async fn logout(state: web::Data<AppState>, req: HttpRequest) -> ActixResult<HttpResponse> {
    info!("User logout");

    // Extract session token from headers or cookies
    if let Some(session_token) = extract_session_token(req.headers()) {
        if let Err(e) = state.auth.logout(&session_token).await {
            warn!("Failed to logout user: {}", e);
        }
    }

    Ok(HttpResponse::Ok().json(ApiResponse::success(())))
}

/// Extract session token from headers
pub fn extract_session_token(headers: &HeaderMap) -> Option<String> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http::header::{HeaderName, HeaderValue};

    #[test]
    fn test_extract_session_token() {
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
