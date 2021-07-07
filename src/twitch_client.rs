use reqwest::blocking::Client;
use lazy_static::lazy_static;
use serde_json::Value;
//client id kimne78kx3ncx6brgo4mv6wki5h1ko
//OAuth hfcm528b89m5eyturgicl5k6jpx2cb

lazy_static! {static ref CLIENT: Client = Client::new();}

pub struct TwitchClient {
    pub id: String,
    pub access_token: String,
}

fn clean_quotes(string: String) -> String {
    string.trim_start_matches("\"").trim_end_matches("\"").to_string()
}

impl TwitchClient {
    pub fn new(id: String, client_secret: String) -> TwitchClient {
        let client_access_token: Value = CLIENT.post(format!("https://id.twitch.tv/oauth2/token?grant_type=client_credentials&client_secret={}", client_secret))
            .header("Client-ID", &id)
            .send()
            .expect("https://id.twitch.tv refused to connect")
            .json()
            .unwrap();
        let access_token = clean_quotes(client_access_token.get("access_token")
            .expect("Failed to find property access_token")
            .to_string());
        TwitchClient {
            id,
            access_token,
        }
    }
}