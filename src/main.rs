use std::io::{BufRead, Write};
use grammers_client::{Client, Config, InitParams, SignInError};
use grammers_session::{Session};
use serde_json;
use crate::config::AppConfig;
use tokio::{runtime};
use crate::utils::{config_exists, is_valid};

mod config;
mod utils;

const SESSION_FILE: &str = "scrapper.session";
type Result = std::result::Result<(), Box<dyn std::error::Error>>;
async fn main_async()->Result {
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
    if login.is_err(){
        panic!("failed to connect to the telegram");
    }
    let client = login.expect("failed to create client");

    if !client.is_authorized().await.unwrap(){
        println!("you are not authorized,requesting verification code");
        let login_token =client
            .request_login_code(&config.phone,config.api_id,&config.api_hash).await
            .expect("failed to send code");
        let code = prompt("Enter Code:").expect("failed to get the code");

        let signed_in = client.sign_in(&login_token,&code).await;

        match signed_in {
            Err(SignInError::PasswordRequired(password_token)) => {

                let hint = password_token.hint().unwrap_or("None");
                let prompt_message = format!("Enter the password (hint {}): ", &hint);
                let password = prompt(prompt_message.as_str()).expect("failed to get the password");
                client.check_password(password_token, password.trim()).await?;
            }
            Ok(user) => {
                println!("logged in with user:{},id:{}",user.username().unwrap(),user.id());
            },
            Err(e) => panic!("{}", e),
        };
        match client.session().save_to_file(SESSION_FILE) {
            Ok(_) => {println!("session saved to: {}", SESSION_FILE)},
            Err(e) => {
                println!("NOTE: failed to save the session[{}],you will sign out when program stops working",e);
            }
        }

    }
    else {
        println!("you are logged in");

    }

    Ok(())
}
fn main() -> Result {
    runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(main_async())
}

fn prompt(message: &str) -> Option<String> {
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