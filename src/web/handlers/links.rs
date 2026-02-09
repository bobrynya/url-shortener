//! Link management page handler.

use askama::Template;
use askama_web::WebTemplate;
use axum::response::IntoResponse;

/// Template for the links management page.
///
/// Renders `templates/links.html` with:
/// - Link creation form
/// - Paginated link list
/// - Bulk actions
#[derive(Template, WebTemplate)]
#[template(path = "links.html")]
pub struct LinksTemplate {}

/// Renders the link management page.
///
/// # Endpoint
///
/// `GET /links`
///
/// # Template
///
/// Uses `templates/links.html` for server-side rendering.
/// The template fetches data via JavaScript from `/api/stats`.
pub async fn links_handler() -> impl IntoResponse {
    LinksTemplate {}
}
