//! Infrastructure layer for external integrations.
//!
//! This layer implements interfaces defined by the domain layer, providing
//! concrete implementations for data persistence and caching.
//!
//! # Modules
//!
//! - [`cache`] - Caching abstractions (Redis and no-op implementations)
//! - [`persistence`] - PostgreSQL repository implementations

pub mod cache;
pub mod persistence;
