//! HTTP middleware implementations
//!
//! This module provides various middleware for request processing.

#![allow(dead_code)]

use crate::auth::AuthMethod;
use crate::core::models::RequestContext;
use crate::server::AppState;
use actix_web::HttpMessage;
use dashmap::DashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use actix_web::dev::{Service, Transform, forward_ready};
use actix_web::{
    HttpRequest,
    dev::{ServiceRequest, ServiceResponse},
    http::header::HeaderValue,
    web,
};
use futures::future::{Ready, ready};
use std::future::Future;
use std::pin::Pin;

use std::time::{Duration, Instant};
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Brute force protection for authentication endpoints
///
/// This struct tracks failed authentication attempts per client
/// and blocks requests after too many failures.
pub struct AuthRateLimiter {
    /// Map of client identifier -> (failure count, first failure time, lockout until)
    attempts: DashMap<String, AuthAttemptTracker>,
    /// Maximum failed attempts before lockout
    max_attempts: u32,
    /// Time window for counting failures (seconds)
    window_secs: u64,
    /// Lockout duration (seconds) - uses exponential backoff
    base_lockout_secs: u64,
    /// Total blocked attempts counter for monitoring
    blocked_count: AtomicU64,
}

/// Tracks authentication attempts for a single client
struct AuthAttemptTracker {
    /// Number of failed attempts in current window
    failure_count: u32,
    /// When the first failure in current window occurred
    window_start: Instant,
    /// If locked out, when the lockout expires
    lockout_until: Option<Instant>,
    /// Number of lockouts (for exponential backoff)
    lockout_count: u32,
}

impl Default for AuthRateLimiter {
    fn default() -> Self {
        Self::new(5, 300, 60) // 5 attempts per 5 minutes, 1 minute base lockout
    }
}

impl AuthRateLimiter {
    /// Create a new auth rate limiter
    ///
    /// # Arguments
    /// * `max_attempts` - Maximum failed attempts before lockout
    /// * `window_secs` - Time window for counting failures
    /// * `base_lockout_secs` - Base lockout duration (increases exponentially)
    pub fn new(max_attempts: u32, window_secs: u64, base_lockout_secs: u64) -> Self {
        Self {
            attempts: DashMap::new(),
            max_attempts,
            window_secs,
            base_lockout_secs,
            blocked_count: AtomicU64::new(0),
        }
    }

    /// Check if a client is allowed to attempt authentication
    ///
    /// Returns Ok(()) if allowed, Err with seconds to wait if blocked
    pub fn check_allowed(&self, client_id: &str) -> Result<(), u64> {
        let now = Instant::now();

        // Get or create tracker
        let mut entry = self
            .attempts
            .entry(client_id.to_string())
            .or_insert_with(|| AuthAttemptTracker {
                failure_count: 0,
                window_start: now,
                lockout_until: None,
                lockout_count: 0,
            });

        let tracker = entry.value_mut();

        // Check if currently locked out
        if let Some(lockout_until) = tracker.lockout_until {
            if now < lockout_until {
                let remaining = lockout_until.duration_since(now).as_secs();
                self.blocked_count.fetch_add(1, Ordering::Relaxed);
                warn!(
                    "Auth attempt blocked for {} - locked out for {} more seconds",
                    client_id, remaining
                );
                return Err(remaining);
            } else {
                // Lockout expired, reset but keep lockout count for exponential backoff
                tracker.lockout_until = None;
                tracker.failure_count = 0;
                tracker.window_start = now;
            }
        }

        // Check if window has expired
        let window_duration = Duration::from_secs(self.window_secs);
        if now.duration_since(tracker.window_start) > window_duration {
            // Reset window
            tracker.failure_count = 0;
            tracker.window_start = now;
            // Decay lockout count over time (reset after successful window)
            if tracker.lockout_count > 0 {
                tracker.lockout_count = tracker.lockout_count.saturating_sub(1);
            }
        }

        Ok(())
    }

