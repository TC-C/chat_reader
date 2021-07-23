use serde_json::Value;
use crate::tools::{clean_quotes, CLIENT};

/// A struct that is meant to be used to assist with API calls made with the Twitch API
///
/// Stores the original Client-ID that was a passed when `TwitchClient::new()` is called as well as an access token that is generated
#[derive(Clone)]
pub(crate) struct TwitchClient {
    pub(crate) id: String,
    pub(crate) access_token: String,
}


impl TwitchClient {
    /// Creates a new `TwitchClient` from 2 `&str`s that represent a Client-ID and Client-Secret, respectively
    ///
    /// A Client-ID and Client-Secret can be generated on the Twitch Developer Console: https://dev.twitch.tv/console/extensions/create
    pub(crate) fn new(id: &str, client_secret: &str) -> TwitchClient {
        let client_access_token: Value = CLIENT.post(format!("https://id.twitch.tv/oauth2/token?grant_type=client_credentials&client_secret={}", client_secret))
            .header("Client-ID", id)
            .send()
            .expect("https://id.twitch.tv refused to connect")
            .json()
            .unwrap();
        let access_token = clean_quotes(&client_access_token.get("access_token")
            .expect("Failed to find property: access_token")
            .to_string());
        TwitchClient {
            id: String::from(id),
            access_token,
        }
    }
}