//! Redis-backed cache implementation.

use super::service::{CacheError, CacheResult, CacheService};
use async_trait::async_trait;
use redis::{AsyncCommands, Client, aio::ConnectionManager};
use tracing::{debug, error, info, warn};

/// Redis cache implementation for fast URL lookups.
///
/// Uses connection pooling via `ConnectionManager` for efficient connection reuse.
/// All operations are fail-open: errors are logged but don't propagate to callers.
pub struct RedisCache {
    client: ConnectionManager,
    default_ttl: usize,
    key_prefix: String,
}

impl RedisCache {
    /// Connects to Redis, validates the connection with a PING, and configures the default TTL.
    ///
    /// # Arguments
    ///
    /// - `redis_url` - Redis connection string (e.g., `"redis://localhost:6379"`)
    /// - `default_ttl_seconds` - TTL applied to cached entries when [`CacheService::set_url`]
    ///   is called with `ttl_seconds = None`; controlled via `CACHE_TTL_SECONDS` env var
    ///
    /// # Errors
    ///
    /// Returns [`CacheError::ConnectionError`] if the URL is invalid, the connection cannot
    /// be established, or the PING health check fails.
    pub async fn connect(redis_url: &str, default_ttl_seconds: u64) -> CacheResult<Self> {
        info!("Connecting to Redis at {}", redis_url);

        let client = Client::open(redis_url).map_err(|e| {
            CacheError::ConnectionError(format!("Failed to create Redis client: {}", e))
        })?;

        let manager = ConnectionManager::new(client).await.map_err(|e| {
            CacheError::ConnectionError(format!("Failed to connect to Redis: {}", e))
        })?;

        let mut test_conn = manager.clone();
        test_conn
            .ping::<()>()
            .await
            .map_err(|e| CacheError::ConnectionError(format!("Redis PING failed: {}", e)))?;

        info!("✓ Connected to Redis");

        Ok(Self {
            client: manager,
            default_ttl: default_ttl_seconds as usize,
            key_prefix: "url:".to_string(),
        })
    }

    /// Constructs the full Redis key with namespace prefix.
    fn build_key(&self, short_code: &str) -> String {
        format!("{}{}", self.key_prefix, short_code)
    }
}

#[async_trait]
impl CacheService for RedisCache {
    async fn get_url(&self, short_code: &str) -> CacheResult<Option<String>> {
        let key = self.build_key(short_code);
        let mut conn = self.client.clone();

        match conn.get::<_, Option<String>>(&key).await {
            Ok(Some(url)) => {
                debug!("Cache HIT: {} -> {}", short_code, url);
                Ok(Some(url))
            }
            Ok(None) => {
                debug!("Cache MISS: {}", short_code);
                Ok(None)
            }
            Err(e) => {
                error!("Redis GET error for {}: {}", short_code, e);
                Ok(None)
            }
        }
    }

    async fn set_url(
        &self,
        short_code: &str,
        original_url: &str,
        ttl: Option<usize>,
    ) -> CacheResult<()> {
        let key = self.build_key(short_code);
        let mut conn = self.client.clone();
        let ttl_seconds = ttl.unwrap_or(self.default_ttl);

        match conn
            .set_ex::<_, _, ()>(&key, original_url, ttl_seconds as u64)
            .await
        {
            Ok(_) => {
                debug!(
                    "Cache SET: {} -> {} (TTL: {}s)",
                    short_code, original_url, ttl_seconds
                );
                Ok(())
            }
            Err(e) => {
                warn!("Redis SET error for {}: {}", short_code, e);
                Ok(())
            }
        }
    }

    async fn invalidate(&self, short_code: &str) -> CacheResult<()> {
        let key = self.build_key(short_code);
        let mut conn = self.client.clone();

        match conn.del::<_, i32>(&key).await {
            Ok(deleted) => {
                if deleted > 0 {
                    debug!("Cache INVALIDATE: {}", short_code);
                }
                Ok(())
            }
            Err(e) => {
                warn!("Redis DEL error for {}: {}", short_code, e);
                Ok(())
            }
        }
    }

    async fn health_check(&self) -> bool {
        let mut conn = self.client.clone();
        conn.ping::<()>().await.is_ok()
    }
}
