//! Domain layer containing business entities and logic.
//!
//! This module implements the core domain logic following Clean Architecture principles.
//! It defines entities, repository interfaces, and domain services independent of
//! infrastructure concerns.
//!
//! # Architecture
//!
//! - [`entities`] - Core business data structures
//! - [`repositories`] - Data access trait definitions
//! - [`click_event`] - Click tracking event model
//! - [`click_worker`] - Asynchronous click processing worker
//!
//! # Design Principles
//!
//! - Domain layer has no dependencies on infrastructure or presentation layers
//! - Repository traits define contracts implemented by infrastructure layer
//! - Business logic is encapsulated in services (see [`crate::application::services`])
//!
//! # Click Processing Flow
//!
//! 1. HTTP handler receives redirect request
//! 2. [`click_event::ClickEvent`] is sent to async channel
//! 3. [`click_worker::run_click_worker`] processes events with retry logic
//! 4. Click data is persisted via [`repositories::StatsRepository`]

pub mod click_event;
pub mod click_worker;
pub mod entities;
pub mod repositories;
