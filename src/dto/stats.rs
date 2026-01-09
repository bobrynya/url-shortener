use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Serialize)]
pub struct StatsResponse {
    pub long_url: String,
    pub code: String,
    pub clicks: i64,
    pub created_at: DateTime<Utc>,
}
