//! Link statistics page handler.

use askama::Template;
use askama_web::WebTemplate;
use axum::{extract::Path, response::IntoResponse};

/// Template for the link statistics page.
///
/// Renders `templates/stats.html` with:
/// - Click chart (time series)
/// - Click details table
/// - Referrer and user agent breakdown
#[derive(Template, WebTemplate)]
#[template(path = "stats.html")]
pub struct StatsTemplate {
    pub code: String,
}

/// Renders the statistics page for a specific link.
///
/// # Endpoint
///
/// `GET /stats/{code}`
///
/// # Template
///
/// Uses `templates/stats.html` for server-side rendering.
/// The template fetches detailed statistics via JavaScript from `/api/stats/{code}`.
pub async fn stats_handler(Path(code): Path<String>) -> impl IntoResponse {
    StatsTemplate { code }
}
