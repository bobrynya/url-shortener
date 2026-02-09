//! No-op cache implementation for testing or disabled caching.

use super::service::{CacheResult, CacheService};
use async_trait::async_trait;
use tracing::debug;

/// A cache implementation that does nothing.
///
/// Used when Redis is unavailable or caching is explicitly disabled.
/// All operations succeed immediately without storing or retrieving data.
///
/// # Use Cases
///
/// - Development environments without Redis
/// - Testing scenarios where caching should be bypassed
/// - Fallback when Redis connection fails at startup
pub struct NullCache;

impl NullCache {
    /// Creates a new NullCache instance.
    pub fn new() -> Self {
        debug!("Using NullCache (caching disabled)");
        Self
    }
}

impl Default for NullCache {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CacheService for NullCache {
    async fn get_url(&self, _short_code: &str) -> CacheResult<Option<String>> {
        Ok(None)
    }

    async fn set_url(
        &self,
        _short_code: &str,
        _original_url: &str,
        _ttl: Option<usize>,
    ) -> CacheResult<()> {
        Ok(())
    }

    async fn invalidate(&self, _short_code: &str) -> CacheResult<()> {
        Ok(())
    }

    async fn health_check(&self) -> bool {
        true
    }
}
