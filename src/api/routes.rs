//! API route configuration.
//!
//! Defines protected (authenticated) and public endpoint groups.

use crate::api::handlers::{
    domain_list_handler, health_handler, shorten_handler, stats_handler, stats_list_handler,
};
use crate::state::AppState;
use axum::{
    Router,
    routing::{get, post},
};

/// Routes requiring API token authentication.
///
/// Protected via [`crate::api::middleware::auth`].
///
/// # Endpoints
///
/// - `GET /health` - Service health status
/// - `GET /domains` - List configured domains
/// - `GET /stats` - Aggregated click statistics
/// - `GET /stats/{code}` - Detailed statistics for a specific link
pub fn protected_routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health_handler))
        .route("/domains", get(domain_list_handler))
        .route("/stats", get(stats_list_handler))
        .route("/stats/{code}", get(stats_handler))
}

/// Publicly accessible routes without authentication.
///
/// # Endpoints
///
/// - `POST /shorten` - Create short links (batch-capable)
pub fn public_routes() -> Router<AppState> {
    Router::new().route("/shorten", post(shorten_handler))
}
