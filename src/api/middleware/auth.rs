//! Bearer token authentication middleware.

use axum::{
    extract::{FromRequestParts, Request, State},
    middleware::Next,
    response::Response,
};
use axum_auth::AuthBearer;

use crate::{error::AppError, state::AppState};

/// Authenticates requests using Bearer tokens from Authorization header.
///
/// # Header Format
///
/// ```text
/// Authorization: Bearer <token>
/// ```
///
/// # Authentication Flow
///
/// 1. Extract token from `Authorization` header
/// 2. Validate token hash against database
/// 3. Check if token is revoked
/// 4. Update `last_used_at` timestamp
/// 5. Continue to next middleware/handler
///
/// # Errors
///
/// Returns `401 Unauthorized` if:
/// - Authorization header is missing
/// - Token format is invalid
/// - Token is not found or revoked
///
/// Adds `WWW-Authenticate: Bearer` header to 401 responses per RFC 6750.
///
/// # Example
///
/// ```rust,ignore
/// use axum::{Router, routing::get, middleware};
/// use crate::api::middleware::auth;
///
/// let protected = Router::new()
///     .route("/api/stats", get(stats_handler))
///     .layer(middleware::from_fn_with_state(state.clone(), auth::layer));
/// ```
pub async fn layer(
    State(st): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let (mut parts, body) = req.into_parts();

    let AuthBearer(token) = AuthBearer::from_request_parts(&mut parts, &())
        .await
        .map_err(|_| {
            AppError::unauthorized(
                "Unauthorized",
                serde_json::json!({"reason": "Authorization header is missing or invalid"}),
            )
        })?;

    let req = Request::from_parts(parts, body);

    st.auth_service.authenticate(&token).await?;

    Ok(next.run(req).await)
}
