//! Rate limiting middleware

use crate::server::state::AppState;
use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::web;
use futures::future::{ready, Ready};
use std::future::Future;
use std::pin::Pin;
use std::time::Instant;
use tracing::{debug, info};

/// Rate limit middleware for Actix-web
pub struct RateLimitMiddleware {
    requests_per_minute: u32,
}

impl RateLimitMiddleware {
    pub fn new(requests_per_minute: u32) -> Self {
        Self {
            requests_per_minute,
        }
    }
}

impl Default for RateLimitMiddleware {
    fn default() -> Self {
        Self::new(60)
    }
}

impl<S, B> Transform<S, ServiceRequest> for RateLimitMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type InitError = ();
    type Transform = RateLimitMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RateLimitMiddlewareService {
            service,
            requests_per_minute: self.requests_per_minute,
        }))
    }
}

/// Service implementation for rate limit middleware
pub struct RateLimitMiddlewareService<S> {
    service: S,
    requests_per_minute: u32,
}

impl<S, B> Service<ServiceRequest> for RateLimitMiddlewareService<S>
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
        let app_state = req.app_data::<web::Data<AppState>>().cloned();
        let _requests_per_minute = self.requests_per_minute;
        let start_time = Instant::now();
        let path = req.path().to_string();
        let method = req.method().to_string();

        let fut = self.service.call(req);

        Box::pin(async move {
            if let Some(_state) = &app_state {
                // Rate limiting logic would go here
                // For now, just log and pass through
                debug!(
                    "Rate limit check for {} {} - start: {:?}",
                    method, path, start_time
                );
            }

            let res = fut.await?;

            let duration = start_time.elapsed();
            info!(
                "{} {} completed in {:?} with status {}",
                method,
                path,
                duration,
                res.status()
            );

            Ok(res)
        })
    }
}
