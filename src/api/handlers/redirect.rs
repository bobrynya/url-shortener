//! Handler for short URL redirect.

use axum::{
    extract::{Path, State},
    http::{HeaderMap, header},
    response::{IntoResponse, Redirect},
};
use std::net::SocketAddr;
use tracing::{debug, error};

use crate::domain::click_event::ClickEvent;
use crate::error::AppError;
use crate::state::AppState;
use crate::utils::extract_domain::extract_domain_from_headers;

/// Redirects a short code to its original URL.
///
/// # Endpoint
///
/// `GET /{code}`
///
/// # Request Flow
///
/// 1. Extract domain from Host header
/// 2. Check cache for URL (cache key: `domain:code`)
/// 3. On cache miss, query database
/// 4. Asynchronously update cache
/// 5. Send click event to background worker
/// 6. Return 307 Temporary Redirect
///
/// # Cache Strategy
///
/// - **Cache hit**: Immediate redirect
/// - **Cache miss**: Query DB, spawn async cache write
/// - **Cache error**: Log and fall back to DB
///
/// # Click Tracking
///
/// Click events are sent to a bounded channel for async processing.
/// If the queue is full, the click is dropped (fire-and-forget).
///
/// # Errors
///
/// Returns 404 Not Found if the short code doesn't exist.
/// Returns 400 Bad Request if the Host header is missing or invalid.
pub async fn redirect_handler(
    Path(code): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<SocketAddr>,
) -> Result<impl IntoResponse, AppError> {
    let domain = extract_domain_from_headers(&headers)?;

    let cache_key = format!("{}:{}", domain, code);

    let long_url = match state.cache.get_url(&cache_key).await {
        Ok(Some(cached_url)) => {
            debug!("Cache HIT for {}", cache_key);
            cached_url
        }
        Ok(None) => {
            debug!("Cache MISS for {}", cache_key);

            let domain_entity = state.domain_service.get_domain(&domain).await?;

            let link = state
                .link_service
                .get_link_by_code(&code, domain_entity.id)
                .await?;

            // Asynchronously update cache (fire-and-forget)
            let cache_clone = state.cache.clone();
            let cache_key_clone = cache_key.clone();
            let url_clone = link.long_url.clone();
            tokio::spawn(async move {
                if let Err(e) = cache_clone
                    .set_url(&cache_key_clone, &url_clone, None)
                    .await
                {
                    error!("Failed to cache URL: {}", e);
                }
            });

            link.long_url
        }
        Err(e) => {
            error!("Cache error: {}", e);

            // Fall back to database on cache error
            let domain_entity = state.domain_service.get_domain(&domain).await?;
            let link = state
                .link_service
                .get_link_by_code(&code, domain_entity.id)
                .await?;

            link.long_url
        }
    };

    // Send click event for async processing
    let click_event = ClickEvent::new(
        domain,
        code,
        Some(addr.ip().to_string()),
        headers
            .get(header::USER_AGENT)
            .and_then(|v| v.to_str().ok()),
        headers.get(header::REFERER).and_then(|v| v.to_str().ok()),
    );

    let _ = state.click_sender.try_send(click_event);

    Ok(Redirect::temporary(&long_url))
}
