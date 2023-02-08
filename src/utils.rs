use crate::config::AppConfig;

pub fn config_exists() -> bool {
    std::env::current_dir().unwrap().join("config.json").exists()
}

pub fn is_valid(config: &AppConfig) -> bool {
    if config.api_hash.is_empty() || config.from.is_empty() || config.to.is_empty()
    {
        return false;
    }
    if config.api_hash.len() < 3 || config.api_id < 10 || config.from.len() < 3 || config.to.len() < 3 || config.phone.len() < 5 {
        return false;
    }
    true
}