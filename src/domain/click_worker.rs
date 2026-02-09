//! Background worker for processing click events asynchronously.

use serde_json::json;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_retry::RetryIf;
use tokio_retry::strategy::ExponentialBackoff;

use crate::domain::click_event::ClickEvent;
use crate::domain::entities::NewClick;
use crate::domain::repositories::{DomainRepository, LinkRepository, StatsRepository};
use crate::error::AppError;

/// Determines if an error should trigger a retry.
///
/// Only internal errors (database connection issues, timeouts) are considered transient.
/// Errors like "link not found" are permanent and not retried.
fn is_transient_error(e: &AppError) -> bool {
    matches!(e, AppError::Internal { .. })
}

/// Runs the background click processing worker.
///
/// Consumes click events from a channel and persists them to the database
/// with exponential backoff retry logic for transient errors.
///
/// # Retry Strategy
///
/// - Attempts: 6 (100ms, 200ms, 400ms, 800ms, 1.6s, 3.2s)
/// - Only retries transient errors (database connection issues)
/// - Permanent errors (not found) are logged and dropped
///
/// # Metrics
///
/// Emits counters for observability:
/// - `click_worker_received_total` - Events received from channel
/// - `click_worker_processed_total` - Successfully persisted events
/// - `click_worker_retried_total` - Retry attempts
/// - `click_worker_failed_total` - Failed after all retries
/// - `click_worker_dropped_total` - Events dropped due to errors
///
/// # Graceful Shutdown
///
/// The worker stops when the sending side of the channel is dropped.
///
/// # Examples
///
/// See integration tests and `src/server.rs` for worker initialization.
pub async fn run_click_worker<S, D, L>(
    mut rx: mpsc::Receiver<ClickEvent>,
    stats_repository: Arc<S>,
    domain_repository: Arc<D>,
    link_repository: Arc<L>,
) where
    S: StatsRepository,
    D: DomainRepository,
    L: LinkRepository,
{
    tracing::info!("Click worker started");

    while let Some(ev) = rx.recv().await {
        metrics::counter!("click_worker_received_total").increment(1);

        let strategy = ExponentialBackoff::from_millis(100).take(6);

        let stats_repo = stats_repository.clone();
        let domain_repo = domain_repository.clone();
        let link_repo = link_repository.clone();
        let event = ev.clone();

        let op = || {
            let stats_repo = stats_repo.clone();
            let domain_repo = domain_repo.clone();
            let link_repo = link_repo.clone();
            let event = event.clone();

            async move {
                let domain_entity =
                    domain_repo
                        .find_by_name(&event.domain)
                        .await?
                        .ok_or_else(|| {
                            AppError::not_found(
                                format!("Domain not found: {}", event.domain),
                                json!({ "domain": event.domain.clone() }),
                            )
                        })?;

                let link = link_repo
                    .find_by_code(&event.code, domain_entity.id)
                    .await?
                    .ok_or_else(|| {
                        AppError::not_found(
                            format!("Link not found: {}", event.code),
                            json!({ "code": event.code.clone(), "domain_id": domain_entity.id }),
                        )
                    })?;

                let new_click = NewClick {
                    link_id: link.id,
                    user_agent: event.user_agent,
                    referer: event.referer,
                    ip: event.ip,
                };

                stats_repo.record_click(new_click).await.map(|_| ())
            }
        };

        let on_error = |e: &AppError| {
            let transient = is_transient_error(e);
            if transient {
                metrics::counter!("click_worker_retried_total").increment(1);
                tracing::warn!(
                    domain = &event.domain,
                    code = &event.code,
                    error = ?e,
                    "Click worker: transient error, retrying"
                );
            }
            transient
        };

        let res = RetryIf::spawn(strategy, op, on_error).await;

        match res {
            Ok(()) => {
                metrics::counter!("click_worker_processed_total").increment(1);
                tracing::debug!(
                    domain = &event.domain,
                    code = &event.code,
                    "Click successfully recorded"
                );
            }
            Err(e) => {
                metrics::counter!("click_worker_failed_total").increment(1);
                metrics::counter!("click_worker_dropped_total").increment(1);

                tracing::error!(
                    error = ?e,
                    domain = &event.domain,
                    code = &event.code,
                    "Click worker: failed to persist click event after retries"
                );
            }
        }
    }

    tracing::info!("Click worker stopped");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::{Click, Domain, Link};
    use crate::domain::repositories::{
        MockDomainRepository, MockLinkRepository, MockStatsRepository,
    };
    use chrono::Utc;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_click_worker_successful_processing() {
        let mut mock_domain_repo = MockDomainRepository::new();
        let mut mock_link_repo = MockLinkRepository::new();
        let mut mock_stats_repo = MockStatsRepository::new();

        let domain = Domain::new(
            1,
            "s.example.com".to_string(),
            true,
            true,
            None,
            Utc::now(),
            Utc::now(),
        );
        mock_domain_repo
            .expect_find_by_name()
            .withf(|name| name == "s.example.com")
            .times(1)
            .returning(move |_| Ok(Some(domain.clone())));

        let link = Link::new(
            10,
            "abc123".to_string(),
            "https://example.com".to_string(),
            Some("s.example.com".to_string()),
            Utc::now(),
        );
        mock_link_repo
            .expect_find_by_code()
            .withf(|code, domain_id| code == "abc123" && *domain_id == 1)
            .times(1)
            .returning(move |_, _| Ok(Some(link.clone())));

        let click = Click::new(1, 10, Utc::now(), None, None, None);
        mock_stats_repo
            .expect_record_click()
            .times(1)
            .returning(move |_| Ok(click.clone()));

        let (tx, rx) = mpsc::channel(10);

        let stats_repo = Arc::new(mock_stats_repo);
        let domain_repo = Arc::new(mock_domain_repo);
        let link_repo = Arc::new(mock_link_repo);

        let worker_handle = tokio::spawn(run_click_worker(rx, stats_repo, domain_repo, link_repo));

        let event = ClickEvent::new(
            "s.example.com".to_string(),
            "abc123".to_string(),
            Some("192.168.1.1".to_string()),
            Some("Mozilla/5.0"),
            None,
        );
        tx.send(event).await.unwrap();

        drop(tx);
        worker_handle.await.unwrap();
    }

    #[tokio::test]
    async fn test_click_worker_domain_not_found() {
        let mut mock_domain_repo = MockDomainRepository::new();
        let mock_link_repo = MockLinkRepository::new();
        let mock_stats_repo = MockStatsRepository::new();

        mock_domain_repo
            .expect_find_by_name()
            .times(1)
            .returning(|_| Ok(None));

        let (tx, rx) = mpsc::channel(10);

        let stats_repo = Arc::new(mock_stats_repo);
        let domain_repo = Arc::new(mock_domain_repo);
        let link_repo = Arc::new(mock_link_repo);

        let worker_handle = tokio::spawn(run_click_worker(rx, stats_repo, domain_repo, link_repo));

        let event = ClickEvent::new(
            "nonexistent.com".to_string(),
            "abc123".to_string(),
            None,
            None,
            None,
        );
        tx.send(event).await.unwrap();

        drop(tx);
        worker_handle.await.unwrap();
    }

    #[tokio::test]
    async fn test_click_worker_link_not_found() {
        let mut mock_domain_repo = MockDomainRepository::new();
        let mut mock_link_repo = MockLinkRepository::new();
        let mock_stats_repo = MockStatsRepository::new();

        let domain = Domain::new(
            1,
            "s.example.com".to_string(),
            true,
            true,
            None,
            Utc::now(),
            Utc::now(),
        );
        mock_domain_repo
            .expect_find_by_name()
            .times(1)
            .returning(move |_| Ok(Some(domain.clone())));

        mock_link_repo
            .expect_find_by_code()
            .times(1)
            .returning(|_, _| Ok(None));

        let (tx, rx) = mpsc::channel(10);

        let stats_repo = Arc::new(mock_stats_repo);
        let domain_repo = Arc::new(mock_domain_repo);
        let link_repo = Arc::new(mock_link_repo);

        let worker_handle = tokio::spawn(run_click_worker(rx, stats_repo, domain_repo, link_repo));

        let event = ClickEvent::new(
            "s.example.com".to_string(),
            "nonexistent".to_string(),
            None,
            None,
            None,
        );
        tx.send(event).await.unwrap();

        drop(tx);
        worker_handle.await.unwrap();
    }

    #[tokio::test]
    async fn test_click_worker_processes_multiple_events() {
        let mut mock_domain_repo = MockDomainRepository::new();
        let mut mock_link_repo = MockLinkRepository::new();
        let mut mock_stats_repo = MockStatsRepository::new();

        let domain = Domain::new(
            1,
            "s.example.com".to_string(),
            true,
            true,
            None,
            Utc::now(),
            Utc::now(),
        );
        mock_domain_repo
            .expect_find_by_name()
            .times(3)
            .returning(move |_| Ok(Some(domain.clone())));

        let link = Link::new(
            10,
            "abc123".to_string(),
            "https://example.com".to_string(),
            Some("s.example.com".to_string()),
            Utc::now(),
        );
        mock_link_repo
            .expect_find_by_code()
            .times(3)
            .returning(move |_, _| Ok(Some(link.clone())));

        let click = Click::new(1, 10, Utc::now(), None, None, None);
        mock_stats_repo
            .expect_record_click()
            .times(3)
            .returning(move |_| Ok(click.clone()));

        let (tx, rx) = mpsc::channel(10);

        let stats_repo = Arc::new(mock_stats_repo);
        let domain_repo = Arc::new(mock_domain_repo);
        let link_repo = Arc::new(mock_link_repo);

        let worker_handle = tokio::spawn(run_click_worker(rx, stats_repo, domain_repo, link_repo));

        for _ in 0..3 {
            let event = ClickEvent::new(
                "s.example.com".to_string(),
                "abc123".to_string(),
                Some("192.168.1.1".to_string()),
                None,
                None,
            );
            tx.send(event).await.unwrap();
        }

        drop(tx);
        worker_handle.await.unwrap();
    }
}
