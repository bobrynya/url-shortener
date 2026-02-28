//! Handlers for domain management endpoints.

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};

use crate::api::dto::domain::{
    CreateDomainRequest, DomainItem, DomainListResponse, UpdateDomainRequest,
};
use crate::domain::entities::{Domain, UpdateDomain};
use crate::error::AppError;
use crate::state::AppState;

fn domain_to_item(d: Domain) -> DomainItem {
    DomainItem {
        id: d.id,
        domain: d.domain,
        is_default: d.is_default,
        is_active: d.is_active,
        description: d.description,
        deleted_at: d.deleted_at,
        created_at: d.created_at,
        updated_at: d.updated_at,
    }
}

/// Lists all non-deleted domains.
///
/// # Endpoint
///
/// `GET /api/domains`
pub async fn domain_list_handler(
    State(state): State<AppState>,
) -> Result<Json<DomainListResponse>, AppError> {
    let all_domains = state.domain_service.list_domains(false).await?;

    Ok(Json(DomainListResponse {
        items: all_domains.into_iter().map(domain_to_item).collect(),
    }))
}

/// Creates a new domain.
///
/// # Endpoint
///
/// `POST /api/domains`
///
/// # Errors
///
/// Returns 400 if domain name is invalid.
/// Returns 409 if domain already exists.
pub async fn create_domain_handler(
    State(state): State<AppState>,
    Json(payload): Json<CreateDomainRequest>,
) -> Result<(StatusCode, Json<DomainItem>), AppError> {
    let domain = state
        .domain_service
        .create_domain(
            payload.domain,
            payload.is_default.unwrap_or(false),
            payload.description,
        )
        .await?;

    Ok((StatusCode::CREATED, Json(domain_to_item(domain))))
}

/// Partially updates a domain.
///
/// # Endpoint
///
/// `PATCH /api/domains/{id}`
///
/// All fields are optional. `description: null` clears the description.
/// `is_default: true` atomically transfers the default flag.
/// `is_default: false` is rejected — set another domain as default instead.
///
/// # Errors
///
/// Returns 400 if `is_default: false` is requested.
/// Returns 400 if domain name is invalid.
/// Returns 404 if domain not found.
pub async fn update_domain_handler(
    Path(id): Path<i64>,
    State(state): State<AppState>,
    Json(payload): Json<UpdateDomainRequest>,
) -> Result<Json<DomainItem>, AppError> {
    let update = UpdateDomain {
        domain: payload.domain,
        is_default: payload.is_default,
        is_active: payload.is_active,
        description: payload.description,
    };

    let domain = state.domain_service.update_domain(id, update).await?;

    Ok(Json(domain_to_item(domain)))
}

/// Soft-deletes a domain.
///
/// # Endpoint
///
/// `DELETE /api/domains/{id}`
///
/// Sets `deleted_at` on the domain (soft delete). The domain disappears from the
/// list API. All redirect requests for links under this domain return 410 Gone.
/// New links cannot be created for a deleted domain.
///
/// # Errors
///
/// Returns 400 if the domain is the system default.
/// Returns 400 if the domain has existing links.
/// Returns 404 if domain not found or already deleted.
pub async fn delete_domain_handler(
    Path(id): Path<i64>,
    State(state): State<AppState>,
) -> Result<StatusCode, AppError> {
    state.domain_service.delete_domain(id).await?;
    Ok(StatusCode::NO_CONTENT)
}
