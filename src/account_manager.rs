use crate::config::AppConfig;
use crate::utils::prompt;
use grammers_client::types::User;
use grammers_client::{Client, SignInError};

pub const SESSION_FILE: &str = "scrapper.session";

pub fn save_session(client_handler: &Client) {
    match client_handler.session().save_to_file(SESSION_FILE) {
        Ok(_) => {
            println!("session saved to: {}", SESSION_FILE)
        }
        Err(e) => {
            println!(
                "NOTE: failed to save the session[{}],you will sign out when program stops working",
                e
            );
        }
    }
}

pub async fn check_status(client_handler: &Client, signed_in: Result<User, SignInError>) {
    match signed_in {
        Err(SignInError::PasswordRequired(password_token)) => {
            let hint = password_token.hint().unwrap_or("None");
            let prompt_message = format!("Enter the password (hint {}): ", &hint);
            let password = prompt(prompt_message.as_str()).expect("failed to get the password");
            client_handler
                .check_password(password_token, password.trim())
                .await
                .unwrap();
        }
        Ok(user) => {
            println!(
                "logged in with user:{},id:{}",
                user.username().unwrap(),
                user.id()
            );
        }
        Err(e) => panic!("{}", e),
    };
}

pub async fn sign_in_async(
    config: &AppConfig,
    client_handler: &Client,
) -> Result<User, SignInError> {
    let login_token = client_handler
        .request_login_code(&config.phone, config.api_id, &config.api_hash)
        .await
        .expect("failed to send code");
    let code = prompt("Enter Code:").expect("failed to get the code");
    return client_handler.sign_in(&login_token, &code).await;
}
