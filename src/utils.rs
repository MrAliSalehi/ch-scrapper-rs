use std::ffi::OsStr;
use crate::config::AppConfig;
use grammers_client::types::Media;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use rand::Rng;


pub fn config_exists() -> bool {
    std::env::current_dir()
        .unwrap()
        .join("config.json")
        .exists()
}

pub fn is_valid(config: &AppConfig) -> bool {
    if config.api_hash.is_empty() || config.from.is_empty() || config.to.is_empty() {
        return false;
    }
    if config.api_hash.len() < 3
        || config.api_id < 10
        || config.from.is_empty()
        || config.to.len() < 3
        || config.phone.len() < 5
    {
        return false;
    }
    true
}

pub fn prompt(message: &str) -> Option<String> {
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    stdout.write_all(message.as_bytes()).unwrap();
    stdout.flush().unwrap();

    let stdin = std::io::stdin();
    let mut stdin = stdin.lock();

    let mut line = String::new();
    stdin.read_line(&mut line).unwrap();
    return Some(line);
}

pub fn file_extension(media: &Media) -> Option<&str> {
    if let Media::Document(doc) = media {
        return Path::new(doc.name()).extension().and_then(OsStr::to_str);
    }
    return None;
}

pub fn create_dir_if_not_exists(path: &str) -> Option<bool> {
    let unwrap = std::env::current_dir();
    if unwrap.is_err() {
        return None;
    }
    let current_dir = unwrap.unwrap();
    let final_path = current_dir.join(path);
    if !final_path.exists() {
        let create_result = std::fs::create_dir_all(&final_path);
        if create_result.is_err() {
            return None;
        }
        return Some(true);
    }
    return Some(true);
}

pub fn create_file_name_with_path(media: &Media, image_dir: &PathBuf) -> PathBuf {
    let extension = file_extension(&media).unwrap_or_else(|| "png");
    let random: i64 = rand::thread_rng().gen();
    let name = format!("{}", chrono::Utc::now().timestamp_nanos() + random);
    let random_hash = format!("{:x}", md5::compute(name));

    return Path::new(image_dir.to_str().unwrap()).join(format!("Pixoro-{}.{}", random_hash, extension));
}
