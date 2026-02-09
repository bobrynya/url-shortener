//! Application layer services implementing business logic.
//!
//! This layer orchestrates domain operations by coordinating repository calls,
//! validation, and business rules. Services consume repository traits and provide
//! a clean API for HTTP handlers.
//!
//! # Available Services
//!
//! - [`services::link_service::LinkService`] - Short link creation and retrieval
//! - [`services::stats_service::StatsService`] - Click tracking and analytics
//! - [`services::auth_service::AuthService`] - API token authentication
//! - [`services::domain_service::DomainService`] - Domain management and validation

pub mod services;
