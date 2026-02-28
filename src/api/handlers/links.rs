//! Handlers for link management endpoints (create, update, delete).

use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::json;
use validator::Validate;

use crate::api::dto::shorten::{
    BatchSummary, ShortenRequest, ShortenResponse, ShortenResultItem, UrlItem,
};
use crate::api::dto::update_link::UpdateLinkRequest;
use crate::domain::entities::LinkPatch;
use crate::error::AppError;
use crate::state::AppState;
use crate::utils::extract_domain::extract_domain_from_headers;

/// JSON representation of a link returned after update.
#[derive(Debug, Serialize)]
pub struct LinkResponse {
    pub code: String,
    pub long_url: String,
    pub short_url: String,
    pub permanent: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Creates shortened URLs for one or more long URLs.
///
/// # Endpoint
///
/// `POST /api/shorten`
///
/// # Batch Processing
///
/// Processes URLs independently. If one fails, others continue processing.
/// Each result includes either success data or error information.
///
/// # Request Body
///
/// ```json
/// {
///   "urls": [
///     {
///       "url": "https://example.com",
///       "domain": "s.example.com",  // optional
///       "custom_code": "my-link"     // optional
///     }
///   ]
/// }
/// ```
///
/// # Errors
///
/// Returns 400 Bad Request if validation fails.
/// Individual URL errors are returned in the response items array.
pub async fn shorten_handler(
    State(state): State<AppState>,
    Json(payload): Json<ShortenRequest>,
) -> Result<Json<ShortenResponse>, AppError> {
    payload.validate()?;

    let total = payload.urls.len();
    let mut results = Vec::with_capacity(total);
    let mut successful = 0;
    let mut failed = 0;

    for item in payload.urls {
        let long_url = item.url.clone();

        match process_single_url(&state, item).await {
            Ok((code, short_url)) => {
                successful += 1;
                results.push(ShortenResultItem::Success {
                    long_url,
                    code,
                    short_url,
                });
            }
            Err(err) => {
                failed += 1;
                results.push(ShortenResultItem::Error {
                    long_url,
                    error: err.to_error_info(),
                });
            }
        }
    }

    Ok(Json(ShortenResponse {
        summary: BatchSummary {
            total,
            successful,
            failed,
        },
        items: results,
    }))
}

/// Resolves the target domain, creates the short link, and generates the full URL.
async fn process_single_url(state: &AppState, item: UrlItem) -> Result<(String, String), AppError> {
    let domain = if let Some(domain_name) = item.domain {
        state.domain_service.get_domain(&domain_name).await?
    } else {
        state.domain_service.get_default_domain().await?
    };

    let link = state
        .link_service
        .create_short_link_for_domain(
            item.url,
            domain.id,
            item.custom_code,
            item.expires_at,
            item.permanent.unwrap_or(false),
        )
        .await?;

    let short_url = state.link_service.get_short_url(&domain.domain, &link.code);

    Ok((link.code, short_url))
}

/// Partially updates a short link.
///
/// # Endpoint
///
/// `PATCH /api/links/{code}`
///
/// # Request Body
///
/// All fields are optional. Only provided fields are changed.
///
/// ```json
/// {
///   "url": "https://new-destination.com",
///   "expires_at": "2026-12-31T23:59:59Z",  // null to clear
///   "permanent": true,
///   "restore": true   // clears deleted_at to un-delete the link
/// }
/// ```
///
/// # Cache
///
/// The cache entry for this link is invalidated so the next redirect uses the
/// updated destination and redirect type.
///
/// # Errors
///
/// Returns 404 Not Found if the link doesn't exist for this domain.
/// Returns 400 Bad Request if validation fails.
pub async fn update_link_handler(
    Path(code): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<UpdateLinkRequest>,
) -> Result<Json<LinkResponse>, AppError> {
    payload.validate()?;

    let domain = extract_domain_from_headers(&headers)?;
    let domain_entity = state.domain_service.get_domain(&domain).await?;

    let patch = LinkPatch {
        url: payload.url,
        expires_at: payload.expires_at,
        permanent: payload.permanent,
        restore: payload.restore,
    };

    let link = state
        .link_service
        .update_link(&code, domain_entity.id, patch)
        .await?;

    let cache_key = format!("{}:{}", domain, code);
    if let Err(e) = state.cache.invalidate(&cache_key).await {
        tracing::warn!(error = ?e, cache_key, "Failed to invalidate cache after update");
    }

    let short_url = state.link_service.get_short_url(&domain, &link.code);

    Ok(Json(LinkResponse {
        code: link.code,
        long_url: link.long_url,
        short_url,
        permanent: link.permanent,
        expires_at: link.expires_at,
        deleted_at: link.deleted_at,
        created_at: link.created_at,
    }))
}

/// Soft-deletes a short link by setting its `deleted_at` timestamp.
///
/// # Endpoint
///
/// `DELETE /api/links/{code}`
///
/// # Behavior
///
/// - The link record is **not** removed from the database. `deleted_at` is set to now.
/// - Subsequent redirect requests for this code will return **410 Gone**.
/// - A deleted link can be restored via `PATCH /api/links/{code}` with `{"restore": true}`.
///
/// # Cache
///
/// The cache entry for this link is invalidated immediately so the next redirect
/// reflects the deleted state without waiting for TTL expiry.
///
/// # Errors
///
/// Returns 404 Not Found if the link doesn't exist or is already deleted.
pub async fn delete_link_handler(
    Path(code): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<StatusCode, AppError> {
    let domain = extract_domain_from_headers(&headers)?;
    let domain_entity = state.domain_service.get_domain(&domain).await?;

    let deleted = state
        .link_service
        .soft_delete_link(&code, domain_entity.id)
        .await?;

    if !deleted {
        return Err(AppError::not_found(
            "Link not found or already deleted",
            json!({ "code": code }),
        ));
    }

    let cache_key = format!("{}:{}", domain, code);
    if let Err(e) = state.cache.invalidate(&cache_key).await {
        tracing::warn!(error = ?e, cache_key, "Failed to invalidate cache after delete");
    }

    Ok(StatusCode::NO_CONTENT)
}
