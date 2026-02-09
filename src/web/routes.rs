//! Web dashboard route configuration.

use crate::state::AppState;
use crate::web::handlers::{dashboard_handler, links_handler, login_handler, stats_handler};
use axum::{Router, routing::get};

/// Protected dashboard routes requiring authentication.
///
/// Protected via [`crate::web::middleware::web_auth`] (cookie-based or similar).
///
/// # Endpoints
///
/// - `GET /` - Dashboard home with overview
/// - `GET /links` - Link management page
/// - `GET /stats/{code}` - Detailed statistics page for a specific link
pub fn protected_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(dashboard_handler))
        .route("/links", get(links_handler))
        .route("/stats/{code}", get(stats_handler))
}

/// Public dashboard routes without authentication.
///
/// # Endpoints
///
/// - `GET /login` - Login page
pub fn public_routes() -> Router<AppState> {
    Router::new().route("/login", get(login_handler))
}