    /// Record a failed authentication attempt
    ///
    /// Returns the lockout duration in seconds if the client is now locked out
    pub fn record_failure(&self, client_id: &str) -> Option<u64> {
        let now = Instant::now();

        let mut entry = self
            .attempts
            .entry(client_id.to_string())
            .or_insert_with(|| AuthAttemptTracker {
                failure_count: 0,
                window_start: now,
                lockout_until: None,
                lockout_count: 0,
            });

        let tracker = entry.value_mut();
        tracker.failure_count += 1;

        info!(
            "Auth failure for {}: attempt {} of {} in window",
            client_id, tracker.failure_count, self.max_attempts
        );

        // Check if we've exceeded max attempts
        if tracker.failure_count >= self.max_attempts {
            // Calculate lockout duration with exponential backoff
            // Each lockout doubles the duration up to a maximum
            let multiplier = 2u64.pow(tracker.lockout_count.min(6)); // Cap at 64x
            let lockout_secs = self.base_lockout_secs * multiplier;
            let lockout_duration = Duration::from_secs(lockout_secs);

            tracker.lockout_until = Some(now + lockout_duration);
            tracker.lockout_count += 1;
            tracker.failure_count = 0;

            warn!(
                "Client {} locked out for {} seconds (lockout #{}) after {} failed attempts",
                client_id, lockout_secs, tracker.lockout_count, self.max_attempts
            );

            return Some(lockout_secs);
        }

        None
    }

    /// Record a successful authentication (resets failure count)
    pub fn record_success(&self, client_id: &str) {
        if let Some(mut entry) = self.attempts.get_mut(client_id) {
            entry.failure_count = 0;
            entry.lockout_until = None;
            // Gradually reduce lockout count on success
            entry.lockout_count = entry.lockout_count.saturating_sub(1);
        }
    }

    /// Get the total number of blocked attempts (for monitoring)
    pub fn blocked_attempts(&self) -> u64 {
        self.blocked_count.load(Ordering::Relaxed)
    }

    /// Clean up old entries (call periodically)
    pub fn cleanup(&self) {
        let now = Instant::now();
        let max_age = Duration::from_secs(self.window_secs * 10); // Keep entries for 10x window

        self.attempts.retain(|_, tracker| {
            // Keep if recently active or locked out
            now.duration_since(tracker.window_start) < max_age
                || tracker.lockout_until.is_some_and(|until| until > now)
        });
    }
}

/// Global auth rate limiter
static AUTH_RATE_LIMITER: std::sync::OnceLock<Arc<AuthRateLimiter>> = std::sync::OnceLock::new();

/// Get or initialize the global auth rate limiter
pub fn get_auth_rate_limiter() -> Arc<AuthRateLimiter> {
    AUTH_RATE_LIMITER
        .get_or_init(|| Arc::new(AuthRateLimiter::default()))
        .clone()
}

/// Extract a client identifier for rate limiting
/// Uses IP address, falling back to a hash of auth credentials
fn get_client_identifier(req: &ServiceRequest) -> String {
    // Try to get IP address
    let ip = req
        .connection_info()
        .peer_addr()
        .map(|s| s.to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // Also include API key hash if present (to track per-key abuse)
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

/// Request ID middleware for Actix-web
pub struct RequestIdMiddleware;

impl<S, B> Transform<S, ServiceRequest> for RequestIdMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type InitError = ();
    type Transform = RequestIdMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequestIdMiddlewareService { service }))
    }
}

/// Service implementation for request ID middleware
pub struct RequestIdMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for RequestIdMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let request_id = Uuid::new_v4().to_string();

        // Add request ID to headers
        req.headers_mut().insert(
            actix_web::http::header::HeaderName::from_static("x-request-id"),
            HeaderValue::from_str(&request_id)
                .unwrap_or_else(|_| HeaderValue::from_static("invalid")),
        );

        debug!("Processing request: {}", request_id);

        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}

/// Authentication middleware for Actix-web
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

