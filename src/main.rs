use std::env::current_dir;
use std::path::{Path};
use grammers_client::{Client, Config, InitParams, SignInError, Update};
use grammers_client::types::Chat::Channel;
use grammers_client::types::{Message};
use grammers_session::{Session};
use serde_json;
use crate::config::AppConfig;
use tokio::{runtime, task};
use crate::utils::{config_exists, create_dir_if_not_exists, file_extension, file_name, is_valid, prompt};

mod config;
mod utils;

const SESSION_FILE: &str = "scrapper.session";

type Result = std::result::Result<(), Box<dyn std::error::Error>>;


fn main() -> Result {
    runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(main_async())
}

async fn main_async() -> Result {
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
        let login_token = client_handler
            .request_login_code(&config.phone, config.api_id, &config.api_hash).await
            .expect("failed to send code");
        let code = prompt("Enter Code:").expect("failed to get the code");

        let signed_in = client_handler.sign_in(&login_token, &code).await;

        match signed_in {
            Err(SignInError::PasswordRequired(password_token)) => {
                let hint = password_token.hint().unwrap_or("None");
                let prompt_message = format!("Enter the password (hint {}): ", &hint);
                let password = prompt(prompt_message.as_str()).expect("failed to get the password");
                client_handler.check_password(password_token, password.trim()).await?;
            }
            Ok(user) => {
                println!("logged in with user:{},id:{}", user.username().unwrap(), user.id());
            }
            Err(e) => panic!("{}", e),
        };
        match client_handler.session().save_to_file(SESSION_FILE) {
            Ok(_) => { println!("session saved to: {}", SESSION_FILE) }
            Err(e) => {
                println!("NOTE: failed to save the session[{}],you will sign out when program stops working", e);
            }
        }
    }
    create_dir_if_not_exists("images").expect("failed to create images directory.");

    println!("signed in");
    let client = client_handler.clone();
    let network = task::spawn(async move { client_handler.run_until_disconnected().await });
    let image_dir = current_dir().unwrap().join("images");

    handle_updates_async(&config.from, &image_dir.to_str().unwrap(), client).await.expect("failed to handle updates");

    network.await??;
    Ok(())
}

async fn handle_updates_async(from: &str, image_dir: &str, client: Client) -> Result {
    while let Some(update) = client.next_update().await? {
        match update {
            Update::NewMessage(message) if !message.outgoing() => {
                handle_new_message(&from, message, &image_dir, &client).await;
            }
            _ => {}
        }
    }
    Ok(())
}

async fn handle_new_message(from: &str, message: Message, image_dir: &str, client: &Client) {
    match message.chat() {
        Channel(ch) if ch.username().unwrap().to_string() == from => {
            if message.media().is_none() {
                return;
            }
            let media = message.media().unwrap();
            let extension = file_extension(&media).expect("couldn't find the file extension.");
            let file_name = file_name(&media).expect("couldn't find the file name.");
            let random_hash = format!("{:x}", md5::compute(file_name));
            let path = Path::new(image_dir).join(format!("Pixoro-{}{}", random_hash, extension));
            println!("addr:{}", path.to_str().unwrap());

            client.download_media(&media, &path).await.expect("couldn't download the media");
        }
        _ => {}
    }
}



