//! Utility functions for code generation, URL processing, and request handling.
//!
//! This module provides helper functions used across the application:
//!
//! - [`code_generator`] - Short code generation and validation
//! - [`url_normalizer`] - URL normalization and sanitization
//! - [`extract_domain`] - Domain extraction from HTTP headers

pub mod code_generator;
pub mod extract_domain;
pub mod url_normalizer;
