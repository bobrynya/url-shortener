//! Data Transfer Objects for API requests and responses.
//!
//! All DTOs use Serde for JSON serialization/deserialization and validator
//! for input validation.

pub mod clicks;
pub mod domain;
pub mod health;
pub mod pagination;
pub mod shorten;
pub mod stats;
pub mod stats_list;