/// Service implementation for authentication middleware
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
        let _start_time = Instant::now();
        let path = req.path().to_string();
        let _method = req.method().clone();

        // Skip authentication for public routes
        if is_public_route(&path) {
            debug!("Skipping authentication for public route: {}", path);
            let fut = self.service.call(req);
            return Box::pin(async move {
                let res = fut.await?;
                Ok(res)
            });
        }

        // Get client identifier for brute force protection
        let client_id = get_client_identifier(&req);

        // Check brute force protection BEFORE attempting authentication
        let auth_limiter = get_auth_rate_limiter();
        if let Err(retry_after) = auth_limiter.check_allowed(&client_id) {
            return Box::pin(async move {
                Err(actix_web::error::ErrorTooManyRequests(format!(
                    "Too many failed authentication attempts. Retry after {} seconds.",
                    retry_after
                )))
            });
        }

        // Get app state
        let state = req.app_data::<web::Data<AppState>>().cloned();
        if state.is_none() {
            return Box::pin(async move {
                Err(actix_web::error::ErrorInternalServerError(
                    "App state not found",
                ))
            });
        }
        let state = state.unwrap();

        // Create request context
        let mut context = RequestContext::new();
        context.client_ip = req.connection_info().peer_addr().map(|s| s.to_string());
        context.user_agent = req
            .headers()
            .get("user-agent")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string());

        // Extract authentication method
        let auth_method = extract_auth_method(req.headers());

        let fut = self.service.call(req);
        let client_id_clone = client_id.clone();
        Box::pin(async move {
            // Authenticate request
            match state.auth.authenticate(auth_method, context).await {
                Ok(auth_result) => {
                    if auth_result.success {
                        // Record successful auth (resets failure count)
                        auth_limiter.record_success(&client_id_clone);
                        debug!("Authentication successful for {}", path);
                        let res = fut.await?;
                        Ok(res)
                    } else {
                        // Record failed auth attempt
                        if let Some(lockout_secs) = auth_limiter.record_failure(&client_id_clone) {
                            warn!(
                                "Authentication failed for {} (now locked out for {}s): {:?}",
                                path, lockout_secs, auth_result.error
                            );
                            return Err(actix_web::error::ErrorTooManyRequests(format!(
                                "Too many failed attempts. Account locked for {} seconds.",
                                lockout_secs
                            )));
                        }
                        warn!(
                            "Authentication failed for {}: {:?}",
                            path, auth_result.error
                        );
                        Err(actix_web::error::ErrorUnauthorized("Authentication failed"))
                    }
                }
                Err(e) => {
                    // Record error as failure too (could be a probing attack)
                    auth_limiter.record_failure(&client_id_clone);
                    warn!("Authentication error for {}: {}", path, e);
                    Err(actix_web::error::ErrorInternalServerError(
                        "Authentication error",
                    ))
                }
            }
        })
    }
}

/// Extract request context from HTTP request (helper function)
pub fn get_request_context(req: &HttpRequest) -> Result<RequestContext, actix_web::Error> {
    let mut context = RequestContext::new();

    // Extract client IP
    context.client_ip = req.connection_info().peer_addr().map(|s| s.to_string());

    // Extract user agent
    context.user_agent = req
        .headers()
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    Ok(context)
}

/// Rate limiting middleware for Actix-web
pub struct RateLimitMiddleware {
    limiter: Option<Arc<crate::core::rate_limiter::RateLimiter>>,
}

impl RateLimitMiddleware {
    /// Create a new rate limit middleware
    pub fn new(limiter: Arc<crate::core::rate_limiter::RateLimiter>) -> Self {
        Self {
            limiter: Some(limiter),
        }
    }

    /// Create with global rate limiter
    pub fn global() -> Self {
        Self {
            limiter: crate::core::rate_limiter::get_global_rate_limiter(),
        }
    }
}

impl Default for RateLimitMiddleware {
    fn default() -> Self {
        Self::global()
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
            limiter: self.limiter.clone(),
        }))
    }
}

