use serde_json::Value;
use crate::tools::{clean_quotes, CLIENT};
use reqwest::blocking::Response;
use reqwest::Error;
use std::process::exit;
use std::io::Read;

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
        let client_access_token: Value = match CLIENT.post(format!("https://id.twitch.tv/oauth2/token?grant_type=client_credentials&client_secret={}", client_secret))
            .header("Client-ID", id)
            .send() {
            Ok(post) => match post.json() {
                Ok(json) => json,
                Err(_) => {
                    eprintln!("Could not parse JSON");
                    exit(-1)
                }
            }
            Err(_) => {
                eprintln!("Could not connect to https://id.twitch.tv/");
                exit(-1);
            }
        };
        let access_token = clean_quotes(
            &match client_access_token.get("access_token") {
                None => {
                    eprintln!("Access Token property could not be found in JSON: {}", client_access_token);
                    exit(-1)
                }
                Some(access_token) => access_token.to_string()
            });


        TwitchClient {
            id: String::from(id),
            access_token,
        }
    }
}