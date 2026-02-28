//! DTOs for domain management.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

/// Individual domain information (used in all domain responses).
#[derive(Debug, Serialize)]
pub struct DomainItem {
    pub id: i64,
    pub domain: String,
    pub is_default: bool,
    pub is_active: bool,
    pub description: Option<String>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Response containing list of domains.
#[derive(Debug, Serialize)]
pub struct DomainListResponse {
    pub items: Vec<DomainItem>,
}

/// Request body for `POST /api/domains`.
#[derive(Debug, Deserialize)]
pub struct CreateDomainRequest {
    pub domain: String,
    /// When true, this domain becomes the system default. Defaults to false.
    pub is_default: Option<bool>,
    pub description: Option<String>,
}

/// Request body for `PATCH /api/domains/{id}`.
///
/// All fields are optional — only provided fields are changed.
///
/// # `description` semantics
///
/// - Absent → leave unchanged
/// - `null` → clear (set to NULL)
/// - String value → set to that value
#[serde_as]
#[derive(Debug, Deserialize)]
pub struct UpdateDomainRequest {
    pub domain: Option<String>,
    pub is_default: Option<bool>,
    pub is_active: Option<bool>,
    #[serde(default, with = "::serde_with::rust::double_option")]
    pub description: Option<Option<String>>,
}
