use crate::tools::{clean_quotes, exit_error, format_time_string, hex_to_rgb, CLIENT_ID};
use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
};
use lazy_static::lazy_static;
use regex::Regex;
use reqwest::blocking::Client;
use serde_json::Value;
use std::{
    io::stdout,
    sync::mpsc::{channel, Receiver},
};
lazy_static! {
    static ref CLIENT: Client = Client::new();
}

#[derive(Clone)]
pub(crate) struct TwitchVOD {
    pub(crate) title: String,
    pub(crate) id: u32,
    animated_preview_url: String,
}

fn dump_comments(comments: &mut Vec<(String, String, String, Color)>) {
    for comment in comments.iter_mut() {
        let timestamp = &comment.0;
        let display_name = &comment.1;
        let message = &comment.2;
        let user_color = comment.3;
        execute!(
            stdout(),
            Print(format!("[{}][", timestamp)),
            SetForegroundColor(user_color),
            Print(String::from(display_name)),
            ResetColor,
            Print(format!("]: {}\n", message))
        );
    }
    comments.clear()
}

impl TwitchVOD {
    /// Creates a new `TwitchVOD` from a `u32` that represents an ID and an `&str` that represents the title
    ///
    /// The function will not check any values and may result in errors when calling other functions
    pub(crate) fn new_unchecked(id: u32, title: String, animated_preview_url: String) -> Self {
        TwitchVOD {
            id,
            title,
            animated_preview_url,
        }
    }
    /// Creates a new `TwitchVOD` from a `u32` that represents the ID of the VOD
    ///
    /// A valid ID would be `799499623`, which can be derived from the VOD URL: https://www.twitch.tv/videos/799499623
    pub(crate) fn new(id: u32) -> Self {
        let request = r#"[{
      "operationName":"ComscoreStreamingQuery",
      "variables":{
         "channel":"",
         "clipSlug":"",
         "isClip":false,
         "isLive":false,
         "isVodOrCollection":true,
         "vodID":""#
            .to_owned()
            + &id.to_string()
            + r#""
      },
      "extensions":{
         "persistedQuery":{
            "version":1,
            "sha256Hash":"e1edae8122517d013405f237ffcc124515dc6ded82480a88daef69c83b53ac01"
         }
      }
   }]"#;
        let data: Value = CLIENT
            .post("https://gql.twitch.tv/gql")
            .header("Client-Id", CLIENT_ID)
            .body(request)
            .send()
            .unwrap()
            .json()
            .unwrap();
        //dbg!(&data);
        let title = clean_quotes(
            &data
                .get(0)
                .unwrap()
                .get("data")
                .unwrap()
                .get("video")
                .unwrap()
                .get("title")
                .unwrap()
                .to_string(),
        );
        TwitchVOD {
            title,
            id,
            animated_preview_url: String::new(),
        }
    }
    /// Identical function to `twitch_vod::print_chat()` except that no Receiver<()> is required.
    ///
    /// Comments will be printed as soon as they are parsed and will not remain in a queue
    ///
    /// This is recommended for single thread use cases
    pub(crate) fn print_chat_blocking(&self, filter: &Regex) {
        let (tx, rx) = channel();
        tx.send(()).unwrap(); //print immediately
        self.print_chat(filter, rx)
    }

    /// Prints the chat to console from an individual `TwitchVOD`
    ///
    /// This required parameters are a `TwitchVOD` with a valid name, `Regex` filter, and `Receiver<bool>`
    ///
    /// All `comments: String` will be ran through the passed `Regex` and only comments that return a match to the filter will be displayed
    ///
    /// The `rx: Receiver<bool>` is used to determine when the comments should be printed out
    ///
    /// By default, the outputs are queued into `comment_queue` and then will be allowed to print only when `rx` receives a boolean from a `Sender<bool>`
    pub(crate) fn print_chat(&self, filter: &Regex, rx: Receiver<()>) {
        let mut cursor = String::new();
        let mut comment_queue: Vec<(String, String, String, Color)> = Vec::new();
        let mut waiting_to_print = true;
        loop {
            let comment_json: Value = match CLIENT
                .get(format!(
                    "https://api.twitch.tv/v5/videos/{}/comments?cursor={}",
                    self.id, cursor
                ))
                .header("Client-ID", CLIENT_ID)
                .header("Connection", "keep-alive")
                .send()
            {
                Ok(get) => match get.json() {
                    Ok(json) => json,
                    Err(e) => exit_error(&e.to_string()),
                },
                Err(e) => exit_error(&e.to_string()),
            };
            let comments = comment_json
                .get("comments")
                .unwrap_or_else(|| panic!("\nCould not find comments in data"))
                .as_array()
                .unwrap_or_else(|| panic!("\nChannel vod data could not be parsed as an array"));
            for comment in comments {
                let timestamp = match format_time_string(
                    &comment
                        .get("content_offset_seconds")
                        .unwrap_or_else(|| {
                            panic!("\nCould not find content_offset_seconds in comment")
                        })
                        .to_string(),
                ) {
                    Ok(timestamp) => timestamp,
                    Err(e) => exit_error(&e.to_string()),
                };
                let display_name = clean_quotes(
                    &comment
                        .get("commenter")
                        .unwrap_or_else(|| panic!("\nCould not find commenter in comment"))
                        .get("display_name")
                        .unwrap_or_else(|| panic!("\nCould not find display_name in commenter"))
                        .to_string(),
                );

                let message = comment
                    .get("message")
                    .unwrap_or_else(|| panic!("\nCould not find message in comment"));
                let body = clean_quotes(
                    &message
                        .get("body")
                        .unwrap_or_else(|| panic!("\nCould not find body in message"))
                        .to_string(),
                );
                if filter.is_match(&body) {
                    let color = match message.get("user_color") {
                        None => Color::Reset,
                        Some(color) => {
                            let c_string = clean_quotes(&color.to_string());
                            if c_string.is_empty() {
                                Color::Reset
                            } else {
                                match hex_to_rgb(&c_string) {
                                    Ok(color) => color,
                                    Err(e) => exit_error(&e.to_string()),
                                }
                            }
                        }
                    };
                    comment_queue.push((timestamp, display_name, body, color))
                }
                if waiting_to_print {
                    if rx.try_recv().is_ok() {
                        waiting_to_print = false
                    }
                } else {
                    dump_comments(&mut comment_queue)
                }
            }
            match comment_json.get("_next") {
                Some(_next) => cursor = clean_quotes(&_next.to_string()),
                None => break,
            }
        }
        if !comment_queue.is_empty() {
            rx.recv().unwrap();
            dump_comments(&mut comment_queue)
        }
    }

    /// When possible, returns a `String` representation of the M3U8 playlist link for the associated VOD
    ///
    /// Requires video ID to be valid
    ///
    /// In special cases, such as for channel trailers, where M3U8's cannot be easily computed, the official VOD link is returned
    pub(crate) fn m3u8(&self) -> String {
        let mut preview_url = (&self.animated_preview_url).to_owned();
        if preview_url.is_empty() {
            let request = r#"[{
      "operationName":"VideoPlayer_VODSeekbarPreviewVideo",
      "variables":{
         "includePrivate":false,
         "videoID":""#
                .to_owned()
                + &self.id.to_string()
                + r#""
      },
      "extensions":{
         "persistedQuery":{
            "version":1,
            "sha256Hash":"07e99e4d56c5a7c67117a154777b0baf85a5ffefa393b213f4bc712ccaf85dd6"
         }
      }
   }]"#;
            let data: Value = CLIENT
                .post("https://gql.twitch.tv/gql")
                .header("Client-Id", CLIENT_ID)
                .body(request)
                .send()
                .unwrap()
                .json()
                .unwrap();
            preview_url = clean_quotes(
                &data
                    .get(0)
                    .unwrap()
                    .get("data")
                    .unwrap()
                    .get("video")
                    .unwrap()
                    .get("seekPreviewsURL")
                    .unwrap()
                    .to_string(),
            );
        }
        let chunked_index = preview_url
            .find("storyboards")
            .unwrap_or_else(|| panic!("\n'storyboards' was not found in the URL"));
        let domain_url = preview_url[..chunked_index].to_owned() + "chunked/";
        let request = r#"[
   {
      "operationName":"VideoMetadata",
      "variables":{
         "channelLogin":"",
         "videoID":""#
            .to_owned()
            + &self.id.to_string()
            + r#""
      },
      "extensions":{
         "persistedQuery":{
            "version":1,
            "sha256Hash":"226edb3e692509f727fd56821f5653c05740242c82b0388883e0c0e75dcbf687"
         }
      }
   }
]"#;
        let metadata: Value = CLIENT
            .post("https://gql.twitch.tv/gql")
            .header("Client-Id", CLIENT_ID)
            .body(request)
            .send()
            .unwrap()
            .json()
            .unwrap();
        let vod_type = clean_quotes(
            &metadata
                .get(0)
                .unwrap()
                .get("data")
                .unwrap()
                .get("video")
                .unwrap()
                .get("broadcastType")
                .unwrap()
                .to_string(),
        );
        let vod_type = vod_type.as_str();
        match vod_type {
            "HIGHLIGHT" => format!("{}highlight-{}.m3u8", domain_url, self.id),
            "ARCHIVE" => format!("{}index-dvr.m3u8", domain_url),
            _ => format!("https://twitch.tv/videos/{}", self.id),
        }
    }
}
