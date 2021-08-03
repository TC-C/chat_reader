use regex::Regex;
use serde_json::Value;
use crate::{
    twitch_channel::TwitchChannel,
    tools::{clean_quotes, CLIENT},
};
use crate::twitch_client::TwitchClient;

pub(crate) fn print_clips_from(channel: &TwitchChannel, filter: &Regex, client: &TwitchClient) {
    let name = &channel.name;
    let client_id = &client.id;
    let token = &client.access_token;
    let mut cursor = String::new();
    loop {
        let mut did_change = false;
        let response = get_clips_json(name, client_id, token, &cursor);
        let clips = match response
            .get(0).expect(&response.to_string())
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
                cursor = format!(r#","cursor":{}"#, &temp_cursor);
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

fn get_clips_json(name: &String, client_id: &String, token: &String, cursor: &String) -> Value {
    let request = (r#"[{"operationName":"ClipsCards__User","variables":{"login":""#).to_owned() + name + r#"","limit":20,"criteria":{"filter":"ALL_TIME"}"# + cursor + r#"},"extensions":{"persistedQuery":{"version":1,"sha256Hash":"b73ad2bfaecfd30a9e6c28fada15bd97032c83ec77a0440766a56fe0bd632777"}}}]"#;
    CLIENT.post("https://gql.twitch.tv/gql")
        .header("Client-Id", client_id)
        .bearer_auth(token)
        .header("Connection", "keep-alive")
        .body(request)
        .send()
        .unwrap()
        .json()
        .unwrap()
}