//! Handler for health check endpoint.

use axum::{Json, extract::State, http::StatusCode};

use crate::api::dto::health::{CheckStatus, HealthChecks, HealthResponse};
use crate::state::AppState;

/// Returns service health status with component checks.
///
/// # Endpoint
///
/// `GET /api/health`
///
/// # Response Codes
///
/// - **200 OK**: All components healthy
/// - **503 Service Unavailable**: One or more components degraded
///
/// # Components Checked
///
/// 1. **Database**: Tests default domain query
/// 2. **Click Queue**: Checks if channel is open and reports capacity
/// 3. **Cache**: Tests Redis PING
///
/// # Response
///
/// ```json
/// {
///   "status": "healthy",
///   "version": "0.1.0",
///   "checks": {
///     "database": {
///       "status": "ok",
///       "message": "Connected, default domain: s.example.com"
///     },
///     "click_queue": {
///       "status": "ok",
///       "message": "Capacity: 10000"
///     },
///     "cache": {
///       "status": "ok",
///       "message": "Redis connected"
///     }
///   }
/// }
/// ```
pub async fn health_handler(
    State(state): State<AppState>,
) -> Result<Json<HealthResponse>, (StatusCode, Json<HealthResponse>)> {
    let db_check = check_database(&state).await;

    let queue_check = check_click_queue(&state);

    let cache_check = check_cache(&state).await;

    let all_healthy =
        db_check.status == "ok" && queue_check.status == "ok" && cache_check.status == "ok";

    let response = HealthResponse {
        status: if all_healthy { "healthy" } else { "degraded" }.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        checks: HealthChecks {
            database: db_check,
            click_queue: queue_check,
            cache: cache_check,
        },
    };

    if all_healthy {
        Ok(Json(response))
    } else {
        Err((StatusCode::SERVICE_UNAVAILABLE, Json(response)))
    }
}

/// Checks database connectivity by querying the default domain.
async fn check_database(state: &AppState) -> CheckStatus {
    match state.domain_service.get_default_domain().await {
        Ok(domain) => CheckStatus {
            status: "ok".to_string(),
            message: Some(format!("Connected, default domain: {}", domain.domain)),
        },
        Err(e) => CheckStatus {
            status: "error".to_string(),
            message: Some(format!("Database error: {}", e)),
        },
    }
}

/// Checks if the click tracking queue is operational.
fn check_click_queue(state: &AppState) -> CheckStatus {
    if state.click_sender.is_closed() {
        CheckStatus {
            status: "error".to_string(),
            message: Some("Click queue is closed".to_string()),
        }
    } else {
        CheckStatus {
            status: "ok".to_string(),
            message: Some(format!("Capacity: {}", state.click_sender.capacity())),
        }
    }
}

/// Checks cache connectivity via PING command.
async fn check_cache(state: &AppState) -> CheckStatus {
    if state.cache.health_check().await {
        CheckStatus {
            status: "ok".to_string(),
            message: Some("Redis connected".to_string()),
        }
    } else {
        CheckStatus {
            status: "error".to_string(),
            message: Some("Redis connection failed".to_string()),
        }
    }
}
