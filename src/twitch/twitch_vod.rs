use lazy_static::lazy_static;
use reqwest::blocking::Client;
use serde_json::Value;
use crate::twitch_client::TwitchClient;
use crate::tools::clean_quotes;
use crate::tools::format_time_string;
use regex::Regex;
use std::collections::VecDeque;
use std::sync::mpsc::{Receiver, channel};


lazy_static! {static ref CLIENT: Client = Client::new();}

#[derive(Clone)]
pub struct TwitchVOD {
    pub title: String,
    pub id: u32,
}

impl TwitchVOD {
    pub fn new_unchecked(id: u32, title: &str) -> TwitchVOD {
        TwitchVOD {
            id,
            title: title.to_string(),
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
    /// Identical function to `twitch_vod::print_chat()` except that no Receiver<bool> is required.
    ///
    /// Comments will be printed as soon as they are parsed and will not remain in a queue
    ///
    /// This is recommended for single thread use cases
    pub fn print_chat_blocking(&self, filter: &Regex, client: &TwitchClient) {
        let (tx, rx) = channel();
        tx.send(true); //print immediately
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
    pub fn print_chat(&self, filter: &Regex, client: &TwitchClient, rx: Receiver<bool>) {
        let mut cursor = String::new();
        let mut comment_queue: VecDeque<String> = VecDeque::new();
        let mut waiting_to_print = true;
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
                let timestamp = format_time_string(&comment
                    .get("content_offset_seconds").unwrap().to_string());
                let display_name = clean_quotes(&comment
                    .get("commenter").unwrap()
                    .get("display_name").unwrap().to_string());
                let message = clean_quotes(&comment
                    .get("message").unwrap()
                    .get("body").unwrap().to_string());
                if filter.is_match(&message) {
                    let comment = format!("[{}][{}]: {}", timestamp, display_name, message);
                    comment_queue.push_back(comment)
                }
                if waiting_to_print {
                    match rx.try_recv() {
                        Ok(_) => {
                            waiting_to_print = false;
                        }
                        Err(_) => {}
                    }
                } else {
                    TwitchVOD::print_queue(&mut comment_queue)
                }
            }
            match comment_json.get("_next") {
                Some(_next) => cursor = clean_quotes(&_next.to_string()),
                None => break
            }
        }
        if !comment_queue.is_empty() {
            rx.recv();
            TwitchVOD::print_queue(&mut comment_queue)
        }
    }
    /// Private function to call println! on all `String`s in a VecDeque whilst emptying it
    fn print_queue(comment_queue: &mut VecDeque<String>) {
        loop {
            match comment_queue.pop_front() {
                None => { break; }
                Some(comment) => println!("{}", comment)
            }
        }
    }
    /// When possible, returns a `String` representation of the M3U8 playlist link for the associated VOD
    ///
    /// Requires video ID to be valid
    ///
    /// In special cases, such as for channel trailers, where M3U8's cannot be easily computed, the official VOD link is returned
    pub fn m3u8(&self, client: &TwitchClient) -> String {
        let vod_info: Value = CLIENT.get(format!("https://api.twitch.tv/v5/videos/{}", self.id))
            .header("Client-ID", &client.id)
            .send()
            .expect("https://api.twitch.tv refused to connect")
            .json()
            .unwrap();
        let preview_url = clean_quotes(&vod_info
            .get("animated_preview_url")
            .expect("Invalid VOD ID")
            .to_string());
        let chunked_index = preview_url.find("storyboards").unwrap();
        let domain_url = preview_url[0..chunked_index].to_string() + "chunked/";
        let vod_type = clean_quotes(&vod_info.get("broadcast_type").unwrap().to_string());
        let vod_type = vod_type.as_str();

        match vod_type {
            "highlight" => format!("{}highlight-{}.m3u8", domain_url, self.id),
            "archive" => format!("{}index-dvr.m3u8", domain_url),
            _ => format!("https://twitch.tv/videos/{}", self.id)
        }
    }
}