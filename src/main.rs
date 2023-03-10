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
use grammers_client::types::Media::Document;
use grammers_tl_types::enums::MessagesFilter::InputMessagesFilterDocument;
use tokio::{spawn};
use std::error::Error;
mod account_manager;
mod config;
mod utils;

#[tokio::main]
async fn main() -> Result<(),Box<dyn Error>> {
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
        "Account:{},[{}-{}].\nFrom [{:?}] To [{}].",
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
    let to_chat = client.resolve_username(&config.to).await?.expect("couldnt resolve [to]");


    let image_dir = current_dir()?.join("images");
    let image_clone = image_dir.clone();
    let to_chat_clone = to_chat.clone();
    let client_clone = client.clone();
    let from_clone = config.from.clone();
    spawn(async move {
        let res = run_history_async(client, &to_chat, from_clone.as_str(), &image_dir).await;
        if let Err(e) = res {
            println!("{:#?}", e)
        }
    });

    handle_updates_async(config.from, to_chat_clone, &image_clone, client_clone).await?;

    network.await??;
    Ok(())
}

async fn run_history_async(client: Client, to_chat: &Chat, from: &str, image_dir: &PathBuf) -> Result<(),Box<dyn Error>> {
    let last_message = client
        .search_messages(to_chat)
        .query("id=").next().await.expect("could not get the next message");
    let mut last_message_id = 0;

    if last_message.is_some() {
        let msg = last_message.expect("failed to get last message.");
        let split = msg.text().split('=').collect::<Vec<&str>>()[1];
        last_message_id = split.parse::<i32>().unwrap_or(0);
    }

    let from_chat = client.resolve_username(from).await?.expect("failed to resolve [from]");
    let mut messages = client
        .search_messages(from_chat)
        .filter(InputMessagesFilterDocument)
        .offset_id(last_message_id);

    while let Some(message) = messages.next().await.expect("failed to get the next message[history]") {
        let caption = format!("id={}", message.id());
        let media = message.media().unwrap();

        continue_on_error!(download_rename_send_media(&client, &media, image_dir, &to_chat, Some(caption.as_str())).await);

    }
    Ok(())
}

async fn handle_updates_async(from: String, chat: Chat, image_dir: &PathBuf, client: Client) -> Result<(),Box<dyn Error>> {
    while let Some(update) = client.next_update().await? {
        match update {
            Update::NewMessage(message) if !message.outgoing() => {
                if let Channel(ch) = message.chat() {
                    if ch.username().is_none() {
                        continue;
                    }
                    if ch.username().unwrap() != from {
                        continue;
                    }
                    if message.media().is_none() {
                        continue;
                    }
                    let media =message.media().expect("failed to unwrap media [handle_update]");
                    continue_on_error!(download_rename_send_media(&client, &media, image_dir, &chat, None).await)
                }
            }
            _ => {}
        }
    }
    Ok(())
}

async fn download_rename_send_media(client: &Client, media: &Media, image_dir: &PathBuf, to: &Chat, caption: Option<&str>) -> Result<(),Box<dyn Error>> {
    if let Document(doc) = media {
        if !doc.mime_type().expect("cant unwrap mime type").starts_with("image") {
            return Ok(());
        }
        if doc.size() > 10100000 {
            return Ok(());
        }
        let path = create_file_name_with_path(&media, image_dir);
        client.download_media(&media, &path).await.expect("couldnt download the media");

        let uploaded = client.upload_file(&path).await.expect("couldnt upload the file");

        let message = InputMessage::document(InputMessage::text(caption.unwrap_or("")), uploaded);
        let send = client.send_message(to, message).await;
        async_std::task::sleep(Duration::from_secs(5)).await;
        if send.is_ok() {
            async_std::fs::remove_file(&path).await.expect("couldn't remove the file");
        }
    }
    Ok(())
}