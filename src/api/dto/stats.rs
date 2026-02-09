//! DTOs for detailed link statistics.

use chrono::{DateTime, Utc};
use serde::Serialize;

use super::clicks::ClickInfo;
use super::stats_list::PaginationMeta;

/// Detailed statistics for a specific short link.
///
/// Includes link metadata, total click count, and paginated click records.
#[derive(Debug, Serialize)]
pub struct StatsResponse {
    pub pagination: PaginationMeta,
    pub code: String,
    pub domain: Option<String>,
    pub long_url: String,
    pub created_at: DateTime<Utc>,
    pub total: i64,
    pub items: Vec<ClickInfo>,
}
