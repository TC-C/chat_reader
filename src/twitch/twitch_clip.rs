use regex::Regex;
use serde_json::Value;
use crate::{twitch_channel::TwitchChannel, tools::{clean_quotes, CLIENT}};

pub(crate) fn print_clips_from(channel: &TwitchChannel, filter: &Regex) {
    let name = &channel.name;
    let mut cursor = String::new();
    loop {
        let mut did_change = false;
        let request = "[{\"operationName\":\"ClipsCards__User\",\"variables\":{\"login\":\"".to_owned() + name + "\",\"limit\":20,\"criteria\":{\"filter\":\"ALL_TIME\"}" + &cursor + "},\"extensions\":{\"persistedQuery\":{\"version\":1,\"sha256Hash\":\"b73ad2bfaecfd30a9e6c28fada15bd97032c83ec77a0440766a56fe0bd632777\"}}}]";
        let response: Value = CLIENT.post("https://gql.twitch.tv/gql")
            .header("Client-Id", "kimne78kx3ncx6brgo4mv6wki5h1ko")
            .header("Authorization", "OAuth hfcm528b89m5eyturgicl5k6jpx2cb")
            .header("Connection", "keep-alive")
            .body(request)
            .send()
            .unwrap()
            .json()
            .unwrap();
        let clips = match response
            .get(0).unwrap()
            .get("data").unwrap()
            .get("user").unwrap()
            .get("clips").expect("Unknown Username!")
            .get("edges") {
            None => { continue; }
            Some(clips) => clips.as_array().unwrap()
        };

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
            if filter.is_match(&title) {
                println!("[{}] {}", clean_quotes(&title), clean_quotes(&url))
            }
        }
        if !did_change {
            break;
        }
    }
}