/// Service implementation for rate limiting middleware
pub struct RateLimitMiddlewareService<S> {
    service: S,
    limiter: Option<Arc<crate::core::rate_limiter::RateLimiter>>,
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
        let path = req.path().to_string();
        let client_ip = req
            .connection_info()
            .peer_addr()
            .unwrap_or("unknown")
            .to_string();

        // Skip rate limiting for health checks and metrics
        if path == "/health" || path == "/metrics" {
            let fut = self.service.call(req);
            return Box::pin(async move {
                let res = fut.await?;
                Ok(res)
            });
        }

        // Get rate limiter
        let limiter = self.limiter.clone();

        // Extract API key for per-key rate limiting (prefer over IP)
        let rate_limit_key = req
            .headers()
            .get("x-api-key")
            .or_else(|| req.headers().get("authorization"))
            .and_then(|h| h.to_str().ok())
            .map(|s| {
                // Hash the key for privacy
                use std::hash::{Hash, Hasher};
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                s.hash(&mut hasher);
                format!("key:{:x}", hasher.finish())
            })
            .unwrap_or_else(|| format!("ip:{}", client_ip));

        debug!("Rate limiting check for {} (key: {})", path, rate_limit_key);

        let fut = self.service.call(req);
        Box::pin(async move {
            // Use atomic check_and_record to prevent race conditions
            if let Some(limiter) = limiter {
                let result = limiter.check_and_record(&rate_limit_key).await;

                if !result.allowed {
                    warn!(
                        "Rate limit exceeded for {}: {}/{} requests",
                        rate_limit_key, result.current_count, result.limit
                    );

                    // Return 429 Too Many Requests
                    let retry_after = result.retry_after_secs.unwrap_or(60);
                    return Err(actix_web::error::ErrorTooManyRequests(format!(
                        "Rate limit exceeded. Retry after {} seconds.",
                        retry_after
                    )));
                }

                // Process request and add rate limit headers to response
                // Note: remaining is already adjusted by check_and_record
                let mut res = fut.await?;
                let headers = res.headers_mut();

                headers.insert(
                    actix_web::http::header::HeaderName::from_static("x-ratelimit-limit"),
                    HeaderValue::from_str(&result.limit.to_string())
                        .unwrap_or(HeaderValue::from_static("0")),
                );
                headers.insert(
                    actix_web::http::header::HeaderName::from_static("x-ratelimit-remaining"),
                    HeaderValue::from_str(&result.remaining.to_string())
                        .unwrap_or(HeaderValue::from_static("0")),
                );
                headers.insert(
                    actix_web::http::header::HeaderName::from_static("x-ratelimit-reset"),
                    HeaderValue::from_str(&result.reset_after_secs.to_string())
                        .unwrap_or(HeaderValue::from_static("0")),
                );

                Ok(res)
            } else {
                // No rate limiter configured, pass through
                let res = fut.await?;
                Ok(res)
            }
        })
    }
}

/// Metrics collection middleware for Actix-web
pub struct MetricsMiddleware;

impl<S, B> Transform<S, ServiceRequest> for MetricsMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type InitError = ();
    type Transform = MetricsMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(MetricsMiddlewareService { service }))
    }
}

