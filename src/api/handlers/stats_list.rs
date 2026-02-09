//! Handler for aggregated link statistics.

use axum::{
    Json,
    extract::{Query, State},
};

use crate::api::dto::pagination::StatsQueryParams;
use crate::api::dto::stats_list::{LinkStatsItem, PaginationMeta, StatsListResponse};
use crate::domain::repositories::StatsFilter;
use crate::error::AppError;
use crate::state::AppState;
use serde_json::json;

/// Retrieves aggregated statistics for all links.
///
/// # Endpoint
///
/// `GET /api/stats`
///
/// # Query Parameters
///
/// - `page` (optional): Page number (default: 1)
/// - `page_size` (optional): Items per page (default: 25, max: 1000)
/// - `from` (optional): Start date for click filtering (RFC3339 format)
/// - `to` (optional): End date for click filtering (RFC3339 format)
/// - `domain` (optional): Filter by domain name
///
/// # Response
///
/// Returns paginated list of links with total click counts.
///
/// # Performance
///
/// Uses `tokio::try_join!` to parallelize stats query and total count query.
///
/// # Errors
///
/// Returns 400 Bad Request if pagination parameters are invalid.
pub async fn stats_list_handler(
    State(state): State<AppState>,
    Query(params): Query<StatsQueryParams>,
) -> Result<Json<StatsListResponse>, AppError> {
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

    let (all_stats, total_items) = tokio::try_join!(
        state.stats_service.get_all_stats(filter),
        state.stats_service.count_all_links()
    )?;

    let items = all_stats
        .into_iter()
        .map(|stat| LinkStatsItem {
            code: stat.code,
            domain: stat.domain,
            long_url: stat.long_url,
            total: stat.total,
            created_at: stat.created_at,
        })
        .collect();

    let total_pages = ((total_items as f64) / (page_size as f64)).ceil() as u32;

    Ok(Json(StatsListResponse {
        pagination: PaginationMeta {
            page,
            page_size,
            total_items,
            total_pages,
        },
        items,
    }))
}
