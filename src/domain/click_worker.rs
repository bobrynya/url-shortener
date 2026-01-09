use sqlx::PgPool;
use tokio::sync::mpsc;

use crate::domain::click_event::ClickEvent;

pub async fn run_click_worker(mut rx: mpsc::Receiver<ClickEvent>, db: PgPool) {
    while let Some(ev) = rx.recv().await {
        let _ = sqlx::query!(
            r#"
    INSERT INTO link_clicks (link_id, clicked_at, referer, user_agent, ip)
    VALUES ($1, $2, $3, $4, $5)
    "#,
            ev.link_id,
            ev.clicked_at,
            ev.referer,
            ev.user_agent,
            ev.ip, // Option<IpNetwork>
        )
        .execute(&db)
        .await;
        // Здесь лучше: обработка ошибок + retry/backoff + метрики dropped/failed.
    }
}
