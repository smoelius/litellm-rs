//! HTTP route handlers
//!
//! This module provides HTTP route handler functions.

use actix_web::HttpResponse;
use serde_json::json;

/// Health check endpoint handler
pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION")
    }))
}
