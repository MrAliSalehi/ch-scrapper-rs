use std::env::current_dir;
use grammers_client::{Client, Config, InitParams, InputMessage, Update};
use grammers_client::types::Chat::Channel;
use grammers_client::types::{Chat, Message};
use grammers_session::{Session};
use serde_json;
use crate::config::AppConfig;
use tokio::{runtime, task};
use crate::account_manager::{*};
use crate::utils::{*};

mod config;
mod utils;
mod account_manager;


type AsyncResult = Result<(), Box<dyn std::error::Error>>;


fn main() -> AsyncResult {
    runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(main_async())
}

async fn main_async() -> AsyncResult {
    if !config_exists()
    {
        println!("Config file not found");
        return Ok(());
    }
    let content = std::fs::read_to_string("config.json").expect("Failed to read config file");

    let config: AppConfig = serde_json::from_str(&content).expect("Failed To parse config,invalid json format.");
    if !is_valid(&config)
    {
        panic!("Invalid config data");
    }
    println!("Account:{},[{}-{}].\nFrom [{}] To [{}].", config.phone, config.api_hash, config.api_id, config.from, config.to);

    let login = Client::connect(Config {
        api_hash: config.api_hash.clone(),
        api_id: config.api_id,
        params: InitParams {
            catch_up: true,
            ..Default::default()
        },
        session: Session::load_file_or_create(SESSION_FILE).expect("Failed to create session"),
    }).await;
    if login.is_err() {
        panic!("failed to connect to the telegram");
    }
    let client_handler = login.expect("failed to create client");

    if !client_handler.is_authorized().await.unwrap() {
        println!("you are not authorized,requesting verification code");

        let signed_in = sign_in_async(&config, &client_handler).await;

        check_status(&client_handler, signed_in).await;

        save_session(&client_handler)
    }
    create_dir_if_not_exists("images").expect("failed to create images directory.");

    println!("signed in,getting updates...");
    let client = client_handler.clone();
    let network = task::spawn(async move { client_handler.run_until_disconnected().await });

    handle_updates_async(&config, client).await.expect("failed to handle updates");

    network.await??;
    Ok(())
}

async fn handle_updates_async(conf: &AppConfig,  client: Client) -> AsyncResult {
    let image_dir = current_dir()?.join("images");
    let to = client.resolve_username(conf.to).await.expect("couldn't resolve the username").unwrap();
    while let Some(update) = client.next_update().await? {
        match update {
            Update::NewMessage(message) if !message.outgoing() => {
                handle_new_message(&conf.from,&to, message, &image_dir.to_str().unwrap(), &client).await;
            }
            _ => {}
        }
    }
    Ok(())
}

async fn handle_new_message(from: &str,to: &Chat, message: Message, image_dir: &str, client: &Client) {
    match message.chat() {
        Channel(ch) if ch.username().unwrap().to_string() == from => {
            if message.media().is_none() {
                return;
            }
            let media = message.media().unwrap();
            let path = create_file_name_with_path(&media, image_dir);
            client.download_media(&media, &path).await.expect("couldn't download the media");

            let uploaded = client.upload_file(path).await.expect("couldn't upload the file");
            let message = InputMessage::document(InputMessage::text("doc"), uploaded);
            client.send_message(to, message).await.expect("couldn't send the file");
        }
        _ => {}
    }
}
