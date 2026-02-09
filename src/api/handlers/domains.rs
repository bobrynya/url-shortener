//! Handler for domain listing.

use crate::api::dto::domain::{DomainItem, DomainListResponse};
use crate::error::AppError;
use crate::state::AppState;
use axum::{Json, extract::State};

/// Lists all configured domains.
///
/// # Endpoint
///
/// `GET /api/domains`
///
/// # Response
///
/// Returns all domains (active and inactive) sorted by default status.
///
/// ```json
/// {
///   "items": [
///     {
///       "domain": "s.example.com",
///       "is_default": true,
///       "is_active": true,
///       "description": "Primary short domain",
///       "created_at": "2024-01-01T00:00:00Z",
///       "updated_at": "2024-01-01T00:00:00Z"
///     }
///   ]
/// }
/// ```
pub async fn domain_list_handler(
    State(state): State<AppState>,
) -> Result<Json<DomainListResponse>, AppError> {
    let all_domains = state.domain_service.list_domains(false).await?;

    let items = all_domains
        .into_iter()
        .map(|domain| DomainItem {
            domain: domain.domain,
            is_default: domain.is_default,
            is_active: domain.is_active,
            description: domain.description,
            created_at: domain.created_at,
            updated_at: domain.updated_at,
        })
        .collect();

    Ok(Json(DomainListResponse { items }))
}
