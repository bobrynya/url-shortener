//! Caching layer for fast redirect lookups.
//!
//! Provides a [`CacheService`] trait with two implementations:
//! - [`RedisCache`] - Production Redis-backed cache
//! - [`NullCache`] - No-op implementation for testing/disabled caching

mod null_cache;
mod redis_cache;
mod service;

pub use null_cache::NullCache;
pub use redis_cache::RedisCache;
pub use service::{CacheError, CacheResult, CacheService};
