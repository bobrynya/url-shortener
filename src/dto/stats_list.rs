use crate::dto::stats::StatsResponse;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct StatsListQuery {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

#[derive(Serialize)]
pub struct StatsListResponse {
    pub page: u32,
    pub page_size: u32,
    pub total: i64,
    pub items: Vec<StatsResponse>,
}
