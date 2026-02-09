//! PostgreSQL repository implementations.
//!
//! Concrete implementations of domain repository traits using SQLx for type-safe
//! SQL queries with compile-time verification.
//!
//! # Repositories
//!
//! - [`PgLinkRepository`] - Link storage and retrieval
//! - [`PgStatsRepository`] - Click tracking and analytics queries
//! - [`PgDomainRepository`] - Domain management
//! - [`PgTokenRepository`] - API token storage and validation

pub mod pg_domain_repository;
pub mod pg_link_repository;
pub mod pg_stats_repository;
pub mod pg_token_repository;

pub use pg_domain_repository::PgDomainRepository;
pub use pg_link_repository::PgLinkRepository;
pub use pg_stats_repository::PgStatsRepository;
pub use pg_token_repository::PgTokenRepository;