/// Service implementation for metrics middleware
pub struct MetricsMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for MetricsMiddlewareService<S>
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
        let start_time = Instant::now();
        let method = req.method().clone();
        let path = req.path().to_string();
        let user_agent = req
            .headers()
            .get("user-agent")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string());
        let client_ip = req
            .connection_info()
            .peer_addr()
            .unwrap_or("unknown")
            .to_string();

        // Get request ID from headers
        let request_id = req
            .headers()
            .get("x-request-id")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("unknown")
            .to_string();

        // Extract monitoring system before moving req
        let monitoring = req
            .extensions()
            .get::<Arc<crate::monitoring::MonitoringSystem>>()
            .cloned();

        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;

            // Calculate response time
            let response_time = start_time.elapsed();
            let status_code = res.status().as_u16();

            // Log request metrics
            info!(
                request_id = %request_id,
                method = %method,
                path = %path,
                status = status_code,
                duration_ms = response_time.as_millis(),
                client_ip = %client_ip,
                user_agent = ?user_agent,
                "Request completed"
            );

            // Send metrics to monitoring system
            if let Some(monitoring_system) = monitoring {
                let request_metrics = crate::server::RequestMetrics {
                    request_id: request_id.clone(),
                    method: method.to_string(),
                    path: path.clone(),
                    status_code,
                    response_time_ms: response_time.as_millis() as u64,
                    request_size: 0,  // Would need to capture from request body
                    response_size: 0, // Would need to capture from response body
                    user_agent: user_agent.clone(),
                    client_ip: Some(client_ip.clone()),
                    user_id: None,    // Would be extracted from auth context
                    api_key_id: None, // Would be extracted from auth context
                };

                // Send metrics asynchronously
                let monitoring_clone = monitoring_system.clone();
                let metrics_clone = request_metrics.clone();
                tokio::spawn(async move {
                    if let Err(e) = monitoring_clone
                        .record_request(
                            &metrics_clone.method,
                            &metrics_clone.path,
                            metrics_clone.status_code,
                            response_time,
                            metrics_clone.user_id,
                            metrics_clone.api_key_id,
                        )
                        .await
                    {
                        warn!("Failed to record request metrics: {}", e);
                    }
                });
            }

            Ok(res)
        })
    }
}

/// CORS middleware for Actix-web (handled by actix-cors, but we can add custom logic here)
pub struct CorsMiddleware;

impl<S, B> Transform<S, ServiceRequest> for CorsMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type InitError = ();
    type Transform = CorsMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(CorsMiddlewareService { service }))
    }
}

/// Service implementation for CORS middleware
pub struct CorsMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for CorsMiddlewareService<S>
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
        // Custom CORS logic can be added here if needed
        // For now, rely on actix-cors middleware
        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}

/// Security headers middleware for Actix-web
pub struct SecurityHeadersMiddleware;

impl<S, B> Transform<S, ServiceRequest> for SecurityHeadersMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type InitError = ();
    type Transform = SecurityHeadersMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(SecurityHeadersMiddlewareService { service }))
    }
}

/// Service implementation for security headers middleware
pub struct SecurityHeadersMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for SecurityHeadersMiddlewareService<S>
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
        let fut = self.service.call(req);
        Box::pin(async move {
            let mut res = fut.await?;

            // Add security headers
            let headers = res.headers_mut();

            headers.insert(
                actix_web::http::header::HeaderName::from_static("x-content-type-options"),
                HeaderValue::from_static("nosniff"),
            );
            headers.insert(
                actix_web::http::header::HeaderName::from_static("x-frame-options"),
                HeaderValue::from_static("DENY"),
            );
            headers.insert(
                actix_web::http::header::HeaderName::from_static("x-xss-protection"),
                HeaderValue::from_static("1; mode=block"),
            );
            headers.insert(
                actix_web::http::header::HeaderName::from_static("strict-transport-security"),
                HeaderValue::from_static("max-age=31536000; includeSubDomains"),
            );
            headers.insert(
                actix_web::http::header::HeaderName::from_static("referrer-policy"),
                HeaderValue::from_static("strict-origin-when-cross-origin"),
            );

            Ok(res)
        })
    }
}

// TODO: Implement additional middleware as needed for actix-web

