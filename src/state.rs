//! Application state shared across HTTP handlers.
//!
//! Contains service instances, database pool, cache, and channels for
//! asynchronous click processing. Cloned for each request via Axum's
//! state extraction.

use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::application::services::{AuthService, DomainService, LinkService, StatsService};
use crate::domain::click_event::ClickEvent;
use crate::infrastructure::cache::CacheService;
use crate::infrastructure::persistence::{
    PgDomainRepository, PgLinkRepository, PgStatsRepository, PgTokenRepository,
};

/// Shared application state injected into HTTP handlers.
///
/// Contains all services and infrastructure components needed to process requests.
/// Cheap to clone due to `Arc` wrapping.
#[derive(Clone)]
pub struct AppState {
    pub link_service: Arc<LinkService<PgLinkRepository, PgDomainRepository>>,
    pub stats_service: Arc<StatsService<PgStatsRepository>>,
    pub auth_service: Arc<AuthService<PgTokenRepository>>,
    pub domain_service: Arc<DomainService<PgDomainRepository>>,

    pub cache: Arc<dyn CacheService>,

    pub click_sender: mpsc::Sender<ClickEvent>,
}

impl AppState {
    /// Creates a new application state with initialized services.
    ///
    /// # Arguments
    ///
    /// - `pool` - Database connection pool
    /// - `click_sender` - Channel sender for async click processing
    /// - `cache` - Cache implementation (Redis or NullCache)
    pub fn new(
        pool: Arc<PgPool>,
        click_sender: mpsc::Sender<ClickEvent>,
        cache: Arc<dyn CacheService>,
    ) -> Self {
        let link_repo = Arc::new(PgLinkRepository::new(pool.clone()));
        let stats_repo = Arc::new(PgStatsRepository::new(pool.clone()));
        let token_repo = Arc::new(PgTokenRepository::new(pool.clone()));
        let domain_repo = Arc::new(PgDomainRepository::new(pool.clone()));

        let link_service = Arc::new(LinkService::new(link_repo, domain_repo.clone()));
        let stats_service = Arc::new(StatsService::new(stats_repo));
        let auth_service = Arc::new(AuthService::new(token_repo));
        let domain_service = Arc::new(DomainService::new(domain_repo));

        Self {
            link_service,
            stats_service,
            auth_service,
            domain_service,
            cache,
            click_sender,
        }
    }
}
