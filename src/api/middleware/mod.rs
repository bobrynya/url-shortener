//! HTTP middleware for request processing and protection.
//!
//! Provides authentication, rate limiting, and observability middleware.

pub mod auth;
pub mod rate_limit;
pub mod tracing;
