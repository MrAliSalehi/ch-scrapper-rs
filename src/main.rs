use serde_json;
use crate::config::AppConfig;

mod config;

fn main() {
    if !config_exists()
    {
        println!("Config file not found");
        return;
    }
    let content = std::fs::read_to_string("config.json").expect("Failed to read config file");

    let config: AppConfig = serde_json::from_str(&content).expect("Failed To parse config,invalid json format.");
    if !is_valid(&config)
    {
        println!("Invalid config data");
        return;
    }
    println!("Account:{},[{}-{}].\nFrom [{}] To [{}].", config.phone, config.api_hash, config.api_id, config.from, config.to);
}

fn config_exists() -> bool {
    std::env::current_dir().unwrap().join("config.json").exists()
}

fn is_valid(config: &AppConfig) -> bool {
    if config.api_hash.is_empty() || config.from.is_empty() || config.to.is_empty()
    {
        return false;
    }
    if config.api_hash.len() < 3 || config.api_id < 10 || config.from.len() < 3 || config.to.len() < 3 || config.phone.len() < 5 {
        return false;
    }
    true
}