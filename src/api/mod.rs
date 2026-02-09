//! REST API layer for HTTP request/response handling.
//!
//! This layer translates HTTP requests into domain operations and formats
//! responses according to API contracts.
//!
//! # Modules
//!
//! - [`dto`] - Data Transfer Objects for request/response serialization
//! - [`handlers`] - HTTP request handlers
//! - [`middleware`] - Authentication and request processing middleware
//! - [`routes`] - Route configuration and composition

pub mod dto;
pub mod handlers;
pub mod middleware;
pub mod routes;
