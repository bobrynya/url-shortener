//! DTOs for domain management.

use chrono::{DateTime, Utc};
use serde::Serialize;

/// Individual domain information.
#[derive(Debug, Serialize)]
pub struct DomainItem {
    pub domain: String,
    pub is_default: bool,
    pub is_active: bool,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Response containing list of domains.
#[derive(Debug, Serialize)]
pub struct DomainListResponse {
    pub items: Vec<DomainItem>,
}
