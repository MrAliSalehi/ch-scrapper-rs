use std::io::{BufRead, Write};
use std::path::PathBuf;
use grammers_client::types::Media;
use grammers_client::types::Media::Document;
use mime::Mime;
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

pub fn file_extension(media: &Media) -> Option<String> {
    if let Document(document) = media {
        let extension = document.mime_type().map(|m| {
            let mime: Mime = m.parse().unwrap();
            format!(".{}", mime.subtype().to_string())
        });

        if extension.is_some() {
            return Some(extension.unwrap());
        }
        return None;
    }
    return None;
}

pub fn file_name(media: &Media) -> Option<&str> {
    if let Document(document) = media {
        return Some(document.name());
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
    if !final_path.exists()
    {
        let create_result = std::fs::create_dir_all(&final_path);
        if create_result.is_err()
        {
            return None;
        }
        return Some(true);
    }
    return Some(true);
}

pub fn create_file_name_with_path(media: &Media, image_dir: &str) -> PathBuf {
    let extension = file_extension(&media)
        .expect("couldn't find the file extension.");
    let file_name = file_name(&media)
        .expect("couldn't find the file name.");

    let random_hash = format!("{:x}", md5::compute(file_name));

    return std::path::Path::new(image_dir).join(format!("Pixoro-{}{}", random_hash, extension));
}
