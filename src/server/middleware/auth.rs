//! Authentication middleware

use crate::auth::AuthMethod;
use crate::core::models::RequestContext;
use crate::server::middleware::auth_rate_limiter::get_auth_rate_limiter;
use crate::server::middleware::helpers::{extract_auth_method, is_public_route};
use crate::server::AppState;
use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::{web, HttpMessage, HttpRequest};
use futures::future::{ready, Ready};
use std::future::Future;
use std::pin::Pin;
use tracing::{debug, warn};

/// Auth middleware for Actix-web
pub struct AuthMiddleware;

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type InitError = ();
    type Transform = AuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService { service }))
    }
}

/// Service implementation for auth middleware
pub struct AuthMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let path = req.path().to_string();

        if is_public_route(&path) {
            return Box::pin(self.service.call(req));
        }

        let auth_method = extract_auth_method(req.headers());
        let client_id = get_client_identifier(&req);
        let rate_limiter = get_auth_rate_limiter();

        if let Err(wait_seconds) = rate_limiter.check_allowed(&client_id) {
            return Box::pin(async move {
                Err(actix_web::error::ErrorTooManyRequests(format!(
                    "Too many failed attempts. Try again in {} seconds",
                    wait_seconds
                )))
            });
        }

        let app_state = req.app_data::<web::Data<AppState>>().cloned();
        let fut = self.service.call(req);

        Box::pin(async move {
            if let Some(state) = &app_state {
                match &auth_method {
                    AuthMethod::ApiKey(_key) => {
                        // API key validation is handled by the auth system
                        // For now, just log and allow - actual validation happens in routes
                        debug!("API key authentication attempt");
                        rate_limiter.record_success(&client_id);
                    }
                    AuthMethod::Jwt(token) => {
                        // Verify JWT token
                        match state.auth.jwt().verify_token(token).await {
                            Ok(_claims) => {
                                rate_limiter.record_success(&client_id);
                                debug!("JWT validated successfully");
                            }
                            Err(e) => {
                                rate_limiter.record_failure(&client_id);
                                warn!("JWT validation error: {}", e);
                            }
                        }
                    }
                    AuthMethod::None => {
                        debug!("No auth method provided for protected route");
                    }
                    _ => {}
                }
            }

            fut.await
        })
    }
}

/// Extract request context from request
pub fn get_request_context(req: &HttpRequest) -> Result<RequestContext, actix_web::Error> {
    req.extensions()
        .get::<RequestContext>()
        .cloned()
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("Missing request context"))
}

/// Extract a client identifier for rate limiting
fn get_client_identifier(req: &ServiceRequest) -> String {
    let ip = req
        .connection_info()
        .peer_addr()
        .map(|s| s.to_string())
        .unwrap_or_else(|| "unknown".to_string());

    if let Some(api_key) = req
        .headers()
        .get("x-api-key")
        .or_else(|| req.headers().get("authorization"))
        .and_then(|h| h.to_str().ok())
    {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        api_key.hash(&mut hasher);
        format!("{}:{:x}", ip, hasher.finish())
    } else {
        format!("ip:{}", ip)
    }
}
