//! Cookie-based authentication middleware for web dashboard.

use axum::{
    extract::{Request, State},
    http::header::COOKIE,
    middleware::Next,
    response::{Redirect, Response},
};

use crate::state::AppState;

/// Authenticates dashboard requests using cookie-based tokens.
///
/// # Cookie Format
///
/// ```text
/// Cookie: auth_token=<token>
/// ```
///
/// # Authentication Flow
///
/// 1. Extract `auth_token` cookie from request
/// 2. Validate token via [`crate::application::services::auth_service::AuthService`]
/// 3. On success, continue to handler
/// 4. On failure or missing token, redirect to `/dashboard/login`
///
/// # Differences from API Auth
///
/// Unlike the API auth middleware which returns `401 Unauthorized`,
/// this middleware redirects to the login page for a better user experience
/// in a browser context.
///
/// # Cookie Parsing
///
/// Handles multiple cookies in the `Cookie` header by:
/// - Splitting on semicolons
/// - Extracting `auth_token` key-value pair
/// - Ignoring other cookies
///
/// # Example
///
/// ```rust,ignore
/// use axum::{Router, routing::get, middleware};
/// use crate::web::middleware::web_auth;
///
/// let protected = Router::new()
///     .route("/dashboard", get(dashboard_handler))
///     .layer(middleware::from_fn_with_state(state.clone(), web_auth::layer));
/// ```
///
/// # Errors
///
/// Returns `Redirect` to `/dashboard/login` if:
/// - `auth_token` cookie is missing
/// - Token format is invalid
/// - Token validation fails (invalid/revoked)
pub async fn layer(
    State(st): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, Redirect> {
    let token = req
        .headers()
        .get(COOKIE)
        .and_then(|cookie_header| cookie_header.to_str().ok())
        .and_then(|cookie_str| {
            cookie_str.split(';').find_map(|cookie| {
                let mut parts = cookie.trim().splitn(2, '=');
                match (parts.next(), parts.next()) {
                    (Some("auth_token"), Some(value)) => Some(value.to_string()),
                    _ => None,
                }
            })
        });

    match token {
        Some(token) => match st.auth_service.authenticate(&token).await {
            Ok(_) => Ok(next.run(req).await),
            Err(_) => Err(Redirect::to("/dashboard/login")),
        },
        None => Err(Redirect::to("/dashboard/login")),
    }
}
