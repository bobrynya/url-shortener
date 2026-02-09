//! Web dashboard layer for browser-based UI.
//!
//! Provides HTML pages for link management and statistics visualization.
//! Uses Askama templates for server-side rendering.
//!
//! # Modules
//!
//! - [`handlers`] - Template rendering handlers
//! - [`middleware`] - Web-specific middleware (auth, session)
//! - [`routes`] - Dashboard route configuration

pub mod handlers;
pub mod middleware;
pub mod routes;
