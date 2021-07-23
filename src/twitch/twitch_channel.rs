use serde_json::Value;
use crate::{
    twitch_client::TwitchClient,
    twitch_vod::TwitchVOD,
    tools::{clean_quotes, CLIENT},
};
use std::process::exit;
use reqwest::blocking::Response;
use reqwest::Error;
use std::num::ParseIntError;


pub(crate) struct TwitchChannel {
    pub(crate) name: String,
}

impl TwitchChannel {
    /// Creates a new `TwitchChannel` from an `&str` that represents the `name` of a channel
    ///
    /// A valid name would be "nasa", which can be derived from the channel URL: https://www.twitch.tv/nasa
    pub(crate) fn new(name: &str) -> TwitchChannel {
        TwitchChannel {
            name: String::from(name)
        }
    }
    /// Returns the Channel ID of a `TwitchChannel`
    ///```
    /// let nasa_channel = TwitchChannel::new("NASA");
    /// assert!(nasa_channel.id(twitch_client), 151920918)
    /// ```
    fn id(&self, client: &TwitchClient) -> u64 {
        let json_result: Value = match CLIENT.get(format!("https://api.twitch.tv/helix/users?login={}", self.name))
            .bearer_auth(&client.access_token)
            .header("Client-ID", &client.id)
            .send() {
            Ok(get) => {
                match get.json::<Value>() {
                    Ok(json) => json,
                    Err(_) => {
                        eprintln!("\nCould not parse JSON");
                        exit(-1)
                    }
                }
            }
            Err(_) => {
                eprintln!("\nCould not connect to https://api.twitch.tv/");
                exit(-1)
            }
        };
        match json_result.get("data") {
            None => {
                eprintln!("\nUnable to retrieve channel data! Does '{}' have any symbols?", &self.name);
                exit(-1)
            }
            Some(data) => {
                match data.get(0) {
                    None => {
                        eprintln!("\nNo channel was found with the name: '{}'", &self.name);
                        exit(-1)
                    }
                    Some(data) => match data.get("id") {
                        None => {
                            eprintln!("\nCould not find channel id in data");
                            exit(-1)
                        }
                        Some(id) => match clean_quotes(&id.to_string()).parse::<u64>() {
                            Ok(id) => id,
                            Err(_) => {
                                eprintln!("\nCould not parse channel id as u64");
                                exit(-1)
                            }
                        }
                    }
                }
            }
        }
    }

    /// Returns an list of `TwitchVOD`'s that are associated with a channel
    ///
    /// The max size of the returned `Vec<TwitchVOD>` will be 100, which is the limit for an API query
    pub(crate) fn vods(&self, client: &TwitchClient) -> Vec<TwitchVOD> {
        let id = self.id(client);
        let client_id = &client.id;
        let access_token = &client.access_token;

        let data: Value = match CLIENT.get(format!("https://api.twitch.tv/helix/videos?user_id={}&first=100", id))
            .bearer_auth(access_token)
            .header("Client-ID", client_id)
            .send() {
            Ok(get) => {
                match get.json::<Value>() {
                    Ok(json) => json,
                    Err(_) => {
                        eprintln!("\nCould not parse JSON");
                        exit(-1)
                    }
                }
            }
            Err(_) => {
                eprintln!("\nCould not connect to https://api.twitch.tv/");
                exit(-1)
            }
        };
        let vod_data = match data.get("data") {
            None => {
                eprintln!("\nUnable to retrieve channel vod data list!");
                exit(-1)
            }
            Some(data) => match data.as_array() {
                None => {
                    eprintln!("\nChannel vod data could not be parsed as an array!");
                    exit(-1)
                }
                Some(vec) => vec
            }
        };
        let mut vods: Vec<TwitchVOD> = Vec::with_capacity(vod_data.len());
        for vod in vod_data {
            let vod_id = match vod.get("id") {
                None => {
                    eprintln!("\nCould not find vod id in data");
                    exit(-1)
                }
                Some(id) => {
                    let id = clean_quotes(&id.to_string());
                    match id.parse::<u32>() {
                        Ok(id) => id,
                        Err(_) => {
                            eprintln!("\nCould not parse '{}'", id);
                            exit(-1)
                        }
                    }
                }
            };
            let title = &match vod.get("title") {
                None => {
                    eprintln!("\nCould not find vod title in data");
                    exit(-1)
                }
                Some(title) => clean_quotes(&title.to_string())
            };
            let vod = TwitchVOD::new_unchecked(vod_id, title);
            vods.push(vod);
        }
        vods
    }
}