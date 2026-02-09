//! Dashboard home page handler.

use askama::Template;
use askama_web::WebTemplate;
use axum::response::IntoResponse;

/// Template for the dashboard home page.
///
/// Renders `templates/dashboard.html` with an overview of:
/// - Total links created
/// - Recent clicks
/// - Quick actions
#[derive(Template, WebTemplate)]
#[template(path = "dashboard.html")]
pub struct DashboardTemplate {}

/// Renders the dashboard home page.
///
/// # Endpoint
///
/// `GET /`
///
/// # Template
///
/// Uses `templates/dashboard.html` for server-side rendering.
pub async fn dashboard_handler() -> impl IntoResponse {
    DashboardTemplate {}
}
