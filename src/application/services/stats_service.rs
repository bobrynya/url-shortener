//! Click statistics and analytics service.

use std::sync::Arc;

use crate::domain::entities::{Click, NewClick};
use crate::domain::repositories::{DetailedStats, LinkStats, StatsFilter, StatsRepository};
use crate::error::AppError;
use serde_json::json;

/// Service for retrieving click statistics and analytics.
///
/// Provides both aggregated statistics (total clicks per link) and detailed
/// click records with filtering by date range and pagination.
pub struct StatsService<R: StatsRepository> {
    repository: Arc<R>,
}

impl<R: StatsRepository> StatsService<R> {
    /// Creates a new statistics service.
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    /// Records a click event for a link.
    ///
    /// # Note
    ///
    /// In production, clicks are typically recorded asynchronously via
    /// the background worker (`click_worker`). This method exists for
    /// testing and direct recording scenarios.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Validation`] if the link does not exist.
    /// Returns [`AppError::Internal`] on database errors.
    #[allow(dead_code)]
    pub async fn record_click(
        &self,
        link_id: i64,
        user_agent: Option<String>,
        referer: Option<String>,
        ip: Option<String>,
    ) -> Result<Click, AppError> {
        let new_click = NewClick {
            link_id,
            user_agent,
            referer,
            ip,
        };

        self.repository.record_click(new_click).await
    }

    /// Retrieves detailed statistics for a specific short code.
    ///
    /// Includes link metadata, total click count, and paginated click records
    /// with optional date filtering.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::NotFound`] if no link matches the code.
    /// Returns [`AppError::Internal`] on database errors.
    pub async fn get_detailed_stats(
        &self,
        code: &str,
        filter: StatsFilter,
    ) -> Result<DetailedStats, AppError> {
        self.repository
            .get_stats_by_code(code, filter)
            .await?
            .ok_or_else(|| AppError::not_found("Statistics not found", json!({ "code": code })))
    }

    /// Retrieves aggregated statistics for all links.
    ///
    /// Returns a paginated list with total click counts per link, optionally
    /// filtered by date range and domain.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Internal`] on database errors.
    pub async fn get_all_stats(&self, filter: StatsFilter) -> Result<Vec<LinkStats>, AppError> {
        self.repository.get_all_stats(filter).await
    }

    /// Counts the total number of links in the system.
    ///
    /// Used for pagination metadata.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Internal`] on database errors.
    pub async fn count_all_links(&self) -> Result<i64, AppError> {
        self.repository.count_all_links().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::Link;
    use crate::domain::repositories::MockStatsRepository;
    use chrono::Utc;

    #[tokio::test]
    async fn test_get_detailed_stats_success() {
        let mut mock_repo = MockStatsRepository::new();

        let link = Link::new(
            1,
            "abc123".to_string(),
            "https://example.com".to_string(),
            Some("s.example.com".to_string()),
            Utc::now(),
        );

        let stats = DetailedStats {
            link: link.clone(),
            total: 5,
            items: vec![],
        };

        mock_repo
            .expect_get_stats_by_code()
            .withf(|code, _| code == "abc123")
            .times(1)
            .returning(move |_, _| Ok(Some(stats.clone())));

        let service = StatsService::new(Arc::new(mock_repo));

        let filter = StatsFilter::new(0, 10);
        let result = service.get_detailed_stats("abc123", filter).await;

        assert!(result.is_ok());
        let stats = result.unwrap();
        assert_eq!(stats.total, 5);
        assert_eq!(stats.link.code, "abc123");
    }

    #[tokio::test]
    async fn test_get_detailed_stats_not_found() {
        let mut mock_repo = MockStatsRepository::new();

        mock_repo
            .expect_get_stats_by_code()
            .times(1)
            .returning(|_, _| Ok(None));

        let service = StatsService::new(Arc::new(mock_repo));

        let filter = StatsFilter::new(0, 10);
        let result = service.get_detailed_stats("notfound", filter).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::NotFound { .. }));
    }

    #[tokio::test]
    async fn test_get_all_stats() {
        let mut mock_repo = MockStatsRepository::new();

        let link_stats = vec![
            LinkStats {
                link_id: 1,
                code: "abc123".to_string(),
                domain: Some("s.example.com".to_string()),
                long_url: "https://example.com".to_string(),
                total: 10,
                created_at: Utc::now(),
            },
            LinkStats {
                link_id: 2,
                code: "xyz789".to_string(),
                domain: Some("s.example.com".to_string()),
                long_url: "https://test.com".to_string(),
                total: 5,
                created_at: Utc::now(),
            },
        ];

        mock_repo
            .expect_get_all_stats()
            .times(1)
            .returning(move |_| Ok(link_stats.clone()));

        let service = StatsService::new(Arc::new(mock_repo));

        let filter = StatsFilter::new(0, 10);
        let result = service.get_all_stats(filter).await;

        assert!(result.is_ok());
        let stats = result.unwrap();
        assert_eq!(stats.len(), 2);
        assert_eq!(stats[0].code, "abc123");
        assert_eq!(stats[1].code, "xyz789");
    }

    #[tokio::test]
    async fn test_count_all_links() {
        let mut mock_repo = MockStatsRepository::new();

        mock_repo
            .expect_count_all_links()
            .times(1)
            .returning(|| Ok(42));

        let service = StatsService::new(Arc::new(mock_repo));

        let result = service.count_all_links().await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_record_click() {
        let mut mock_repo = MockStatsRepository::new();

        let click = crate::domain::entities::Click::new(
            1,
            10,
            Utc::now(),
            Some("Mozilla/5.0".to_string()),
            None,
            Some("192.168.1.1".to_string()),
        );

        mock_repo
            .expect_record_click()
            .times(1)
            .returning(move |_| Ok(click.clone()));

        let service = StatsService::new(Arc::new(mock_repo));

        let result = service
            .record_click(
                10,
                Some("Mozilla/5.0".to_string()),
                None,
                Some("192.168.1.1".to_string()),
            )
            .await;

        assert!(result.is_ok());
        let recorded = result.unwrap();
        assert_eq!(recorded.link_id, 10);
        assert_eq!(recorded.user_agent, Some("Mozilla/5.0".to_string()));
    }
}
