//! Cache service trait and error types.

use async_trait::async_trait;
use std::fmt;

/// Errors that can occur during cache operations.
#[derive(Debug)]
pub enum CacheError {
    ConnectionError(String),
    OperationError(String),
}

impl fmt::Display for CacheError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ConnectionError(e) => write!(f, "Cache connection error: {}", e),
            Self::OperationError(e) => write!(f, "Cache operation error: {}", e),
        }
    }
}

impl std::error::Error for CacheError {}

/// Result type for cache operations.
pub type CacheResult<T> = Result<T, CacheError>;

/// Trait for caching short URL mappings.
///
/// Implementations must be thread-safe and handle errors gracefully without
/// disrupting the application (cache failures should degrade to database lookups).
///
/// # Implementations
///
/// - [`crate::infrastructure::cache::RedisCache`] - Redis-backed cache with TTL support
/// - [`crate::infrastructure::cache::NullCache`] - No-op implementation for disabled caching
#[async_trait]
pub trait CacheService: Send + Sync {
    /// Retrieves the original URL for a short code from cache.
    ///
    /// # Returns
    ///
    /// - `Ok(Some(url))` on cache hit
    /// - `Ok(None)` on cache miss or error (fail-open behavior)
    ///
    /// # Errors
    ///
    /// Should not return errors in production implementations. Errors are logged
    /// and treated as cache misses.
    async fn get_url(&self, short_code: &str) -> CacheResult<Option<String>>;

    /// Stores a URL mapping in cache with optional TTL.
    ///
    /// # Arguments
    ///
    /// - `short_code` - The short code key
    /// - `original_url` - The full URL to cache
    /// - `ttl_seconds` - Optional TTL in seconds (implementation-specific default if None)
    ///
    /// # Errors
    ///
    /// Should not propagate errors to callers. Implementations should log errors
    /// and return `Ok(())` to avoid disrupting the request flow.
    async fn set_url(
        &self,
        short_code: &str,
        original_url: &str,
        ttl_seconds: Option<usize>,
    ) -> CacheResult<()>;

    /// Removes a cached URL mapping.
    ///
    /// Used when a link is deleted or modified.
    ///
    /// # Errors
    ///
    /// Should not propagate errors to callers.
    async fn invalidate(&self, short_code: &str) -> CacheResult<()>;

    /// Checks if the cache backend is healthy.
    ///
    /// Used by health check endpoints to report cache status.
    async fn health_check(&self) -> bool;
}
