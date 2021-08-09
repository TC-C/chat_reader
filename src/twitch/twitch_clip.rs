use crate::tools::error;
use crate::{
    tools::{clean_quotes, CLIENT, CLIENT_ID},
    twitch_channel::TwitchChannel,
};
use regex::Regex;
use reqwest::Error;
use serde_json::Value;

pub(crate) fn print_clips_from(channel: &TwitchChannel, filter: &Regex) {
    let name = &channel.name;
    let mut cursor = String::new();
    loop {
        let mut did_change = false;
        let response = match get_clips_json(name, &cursor) {
            Ok(response) => response,
            Err(e) => {
                error(&e.to_string());
                return;
            }
        };
        let clips = match response
            .get(0)
            .unwrap_or_else(|| panic!("{}", response.to_string()))
            .get("data")
            .unwrap()
            .get("user")
            .unwrap()
            .get("clips")
            .expect("Unknown Username!")
            .get("edges")
        {
            None => continue,
            Some(clips) => clips.as_array().unwrap(),
        };

        for clip in clips {
            let temp_cursor = clip.get("cursor").unwrap();
            if !temp_cursor.is_null() {
                cursor = format!(r#","cursor":{}"#, &temp_cursor);
                did_change = true
            }
            let title = clip.get("node").unwrap().get("title").unwrap().to_string();
            let url = clip.get("node").unwrap().get("url").unwrap().to_string();
            if filter.is_match(&title) {
                println!("[{}] {}", clean_quotes(&title), clean_quotes(&url))
            }
        }
        if !did_change {
            break;
        }
    }
}

fn get_clips_json(name: &str, cursor: &str) -> Result<Value, Error> {
    let request = r#"[
   {
      "operationName":"ClipsCards__User",
      "variables":{
         "login":""#
        .to_owned()
        + name
        + r#"",
         "limit":100,
         "criteria":{
            "filter":"ALL_TIME"
         }"#
        + cursor
        + r#"
      },
      "extensions":{
         "persistedQuery":{
            "version":1,
            "sha256Hash":"b73ad2bfaecfd30a9e6c28fada15bd97032c83ec77a0440766a56fe0bd632777"
         }
      }
   }
]"#;
    match CLIENT
        .post("https://gql.twitch.tv/gql")
        .header("Client-Id", CLIENT_ID)
        .header("Connection", "keep-alive")
        .body(request)
        .send()
    {
        Ok(result) => Ok(result.json().unwrap()),
        Err(e) => Err(e),
    }
}
