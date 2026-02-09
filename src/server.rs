//! HTTP server initialization and runtime setup.
//!
//! Handles database connections, cache setup, worker spawning, and Axum server lifecycle.

use crate::config::Config;
use crate::domain::click_worker::run_click_worker;
use crate::infrastructure::cache::{CacheService, NullCache, RedisCache};
use crate::infrastructure::persistence::{PgDomainRepository, PgLinkRepository, PgStatsRepository};
use crate::routes::app_router;
use crate::state::AppState;

use anyhow::Result;
use axum::ServiceExt;
use axum::extract::Request;
use sqlx::PgPool;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Runs the HTTP server with the given configuration.
///
/// Initializes:
/// - PostgreSQL connection pool
/// - Apply migrations
/// - Redis cache (or NullCache fallback)
/// - Background click worker
/// - Axum HTTP server
///
/// # Errors
///
/// Returns an error if:
/// - Database connection fails
/// - Server bind fails
/// - Server runtime error occurs
pub async fn run(config: Config) -> Result<()> {
    let pool = PgPool::connect(&config.database_url).await?;
    tracing::info!("Connected to database");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to migrate");

    let cache: Arc<dyn CacheService> = if let Some(redis_url) = &config.redis_url {
        match RedisCache::connect(redis_url).await {
            Ok(redis) => {
                tracing::info!("Cache enabled (Redis)");
                Arc::new(redis)
            }
            Err(e) => {
                tracing::warn!("Failed to connect to Redis: {}. Using NullCache.", e);
                Arc::new(NullCache::new())
            }
        }
    } else {
        tracing::info!("Cache disabled (NullCache)");
        Arc::new(NullCache::new())
    };

    let (click_tx, click_rx) = mpsc::channel(config.click_queue_capacity);

    let pool_arc = Arc::new(pool.clone());
    let stats_repository = Arc::new(PgStatsRepository::new(pool_arc.clone()));
    let domain_repository = Arc::new(PgDomainRepository::new(pool_arc.clone()));
    let link_repository = Arc::new(PgLinkRepository::new(pool_arc.clone()));
    tokio::spawn(run_click_worker(
        click_rx,
        stats_repository,
        domain_repository,
        link_repository,
    ));
    tracing::info!("Click worker started");

    let state = AppState::new(Arc::new(pool), click_tx, cache);

    let app = app_router(state);

    let addr: SocketAddr = config.listen_addr.parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("Listening on http://{addr}");

    axum::serve(
        listener,
        ServiceExt::<Request>::into_make_service_with_connect_info::<SocketAddr>(app),
    )
    .await?;

    Ok(())
}
