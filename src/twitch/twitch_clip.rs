use crate::twitch_channel::TwitchChannel;
use lazy_static::lazy_static;
use reqwest::blocking::Client;
use serde_json::{Value, json};
use crate::tools::clean_quotes;
use serde_json::value::Value::Null;

lazy_static! {static ref CLIENT: Client = Client::new();}

pub fn print_clips_from(channel: TwitchChannel, pat: String) {
    let name = channel.name;
    let mut cursor = String::from("");
    loop {
        let mut did_change = false;
        let request = "[{\"operationName\":\"ClipsCards__User\",\"variables\":{\"login\":\"".to_owned() + &name + "\",\"limit\":20,\"criteria\":{\"filter\":\"ALL_TIME\"}" + &cursor + "},\"extensions\":{\"persistedQuery\":{\"version\":1,\"sha256Hash\":\"b73ad2bfaecfd30a9e6c28fada15bd97032c83ec77a0440766a56fe0bd632777\"}}}]";
        let response: Value = CLIENT.post("https://gql.twitch.tv/gql")
            .header("Client-Id", "kimne78kx3ncx6brgo4mv6wki5h1ko")
            .header("Authorization", "OAuth hfcm528b89m5eyturgicl5k6jpx2cb")
            .header("Connection", "keep-alive")
            .body(request)
            .send()
            .unwrap()
            .json()
            .unwrap();
        let clips = response
            .get(0).unwrap()
            .get("data").unwrap()
            .get("user").unwrap()
            .get("clips").expect("Unknown Username!")
            .get("edges");
        if clips.is_none() {
            continue;
        }
        let clips = clips.expect(&response.to_string())
            .as_array().unwrap();

        for clip in clips {
            let temp_cursor = clip.get("cursor").unwrap();
            if !temp_cursor.is_null() {
                cursor = format!(",\"cursor\":{}", &temp_cursor);
                did_change = true
            }
            let title = clip
                .get("node").unwrap()
                .get("title").unwrap().to_string();
            let url = clip
                .get("node").unwrap()
                .get("url").unwrap().to_string();
            if title.contains(&pat) {
                println!("[{}] {}", clean_quotes(&title), clean_quotes(&url))
            }
        }
        if !did_change {
            break;
        }
    }
}