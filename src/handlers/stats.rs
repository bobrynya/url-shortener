use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use serde_json::json;

use crate::{
    dto::clicks::{ClickItem, ClicksResponse},
    error::{map_sqlx_error, AppError},
    state::AppState,
};

#[derive(Deserialize)]
pub struct PageQuery {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

pub async fn stats_by_code(
    State(st): State<AppState>,
    Path(code): Path<String>,
    Query(q): Query<PageQuery>,
) -> Result<Json<ClicksResponse>, AppError> {
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

    // 0) Находим link_id по code (или 404)
    let link = sqlx::query!(
        r#"
        SELECT id
        FROM links
        WHERE code = $1
        "#,
        code
    )
    .fetch_optional(&st.db)
    .await
    .map_err(map_sqlx_error)?;

    let link_id = match link {
        Some(r) => r.id,
        None => return Err(AppError::not_found("Unknown code", json!({"code": code}))),
    };

    // 1) total кликов по ссылке
    let total_row = sqlx::query!(
        r#"
        SELECT COUNT(*)::bigint AS "total!"
        FROM link_clicks
        WHERE link_id = $1
        "#,
        link_id
    )
    .fetch_one(&st.db)
    .await
    .map_err(map_sqlx_error)?;

    let total = total_row.total;

    // 2) страница кликов
    let rows = sqlx::query!(
        r#"
        SELECT id, clicked_at, referer, user_agent, ip
        FROM link_clicks
        WHERE link_id = $1
        ORDER BY clicked_at DESC, id DESC
        LIMIT $2 OFFSET $3
        "#,
        link_id,
        limit,
        offset
    )
    .fetch_all(&st.db)
    .await
    .map_err(map_sqlx_error)?;

    let items = rows
        .into_iter()
        .map(|r| ClickItem {
            id: r.id,
            clicked_at: r.clicked_at,
            referer: r.referer,
            user_agent: r.user_agent,
            ip: r.ip.map(|v| v.to_string()),
        })
        .collect();

    Ok(Json(ClicksResponse {
        code,
        page,
        page_size,
        total,
        items,
    }))
}
