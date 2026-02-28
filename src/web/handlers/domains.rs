//! Domain management page handler.

use askama::Template;
use askama_web::WebTemplate;
use axum::response::IntoResponse;

/// Template for the domain management page.
///
/// Renders `templates/domains.html` with a full CRUD interface for domains.
/// Data is fetched client-side via Alpine.js from `/api/domains`.
#[derive(Template, WebTemplate)]
#[template(path = "domains.html")]
pub struct DomainsTemplate {}

/// Renders the domain management page.
///
/// # Endpoint
///
/// `GET /domains`
pub async fn domains_handler() -> impl IntoResponse {
    DomainsTemplate {}
}
