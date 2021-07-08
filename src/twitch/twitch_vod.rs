use lazy_static::lazy_static;
use reqwest::blocking::Client;
use serde_json::Value;
use crate::twitch_client::TwitchClient;
use crate::tools::clean_quotes;
use crate::tools::format_time;


lazy_static! {static ref CLIENT: Client = Client::new();}

pub struct TwitchVOD {
    pub title: String,
    pub id: u32,
}

impl TwitchVOD {
    pub fn new_unchecked(id: u32, title: String) -> TwitchVOD {
        TwitchVOD {
            id,
            title,
        }
    }
    pub fn new(id: u32, client: &TwitchClient) -> TwitchVOD {
        let data: Value = CLIENT.get(format!("https://api.twitch.tv/helix/videos?id={}", id))
            .bearer_auth(&client.access_token)
            .header("Client-ID", &client.id)
            .send()
            .expect("https://api.twitch.tv refused to connect")
            .json().unwrap();
        let title = clean_quotes(&data
            .get("data").expect("Invalid VOD ID ")
            .get(0).unwrap()
            .get("title").unwrap().to_string());
        TwitchVOD {
            title,
            id,
        }
    }
    pub fn print_chat(self, pat: String, client: &TwitchClient) {
        let mut cursor = String::from("");
        loop {
            let comment_json: Value = CLIENT.get(format!("https://api.twitch.tv/v5/videos/{}/comments?cursor={}", self.id, cursor))
                .header("Client-ID", &client.id)
                .header("Connection", "keep-alive")
                .send()
                .expect("https://api.twitch.tv refused to connect")
                .json()
                .unwrap();
            let comments = comment_json.get("comments").expect(&comment_json.to_string()).as_array().unwrap();
            for comment in comments {
                let timestamp = format_time(comment
                    .get("content_offset_seconds").unwrap().to_string());
                let display_name = clean_quotes(&comment
                    .get("commenter").unwrap()
                    .get("display_name").unwrap().to_string());
                let message = clean_quotes(&comment
                    .get("message").unwrap()
                    .get("body").unwrap().to_string());
                if message.contains(&pat) {
                    println!("[{}][{}]: {}", timestamp, display_name, message)
                }
            }

            match comment_json.get("_next") {
                Some(_next) => cursor = clean_quotes(&_next.to_string()),
                None => break
            }
        }
    }
    pub fn m3u8(self, client: &TwitchClient) {
        let vod_info: Value = CLIENT.get(format!("https://api.twitch.tv/helix/videos?id={}", self.id))
            .bearer_auth(&client.access_token)
            .header("Client-ID", &client.id)
            .send()
            .expect("https://api.twitch.tv/ refused to connect")
            .json()
            .unwrap();
        println!("{}", vod_info);
    }
}