//! Handler for detailed link statistics.

use axum::{
    Json,
    extract::{Path, Query, State},
};

use crate::api::dto::pagination::StatsQueryParams;
use crate::api::dto::stats::StatsResponse;
use crate::api::dto::stats_list::PaginationMeta;
use crate::domain::repositories::StatsFilter;
use crate::error::AppError;
use crate::state::AppState;
use serde_json::json;

/// Retrieves detailed statistics for a specific short link.
///
/// # Endpoint
///
/// `GET /api/stats/{code}`
///
/// # Query Parameters
///
/// - `page` (optional): Page number (default: 1)
/// - `page_size` (optional): Items per page (default: 25, max: 1000)
/// - `from` (optional): Start date (RFC3339 format)
/// - `to` (optional): End date (RFC3339 format)
/// - `domain` (optional): Filter by domain name
///
/// # Response
///
/// Returns link metadata, total click count, and paginated click records.
///
/// # Errors
///
/// Returns 404 Not Found if the short code doesn't exist.
/// Returns 400 Bad Request if pagination parameters are invalid.
pub async fn stats_handler(
    State(state): State<AppState>,
    Path(code): Path<String>,
    Query(params): Query<StatsQueryParams>,
) -> Result<Json<StatsResponse>, AppError> {
    let (offset, limit) = params
        .pagination
        .validate_and_get_offset_limit()
        .map_err(|e| AppError::bad_request(e, json!({})))?;

    let page = params.pagination.page.unwrap_or(1);
    let page_size = params.pagination.page_size.unwrap_or(25);

    let domain_id = if let Some(domain_name) = &params.domain {
        let domain = state.domain_service.get_domain(domain_name).await?;
        Some(domain.id)
    } else {
        None
    };

    let filter = StatsFilter::new(offset, limit)
        .with_domain(domain_id)
        .with_date_range(params.date_filter.from, params.date_filter.to);

    let detailed_stats = state
        .stats_service
        .get_detailed_stats(&code, filter)
        .await?;

    let total_pages = (detailed_stats.total as f64 / page_size as f64).ceil() as u32;

    let response = StatsResponse {
        pagination: PaginationMeta {
            page,
            page_size,
            total_items: detailed_stats.total,
            total_pages,
        },
        code: detailed_stats.link.code,
        domain: detailed_stats.link.domain,
        long_url: detailed_stats.link.long_url,
        created_at: detailed_stats.link.created_at,
        total: detailed_stats.total,
        items: detailed_stats
            .items
            .into_iter()
            .map(|click| crate::api::dto::clicks::ClickInfo {
                clicked_at: click.clicked_at,
                user_agent: click.user_agent,
                referer: click.referer,
                ip: click.ip,
            })
            .collect(),
    };

    Ok(Json(response))
}
