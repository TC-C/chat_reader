use crate::tools::hex_to_rgb;
use crate::{
    tools::{clean_quotes, format_time_string, print_queue},
    twitch_client::TwitchClient,
};
use lazy_static::lazy_static;
use regex::Regex;
use reqwest::blocking::Client;
use serde_json::Value;
use std::sync::mpsc::{channel, Receiver};
use termion::color::{Fg, Red, Reset};
lazy_static! {
    static ref CLIENT: Client = Client::new();
}

#[derive(Clone)]
pub(crate) struct TwitchVOD {
    pub(crate) title: String,
    pub(crate) id: u32,
}

impl TwitchVOD {
    /// Creates a new `TwitchVOD` from a `u32` that represents an ID and an `&str` that represents the title
    ///
    /// The function will not check any values and may result in errors when calling other functions
    pub(crate) fn new_unchecked(id: u32, title: &str) -> TwitchVOD {
        TwitchVOD {
            id,
            title: String::from(title),
        }
    }
    /// Creates a new `TwitchVOD` from a `u32` that represents the ID of the VOD
    ///
    /// A valid ID would be `799499623`, which can be derived from the VOD URL: https://www.twitch.tv/videos/799499623
    pub(crate) fn new(id: u32, client: &TwitchClient) -> TwitchVOD {
        let data: Value = match CLIENT
            .get(format!("https://api.twitch.tv/helix/videos?id={}", id))
            .bearer_auth(&client.access_token)
            .header("Client-ID", &client.id)
            .send()
        {
            Ok(get) => match get.json() {
                Ok(json) => json,
                Err(e) => {
                    panic!("{red}{}{reset}", e, red = Fg(Red), reset = Fg(Reset));
                }
            },
            Err(e) => {
                panic!("{red}{}{reset}", e, red = Fg(Red), reset = Fg(Reset));
            }
        };
        let title = data
            .get("data")
            .unwrap_or_else(|| panic!("\nThe VOD ID {} could not be found", id))
            .get(0)
            .unwrap_or_else(|| panic!("\nVOD Data is an empty array"))
            .get("title")
            .unwrap_or_else(|| panic!("\nCould not find title in data"))
            .to_string();
        TwitchVOD { title, id }
    }
    /// Identical function to `twitch_vod::print_chat()` except that no Receiver<()> is required.
    ///
    /// Comments will be printed as soon as they are parsed and will not remain in a queue
    ///
    /// This is recommended for single thread use cases
    pub(crate) fn print_chat_blocking(&self, filter: &Regex, client: &TwitchClient) {
        let (tx, rx) = channel();
        tx.send(()).unwrap(); //print immediately
        self.print_chat(filter, client, rx)
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
    pub(crate) fn print_chat(&self, filter: &Regex, client: &TwitchClient, rx: Receiver<()>) {
        let mut cursor = String::new();
        let mut comment_queue: Vec<String> = Vec::new();
        let mut waiting_to_print = true;
        loop {
            let comment_json: Value = match CLIENT
                .get(format!(
                    "https://api.twitch.tv/v5/videos/{}/comments?cursor={}",
                    self.id, cursor
                ))
                .header("Client-ID", &client.id)
                .header("Connection", "keep-alive")
                .send()
            {
                Ok(get) => match get.json() {
                    Ok(json) => json,
                    Err(e) => panic!("{red}{}{reset}", e, red = Fg(Red), reset = Fg(Reset)),
                },
                Err(e) => panic!("{red}{}{reset}", e, red = Fg(Red), reset = Fg(Reset)),
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
                    Err(e) => panic!("{red}{}{reset}", e, red = Fg(Red), reset = Fg(Reset)),
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
                let color = match message.get("user_color") {
                    None => String::new(),
                    Some(color) => clean_quotes(&color.to_string()),
                };
                if filter.is_match(&body) {
                    let comment;
                    if color.is_empty() {
                        comment = format!("[{}][{}]: {}", timestamp, display_name, body);
                    } else {
                        let color = match hex_to_rgb(&color) {
                            Ok(color) => color,
                            Err(e) => panic!("{red}{}{reset}", e, red = Fg(Red), reset = Fg(Reset)),
                        };
                        comment = format!(
                            "[{}][{user_color}{}{reset}]: {}",
                            timestamp,
                            display_name,
                            body,
                            user_color = Fg(color),
                            reset = Fg(Reset)
                        );
                    }
                    comment_queue.push(comment)
                }
                if waiting_to_print {
                    if rx.try_recv().is_ok() {
                        waiting_to_print = false
                    }
                } else {
                    print_queue(&mut comment_queue)
                }
            }
            match comment_json.get("_next") {
                Some(_next) => cursor = clean_quotes(&_next.to_string()),
                None => break,
            }
        }
        if !comment_queue.is_empty() {
            rx.recv().unwrap();
            print_queue(&mut comment_queue)
        }
    }

    /// When possible, returns a `String` representation of the M3U8 playlist link for the associated VOD
    ///
    /// Requires video ID to be valid
    ///
    /// In special cases, such as for channel trailers, where M3U8's cannot be easily computed, the official VOD link is returned
    pub(crate) fn m3u8(&self, client: &TwitchClient) -> String {
        let vod_info: Value = match CLIENT
            .get(format!("https://api.twitch.tv/v5/videos/{}", self.id))
            .header("Client-ID", &client.id)
            .send()
        {
            Ok(get) => get
                .json()
                .unwrap_or_else(|_| panic!("\nCould not parse JSON")),
            Err(e) => panic!("{red}{}{reset}", e, red = Fg(Red), reset = Fg(Reset)),
        };
        let preview_url = vod_info
            .get("animated_preview_url")
            .unwrap_or_else(|| {
                panic!("\nCould not find animated_preview_url in vod_info, possibly invalid VOD ID")
            })
            .to_string();
        let chunked_index = preview_url
            .find("storyboards")
            .unwrap_or_else(|| panic!("\n'storyboards' was not found in the URL"));
        let domain_url = preview_url[1..chunked_index].to_owned() + "chunked/";
        let vod_type = clean_quotes(
            &vod_info
                .get("broadcast_type")
                .unwrap_or_else(|| panic!("\n'storyboards' was not found in the URL"))
                .to_string(),
        );
        let vod_type = vod_type.as_str();
        match vod_type {
            "highlight" => format!("{}highlight-{}.m3u8", domain_url, self.id),
            "archive" => format!("{}index-dvr.m3u8", domain_url),
            _ => format!("https://twitch.tv/videos/{}", self.id),
        }
    }
}
