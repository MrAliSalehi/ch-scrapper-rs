use crate::account_manager::*;
use crate::config::AppConfig;
use crate::utils::*;
use grammers_client::types::Chat::Channel;
use grammers_client::types::{Chat, Media};
use grammers_client::{Client, Config, InitParams, InputMessage, Update};
use grammers_session::{Session};
use serde_json;
use std::env::current_dir;
use std::path::PathBuf;
use std::time::Duration;
use grammers_tl_types::enums::{MessageMedia};
use grammers_tl_types::enums::MessagesFilter::InputMessagesFilterDocument;
use tokio::{spawn};

mod account_manager;
mod config;
mod utils;

type AsyncResult = Result<(), Box<dyn std::error::Error>>;


#[tokio::main]
async fn main() -> AsyncResult {
    if !config_exists() {
        println!("Config file not found");
        return Ok(());
    }
    let content = std::fs::read_to_string("config.json").expect("Failed to read config file");

    let config: AppConfig =
        serde_json::from_str(&content).expect("Failed To parse config,invalid json format.");

    if !is_valid(&config) {
        panic!("Invalid config data");
    }
    println!(
        "Account:{},[{}-{}].\nFrom [{}] To [{}].",
        config.phone, config.api_hash, config.api_id, config.from, config.to
    );

    let login = Client::connect(Config {
        api_hash: config.api_hash.clone(),
        api_id: config.api_id,
        params: InitParams {
            catch_up: true,
            ..Default::default()
        },
        session: Session::load_file_or_create(SESSION_FILE).expect("Failed to create session"),
    })
        .await;
    if login.is_err() {
        panic!("failed to connect to the telegram");
    }
    let client_handler = login.expect("failed to create client");

    if !client_handler
        .is_authorized()
        .await
        .expect("couldnt get authorization status")
    {
        println!("you are not authorized,requesting verification code");

        let signed_in = sign_in_async(&config, &client_handler).await;

        check_status(&client_handler, signed_in).await;

        save_session(&client_handler)
    }
    create_dir_if_not_exists("images").expect("failed to create images directory.");

    println!("signed in,getting updates...");
    let client = client_handler.clone();
    let network = spawn(async move { client_handler.run_until_disconnected().await });
    let target_chat = client.resolve_username(&config.to).await
        .expect("couldn't resolve the username[destination channel]").unwrap();

    let image_dir = current_dir()?.join("images");

    let image_clone = image_dir.clone();
    let chat_clone = target_chat.clone();
    let client_clone = client.clone();

    /*spawn(async move {
        //run_history_async(client, &target_chat, &image_dir).await.unwrap();
    });
    println!("out of history");
    */
    handle_updates_async(&config, chat_clone, &image_clone, client_clone).await?;

    network.await??;
    Ok(())
}

async fn run_history_async(client: Client, chat: &Chat, image_dir: &PathBuf) -> AsyncResult {
    let mut messages = client.search_messages(chat).filter(InputMessagesFilterDocument);

    let mut last_message_id = 0;
    while let Some(message) = messages.next().await? {
        download_rename_send_media(&client, &message.media().unwrap(), image_dir, &chat).await
            .expect("failed to process media");
        async_std::task::sleep(Duration::from_millis(1300)).await;
        last_message_id = message.id();
    }
    println!("last message id was:{}", last_message_id);
    Ok(())
}

async fn handle_updates_async(conf: &AppConfig, chat: Chat, image_dir: &PathBuf, client: Client) -> AsyncResult {
    while let Some(update) = client.next_update().await? {
        match update {
            Update::NewMessage(message) if !message.outgoing() => {
                if let Channel(ch) = message.chat() {
                    if ch.username().is_none() {
                        continue;
                    }
                    if ch.username().unwrap() != conf.from {
                        continue;
                    }
                    if message.media().is_none() {
                        continue;
                    }
                    download_rename_send_media(&client, &message.media().unwrap(), image_dir, &chat).await
                        .expect("failed to process media");
                }
            }
            _ => {}
        }
        async_std::task::sleep(Duration::from_secs(1)).await;
    }
    Ok(())
}

async fn download_rename_send_media(client: &Client, media: &Media, image_dir: &PathBuf, to: &Chat) -> AsyncResult {
    let path = create_file_name_with_path(&media, image_dir);
    client.download_media(&media, &path).await
        .expect("couldn't download the media");

    let uploaded = client.upload_file(&path).await
        .expect("couldn't upload the file");

    let message = InputMessage::document(InputMessage::text(""), uploaded);
    let send = client.send_message(to, message).await;
    if send.is_ok() {
        async_std::fs::remove_file(&path).await
            .expect("couldn't remove the file");
        return Ok(());
    }
    panic!("couldn't send the file");
}
