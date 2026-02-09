//! Repository trait for click statistics and analytics.

use crate::domain::entities::{Click, NewClick};
use crate::error::AppError;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

/// Aggregated statistics for a single link.
///
/// Combines link metadata with total click count.
#[derive(Debug, Clone)]
pub struct LinkStats {
    #[allow(dead_code)]
    pub link_id: i64,
    pub code: String,
    pub domain: Option<String>,
    pub long_url: String,
    pub total: i64,
    pub created_at: DateTime<Utc>,
}

/// Detailed statistics with individual click records.
///
/// Includes full link information, total count, and paginated click events.
#[derive(Debug, Clone)]
pub struct DetailedStats {
    pub link: crate::domain::entities::Link,
    pub total: i64,
    pub items: Vec<Click>,
}

/// Filter criteria for statistics queries.
///
/// Supports date range filtering, pagination, and domain scoping.
#[derive(Debug, Clone)]
pub struct StatsFilter {
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
    pub offset: i64,
    pub limit: i64,
    pub domain_id: Option<i64>,
}

impl StatsFilter {
    /// Creates a new filter with pagination parameters.
    pub fn new(offset: i64, limit: i64) -> Self {
        Self {
            from_date: None,
            to_date: None,
            offset,
            limit,
            domain_id: None,
        }
    }

    /// Adds domain filtering to the query.
    pub fn with_domain(mut self, domain_id: Option<i64>) -> Self {
        self.domain_id = domain_id;
        self
    }

    /// Adds date range filtering to the query.
    pub fn with_date_range(
        mut self,
        from_date: Option<DateTime<Utc>>,
        to_date: Option<DateTime<Utc>>,
    ) -> Self {
        self.from_date = from_date;
        self.to_date = to_date;
        self
    }
}

/// Repository interface for click tracking and statistics.
///
/// Handles both recording click events and querying aggregated statistics
/// with flexible filtering options.
///
/// # Implementations
///
/// - [`crate::infrastructure::persistence::PgStatsRepository`] - PostgreSQL implementation
/// - Test mocks available with `cfg(test)`
///
/// # Examples
///
/// See integration tests: `tests/repository_stats.rs`
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait StatsRepository: Send + Sync {
    /// Records a new click event.
    ///
    /// Stores metadata like IP address, user agent, and referrer for analytics.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Validation`] if the referenced link does not exist.
    /// Returns [`AppError::Internal`] on database errors.
    async fn record_click(&self, new_click: NewClick) -> Result<Click, AppError>;

    /// Retrieves detailed statistics for a specific short code.
    ///
    /// Includes individual click records with pagination and optional filtering.
    ///
    /// # Returns
    ///
    /// - `Ok(Some(DetailedStats))` if the link exists
    /// - `Ok(None)` if the link is not found
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Internal`] on database errors.
    async fn get_stats_by_code(
        &self,
        code: &str,
        filter: StatsFilter,
    ) -> Result<Option<DetailedStats>, AppError>;

    /// Retrieves aggregated statistics for all links.
    ///
    /// Returns a paginated list with total click counts per link.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Internal`] on database errors.
    async fn get_all_stats(&self, filter: StatsFilter) -> Result<Vec<LinkStats>, AppError>;

    /// Counts the total number of links in the system.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Internal`] on database errors.
    async fn count_all_links(&self) -> Result<i64, AppError>;

    /// Counts clicks for a specific link within an optional date range.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Internal`] on database errors.
    async fn count_clicks_by_link_id(
        &self,
        link_id: i64,
        from_date: Option<DateTime<Utc>>,
        to_date: Option<DateTime<Utc>>,
    ) -> Result<i64, AppError>;
}