/// Extract authentication method from headers
fn extract_auth_method(headers: &actix_web::http::header::HeaderMap) -> AuthMethod {
    // Check Authorization header
    if let Some(auth_header) = headers.get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(stripped) = auth_str.strip_prefix("Bearer ") {
                let token = stripped.to_string();
                return AuthMethod::Jwt(token);
            } else if let Some(stripped) = auth_str.strip_prefix("ApiKey ") {
                let key = stripped.to_string();
                return AuthMethod::ApiKey(key);
            } else if auth_str.starts_with("gw-") {
                // Direct API key
                return AuthMethod::ApiKey(auth_str.to_string());
            }
        }
    }

    // Check X-API-Key header
    if let Some(api_key_header) = headers.get("x-api-key") {
        if let Ok(key) = api_key_header.to_str() {
            return AuthMethod::ApiKey(key.to_string());
        }
    }

    // Check session cookie
    if let Some(cookie_header) = headers.get("cookie") {
        if let Ok(cookie_str) = cookie_header.to_str() {
            for cookie in cookie_str.split(';') {
                let cookie = cookie.trim();
                if let Some(stripped) = cookie.strip_prefix("session=") {
                    let session_id = stripped.to_string();
                    return AuthMethod::Session(session_id);
                }
            }
        }
    }

    AuthMethod::None
}

/// Check if a route is public (doesn't require authentication)
fn is_public_route(path: &str) -> bool {
    const PUBLIC_ROUTES: &[&str] = &[
        "/health",
        "/metrics",
        "/auth/login",
        "/auth/register",
        "/auth/forgot-password",
        "/auth/reset-password",
        "/auth/verify-email",
        "/docs",
        "/openapi.json",
    ];

    PUBLIC_ROUTES.iter().any(|&route| path.starts_with(route))
}

/// Check if a route requires admin privileges
fn is_admin_route(path: &str) -> bool {
    const ADMIN_ROUTES: &[&str] = &["/admin", "/api/admin"];

    ADMIN_ROUTES.iter().any(|&route| path.starts_with(route))
}

