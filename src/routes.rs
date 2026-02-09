//! Top-level router configuration combining API and web routes.

use crate::api;
use crate::api::handlers::redirect_handler;
use crate::api::middleware::{auth, rate_limit, tracing};
use crate::state::AppState;
use crate::web;
use crate::web::middleware::web_auth;
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Router, middleware};
use tower::Layer;
use tower_http::normalize_path::{NormalizePath, NormalizePathLayer};
use tower_http::services::ServeDir;

/// Constructs the application router with all routes and middleware.
///
/// # Route Structure
///
/// - `/{code}` - Redirect endpoint (public)
/// - `/health` - Health status
/// - `/api/*` - REST API (protected routes require Bearer token)
/// - `/dashboard/*` - Web UI (protected routes require session)
/// - `/static/*` - Static assets
///
/// # Middleware Layers
///
/// - Tracing - Request/response logging
/// - Rate limiting - Different limits for public/protected routes
/// - Authentication - Token (API) or session (web) validation
/// - Path normalization - Trailing slash handling
pub fn app_router(state: AppState) -> NormalizePath<Router> {
    let api_protected = api::routes::protected_routes()
        .route_layer(middleware::from_fn_with_state(state.clone(), auth::layer))
        .layer(rate_limit::secure_layer());

    let api_public = api::routes::public_routes().layer(rate_limit::layer());

    let api_router = Router::new().merge(api_protected).merge(api_public);

    let web_protected = web::routes::protected_routes()
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            web_auth::layer,
        ))
        .layer(rate_limit::secure_layer());

    let web_public = web::routes::public_routes().layer(rate_limit::layer());

    let web_router = Router::new().merge(web_protected).merge(web_public);

    let router = Router::new()
        .route("/{code}", get(redirect_handler))
        .route("/health", get(|| async { StatusCode::OK }))
        .nest("/api", api_router)
        .nest("/dashboard", web_router)
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state)
        .layer(tracing::layer());

    NormalizePathLayer::trim_trailing_slash().layer(router)
}
