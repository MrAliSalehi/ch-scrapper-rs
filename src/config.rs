use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub api_hash: String,
    pub api_id: i32,
    pub phone: String,
    pub from: String,
    pub to: String,
}
