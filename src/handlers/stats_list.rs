use axum::{
    extract::{Query, State},
    Json,
};
use serde_json::json;

use crate::{
    dto::{
        stats::StatsResponse,
        stats_list::{StatsListQuery, StatsListResponse},
    },
    error::{map_sqlx_error, AppError},
    state::AppState,
};

pub async fn stats_list(
    State(st): State<AppState>,
    Query(q): Query<StatsListQuery>,
) -> Result<Json<StatsListResponse>, AppError> {
    let page = q.page.unwrap_or(1);
    if page == 0 {
        return Err(AppError::bad_request(
            "page must be >= 1",
            json!({"field": "page", "min": 1}),
        ));
    }

    let page_size = q.page_size.unwrap_or(25);
    if !(10..=50).contains(&page_size) {
        return Err(AppError::bad_request(
            "page_size must be in [10..50]",
            json!({"field": "page_size", "min": 10, "max": 50}),
        ));
    }

    let limit: i64 = page_size as i64;
    let offset: i64 = ((page - 1) as i64) * limit;

    let total_row = sqlx::query!(r#"SELECT COUNT(*)::bigint AS "total!" FROM links"#)
        .fetch_one(&st.db)
        .await
        .map_err(map_sqlx_error)?;
    let total = total_row.total;

    let rows = sqlx::query!(
        r#"
        SELECT long_url, code, clicks as "clicks!", created_at as "created_at!"
        FROM links
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
        "#,
        limit,
        offset
    )
    .fetch_all(&st.db)
    .await
    .map_err(map_sqlx_error)?;

    let items = rows
        .into_iter()
        .map(|r| StatsResponse {
            long_url: r.long_url,
            code: r.code,
            clicks: r.clicks,
            created_at: r.created_at,
        })
        .collect();

    Ok(Json(StatsListResponse {
        page,
        page_size,
        total,
        items,
    }))
}
