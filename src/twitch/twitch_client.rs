use serde_json::Value;
use crate::tools::CLIENT;
use crate::tools::clean_quotes;

#[derive(Clone)]
pub struct TwitchClient {
    pub id: String,
    pub access_token: String,
}


impl TwitchClient {
    pub fn new(id: String, client_secret: String) -> TwitchClient {
        let client_access_token: Value = CLIENT.post(format!("https://id.twitch.tv/oauth2/token?grant_type=client_credentials&client_secret={}", client_secret))
            .header("Client-ID", &id)
            .send()
            .expect("https://id.twitch.tv refused to connect")
            .json()
            .unwrap();
        let access_token = clean_quotes(&client_access_token.get("access_token")
            .expect("Failed to find property: access_token")
            .to_string());
        TwitchClient {
            id,
            access_token,
        }
    }
}