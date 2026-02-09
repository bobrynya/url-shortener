//! Login page handler.

use askama::Template;
use askama_web::WebTemplate;
use axum::response::IntoResponse;

/// Template for the login page.
///
/// Renders `templates/login.html` with:
/// - Token input form
/// - Authentication instructions
#[derive(Template, WebTemplate)]
#[template(path = "login.html")]
struct LoginTemplate {}

/// Renders the login page.
///
/// # Endpoint
///
/// `GET /login`
///
/// # Authentication
///
/// Users enter their API token which is stored in a cookie or session
/// for subsequent requests to protected dashboard routes.
///
/// # Template
///
/// Uses `templates/login.html` for server-side rendering.
pub async fn login_handler() -> impl IntoResponse {
    LoginTemplate {}
}