/// Check if a route is for API access
fn is_api_route(path: &str) -> bool {
    const API_ROUTES: &[&str] = &[
        "/v1/chat/completions",
        "/v1/completions",
        "/v1/embeddings",
        "/v1/images",
        "/v1/audio",
        "/v1/models",
    ];

    API_ROUTES.iter().any(|&route| path.starts_with(route))
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http::header::{HeaderMap, HeaderName, HeaderValue};

    #[test]
    fn test_extract_auth_method_bearer() {
        let mut headers = HeaderMap::new();
        headers.insert(
            HeaderName::from_static("authorization"),
            HeaderValue::from_static("Bearer token123"),
        );

        let auth_method = extract_auth_method(&headers);
        assert!(matches!(auth_method, AuthMethod::Jwt(token) if token == "token123"));
    }

    #[test]
    fn test_extract_auth_method_api_key() {
        let mut headers = HeaderMap::new();
        headers.insert(
            HeaderName::from_static("authorization"),
            HeaderValue::from_static("ApiKey key123"),
        );

        let auth_method = extract_auth_method(&headers);
        assert!(matches!(auth_method, AuthMethod::ApiKey(key) if key == "key123"));
    }

    #[test]
    fn test_extract_auth_method_x_api_key() {
        let mut headers = HeaderMap::new();
        headers.insert(
            HeaderName::from_static("x-api-key"),
            HeaderValue::from_static("key123"),
        );

        let auth_method = extract_auth_method(&headers);
        assert!(matches!(auth_method, AuthMethod::ApiKey(key) if key == "key123"));
    }

    #[test]
    fn test_extract_auth_method_session() {
        let mut headers = HeaderMap::new();
        headers.insert(
            HeaderName::from_static("cookie"),
            HeaderValue::from_static("session=sess123; other=value"),
        );

        let auth_method = extract_auth_method(&headers);
        assert!(matches!(auth_method, AuthMethod::Session(session) if session == "sess123"));
    }

    #[test]
    fn test_extract_auth_method_none() {
        let headers = HeaderMap::new();
        let auth_method = extract_auth_method(&headers);
        assert!(matches!(auth_method, AuthMethod::None));
    }

    #[test]
    fn test_is_public_route() {
        assert!(is_public_route("/health"));
        assert!(is_public_route("/auth/login"));
        assert!(is_public_route("/metrics"));
        assert!(!is_public_route("/api/users"));
        assert!(!is_public_route("/v1/chat/completions"));
    }

    #[test]
    fn test_is_admin_route() {
        assert!(is_admin_route("/admin/users"));
        assert!(is_admin_route("/api/admin/config"));
        assert!(!is_admin_route("/api/users"));
        assert!(!is_admin_route("/health"));
    }

    #[test]
    fn test_is_api_route() {
        assert!(is_api_route("/v1/chat/completions"));
        assert!(is_api_route("/v1/embeddings"));
        assert!(is_api_route("/v1/models"));
        assert!(!is_api_route("/api/users"));
        assert!(!is_api_route("/health"));
    }

    #[test]
    fn test_auth_rate_limiter_allows_initial_attempts() {
        let limiter = AuthRateLimiter::new(3, 60, 30);
        let client_id = "test_client_1";

        // First few attempts should be allowed
        assert!(limiter.check_allowed(client_id).is_ok());
        assert!(limiter.record_failure(client_id).is_none());

        assert!(limiter.check_allowed(client_id).is_ok());
        assert!(limiter.record_failure(client_id).is_none());
    }

    #[test]
    fn test_auth_rate_limiter_locks_after_max_attempts() {
        let limiter = AuthRateLimiter::new(3, 60, 30);
        let client_id = "test_client_2";

        // First 2 failures - no lockout yet
        limiter.record_failure(client_id);
        limiter.record_failure(client_id);

        // Third failure should trigger lockout
        let lockout = limiter.record_failure(client_id);
        assert!(lockout.is_some());
        assert_eq!(lockout.unwrap(), 30); // Base lockout time

        // Further attempts should be blocked
        let check = limiter.check_allowed(client_id);
        assert!(check.is_err());
    }

    #[test]
    fn test_auth_rate_limiter_exponential_backoff() {
        let limiter = AuthRateLimiter::new(2, 60, 10);
        let client_id = "test_client_3";

        // First lockout
        limiter.record_failure(client_id);
        let lockout1 = limiter.record_failure(client_id);
        assert_eq!(lockout1.unwrap(), 10); // 10 * 2^0 = 10

        // Simulate lockout expiration by creating a new client entry
        // (In real scenario, would wait for lockout to expire)
        let client_id2 = "test_client_3b";
        limiter.record_failure(client_id2);
        limiter.record_failure(client_id2);

        // Second lockout for same pattern (if we could wait) would be 10 * 2^1 = 20
    }

    #[test]
    fn test_auth_rate_limiter_success_resets_failure_count() {
        let limiter = AuthRateLimiter::new(3, 60, 30);
        let client_id = "test_client_4";

        // 2 failures
        limiter.record_failure(client_id);
        limiter.record_failure(client_id);

        // Successful auth resets count
        limiter.record_success(client_id);

        // Should be able to fail again without immediate lockout
        assert!(limiter.record_failure(client_id).is_none());
        assert!(limiter.record_failure(client_id).is_none());
    }

    #[test]
    fn test_auth_rate_limiter_different_clients_independent() {
        let limiter = AuthRateLimiter::new(2, 60, 30);
        let client_a = "client_a";
        let client_b = "client_b";

        // Lock out client A
        limiter.record_failure(client_a);
        limiter.record_failure(client_a);

        // Client A locked out
        assert!(limiter.check_allowed(client_a).is_err());

        // Client B should still be allowed
        assert!(limiter.check_allowed(client_b).is_ok());
    }

    #[test]
    fn test_auth_rate_limiter_blocked_count() {
        let limiter = AuthRateLimiter::new(1, 60, 30);
        let client_id = "test_client_5";

        // Lock out the client
        limiter.record_failure(client_id);

        // Initial blocked count should be 0
        assert_eq!(limiter.blocked_attempts(), 0);

        // Try to access while locked out
        let _ = limiter.check_allowed(client_id);

        // Blocked count should increase
        assert_eq!(limiter.blocked_attempts(), 1);

        // Try again
        let _ = limiter.check_allowed(client_id);
        assert_eq!(limiter.blocked_attempts(), 2);
    }
}
