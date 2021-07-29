use lazy_static::lazy_static;
use regex::Regex;
use reqwest::blocking::Client;
use serde_json::Value;
use std::{process::exit, sync::mpsc::{Receiver, channel}};
use termion::color::{Fg, Reset, Rgb};
use crate::{
    twitch_client::TwitchClient,
    tools::{clean_quotes, print_queue, format_time_string},
};
use crate::tools::hex_to_rgb;
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
        let data: Value = match CLIENT.get(format!("https://api.twitch.tv/helix/videos?id={}", id))
            .bearer_auth(&client.access_token)
            .header("Client-ID", &client.id)
            .send() {
            Ok(get) => match get.json() {
                Ok(json) => json,
                Err(_) => {
                    eprintln!("\nCould not parse JSON");
                    exit(-1)
                }
            }
            Err(_) => {
                eprintln!("\nCould not connect to https://api.twitch.tv/");
                exit(-1)
            }
        };
        let title = match data
            .get("data") {
            None => {
                eprintln!("\nThe VOD ID {} could not be found", id);
                std::process::exit(-1)
            }
            Some(data) => match data.get(0) {
                None => {
                    eprintln!("\nVOD Data is an empty array");
                    exit(-1)
                }
                Some(data) => match data.get("title") {
                    None => {
                        eprintln!("\nCould not find comments in data");
                        exit(-1)
                    }
                    Some(title) => title.to_string()
                }
            }
        };
        TwitchVOD {
            title,
            id,
        }
    }
    /// Identical function to `twitch_vod::print_chat()` except that no Receiver<()> is required.
    ///
    /// Comments will be printed as soon as they are parsed and will not remain in a queue
    ///
    /// This is recommended for single thread use cases
    pub(crate) fn print_chat_blocking(&self, filter: &Regex, client: &TwitchClient) {
        let (tx, rx) = channel();
        tx.send(()); //print immediately
        self.print_chat(&filter, &client, rx)
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
            let comment_json: Value = match CLIENT.get(format!("https://api.twitch.tv/v5/videos/{}/comments?cursor={}", self.id, cursor))
                .header("Client-ID", &client.id)
                .header("Connection", "keep-alive")
                .send() {
                Ok(get) => match get.json() {
                    Ok(json) => json,
                    Err(_) => {
                        eprintln!("\nCould not parse JSON");
                        exit(-1)
                    }
                }
                Err(_) => {
                    eprintln!("\nCould not connect to https://api.twitch.tv/");
                    exit(-1)
                }
            };
            let comments = match comment_json.get("comments") {
                None => {
                    eprintln!("\nCould not find comments in data");
                    exit(-1)
                }
                Some(comments) => match comments.as_array() {
                    None => {
                        eprintln!("\nChannel vod data could not be parsed as an array!");
                        exit(-1)
                    }
                    Some(array) => array
                }
            };
            for comment in comments {
                let timestamp = match comment.get("content_offset_seconds") {
                    None => {
                        eprintln!("\nCould not find content_offset_seconds in comment");
                        exit(-1)
                    }
                    Some(timestamp) => format_time_string(&timestamp.to_string())
                };
                let display_name = match comment.get("commenter") {
                    None => {
                        eprintln!("\nCould not find commenter in comment");
                        exit(-1)
                    }
                    Some(commenter) => match commenter.get("display_name") {
                        None => {
                            eprintln!("\nCould not find display_name in commenter");
                            exit(-1)
                        }
                        Some(commenter) => clean_quotes(&commenter.to_string())
                    }
                };
                let message = match comment.get("message") {
                    None => {
                        eprintln!("\nCould not find message in comment");
                        exit(-1)
                    }
                    Some(message) => message
                };
                let body = match message.get("body") {
                    None => {
                        eprintln!("\nCould not find body in message");
                        exit(-1)
                    }
                    Some(body) => clean_quotes(&body.to_string())
                };
                let color = match message.get("user_color") {
                    None => String::new(),
                    Some(color) => clean_quotes(&color.to_string())
                };
                if filter.is_match(&body) {
                    let mut comment = String::new();
                    if color.is_empty() {
                        comment = format!("[{}][{}]: {}", timestamp, display_name, body);
                    } else {
                        comment = format!("[{}][{user_color}{}{reset}]: {}",
                                          timestamp, display_name, body,
                                          user_color = Fg(hex_to_rgb(&color)),
                                          reset = Fg(Reset));
                    }
                    comment_queue.push(comment)
                }
                if waiting_to_print {
                    if rx.try_recv().is_ok() { waiting_to_print = false }
                } else { print_queue(&mut comment_queue) }
            }
            match comment_json.get("_next") {
                Some(_next) => cursor = clean_quotes(&_next.to_string()),
                None => break
            }
        }
        if !comment_queue.is_empty() {
            rx.recv();
            print_queue(&mut comment_queue)
        }
    }

    /// When possible, returns a `String` representation of the M3U8 playlist link for the associated VOD
    ///
    /// Requires video ID to be valid
    ///
    /// In special cases, such as for channel trailers, where M3U8's cannot be easily computed, the official VOD link is returned
    pub(crate) fn m3u8(&self, client: &TwitchClient) -> String {
        let vod_info: Value = match CLIENT.get(format!("https://api.twitch.tv/v5/videos/{}", self.id))
            .header("Client-ID", &client.id)
            .send() {
            Ok(get) => match get.json() {
                Ok(json) => json,
                Err(_) => {
                    eprintln!("\nCould not parse JSON");
                    exit(-1)
                }
            }
            Err(_) => {
                eprintln!("\nCould not connect to https://api.twitch.tv/");
                exit(-1)
            }
        };
        let preview_url = match vod_info.get("animated_preview_url") {
            None => {
                eprintln!("\nCould not find animated_preview_url in vod_info, possibly invalid VOD ID");
                exit(-1)
            }
            Some(animated_preview_url) => animated_preview_url.to_string()
        };
        let chunked_index = match preview_url.find("storyboards") {
            None => {
                eprintln!("\n'storyboards' was not found in the URL");
                exit(-1)
            }
            Some(storyboards) => storyboards
        };
        let domain_url = preview_url[1..chunked_index].to_owned() + "chunked/";
        let vod_type = match vod_info.get("broadcast_type") {
            None => {
                eprintln!("\n'storyboards' was not found in the URL");
                exit(-1)
            }
            Some(broadcast_type) => clean_quotes(&broadcast_type.to_string())
        };
        let vod_type = vod_type.as_str();
        match vod_type {
            "highlight" => format!("{}highlight-{}.m3u8", domain_url, self.id),
            "archive" => format!("{}index-dvr.m3u8", domain_url),
            _ => format!("https://twitch.tv/videos/{}", self.id)
        }
    }
}