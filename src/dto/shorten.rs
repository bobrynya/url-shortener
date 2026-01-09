use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(untagged)]
pub enum ShortenInput {
    One(String),
    Many(Vec<String>),
}

#[derive(Serialize)]
pub struct Item {
    pub long_url: String,
    pub code: String,
    pub short_url: String,
}

#[derive(Serialize)]
pub struct ShortenResponse {
    pub items: Vec<Item>,
}
