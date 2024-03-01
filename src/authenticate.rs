use eyre::{eyre, Result};
use google_calendar::Client;
use url::Url;
use serde::{Serialize, Deserialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct Creds {
    pub client_id: String,
    pub client_secret: String,
    pub token: String,
    pub refresh_token: String,
}

impl Creds {
    pub async fn authenticate(client_id: &str, client_secret: &str) -> Result<Self> {
        let mut calendar_client =
            Client::new(client_id, client_secret, "http://localhost:8080", "", "");

        let user_consent_url = calendar_client
            .user_consent_url(&["https://www.googleapis.com/auth/calendar.readonly".to_string()]);

        println!(
            "Please visit the following URL to grant access to your Google Calendar:\n  {}",
            user_consent_url
        );

        loop {
            // Read input from user
            let mut redirect_url = String::new();

            while let Err(_) = std::io::stdin().read_line(&mut redirect_url) {
                println!("Please enter a valid URL");
            }

            let parsed = Self::parse_redirect_url(&redirect_url);

            if let Err(e) = parsed {
                println!("Error: {}", e);
                continue;
            }

            let (code, state) = parsed.unwrap();
            println!("Code: {}, state: {}", code, state);

            let token = calendar_client
                .get_access_token(&code, &state)
                .await
                .expect("Error getting access token");

            if token.access_token.is_empty() || token.refresh_token.is_empty() {
                return Err(eyre!(
                    "Failed to get access token and refresh token. Please try again."
                ));
            }

            return Ok(Self {
                client_id: client_id.to_string(),
                client_secret: client_secret.to_string(),
                token: token.access_token,
                refresh_token: token.refresh_token,
            });
        }
    }

    fn parse_redirect_url(redirect_url: &str) -> Result<(String, String)> {
        let redirect_url_parsed = Url::parse(&redirect_url)?;

        let mut code = None;
        let mut state = None;

        for (key, value) in redirect_url_parsed.query_pairs() {
            match key.as_ref() {
                "code" => {
                    code = Some(value.to_string());
                }
                "state" => {
                    state = Some(value.to_string());
                }
                _ => {}
            }
        }

        if code.is_none() || state.is_none() {
            return Err(eyre!("Invalid redirect URL"));
        }

        Ok((code.unwrap(), state.unwrap()))
    }
}
