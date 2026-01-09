use sqlx::PgPool;
use tokio::sync::mpsc;

use crate::domain::click_event::ClickEvent;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub base_url: String,
    pub click_tx: mpsc::Sender<ClickEvent>,
}
