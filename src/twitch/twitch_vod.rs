use lazy_static::lazy_static;
use reqwest::blocking::Client;
use serde_json::Value;
use crate::twitch_client::TwitchClient;


lazy_static! {static ref CLIENT: Client = Client::new();}

pub struct TwitchVOD {
    pub title: String,
    pub id: u32,
}

fn format_time(seconds: String) -> String {
    let seconds: f64 = seconds.parse().unwrap();
    let seconds = seconds as i16;

    let mut hours = (seconds / (60 * 60)).to_string();
    if hours.len() == 1 {
        hours = format!("0{}", hours);
    }
    let mut minutes = (seconds / 60 % 60).to_string();
    if minutes.len() == 1 {
        minutes = format!("0{}", minutes);
    }
    let mut seconds = (seconds % 60).to_string();
    if seconds.len() == 1 {
        seconds = format!("0{}", seconds);
    }
    return format!("{}:{}:{}", hours, minutes, seconds);
}

fn clean_quotes(string: String) -> String {
    string.trim_start_matches("\"").trim_end_matches("\"").to_string()
}

impl TwitchVOD {
    pub fn new_unchecked(id: u32, title: String) -> TwitchVOD {
        TwitchVOD{
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
        let title = clean_quotes(data
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
                .send()
                .expect("https://api.twitch.tv refused to connect")
                .json()
                .unwrap();
            let comments = comment_json.get("comments").expect(&comment_json.to_string()).as_array().unwrap();
            for comment in comments {
                let timestamp = format_time(comment
                    .get("content_offset_seconds").unwrap().to_string());
                let display_name = clean_quotes(comment
                    .get("commenter").unwrap()
                    .get("display_name").unwrap().to_string());
                let message = clean_quotes(comment
                    .get("message").unwrap()
                    .get("body").unwrap().to_string());
                if message.contains(&pat) {
                    println!("[{}][{}]: {}", timestamp, display_name, message)
                }
            }

            match comment_json.get("_next") {
                Some(_next) => cursor = clean_quotes(_next.to_string()),
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