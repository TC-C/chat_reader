use crate::twitch_client::TwitchClient;
use lazy_static::lazy_static;
use reqwest::blocking::Client;
use serde_json::Value;
use crate::twitch_vod::TwitchVOD;


lazy_static! {static ref CLIENT: Client = Client::new();}

fn clean_quotes(string: String) -> String {
    string.trim_start_matches("\"").trim_end_matches("\"").to_string()
}

pub(crate) struct TwitchChannel {
    name: String,
}

impl TwitchChannel {
    pub fn new(name: String) -> TwitchChannel {
        TwitchChannel {
            name
        }
    }
    fn id(self, client: &TwitchClient) -> u64 {
        let json_result: Value = CLIENT.get(format!("https://api.twitch.tv/helix/users?login={}", self.name))
            .bearer_auth(&client.access_token)
            .header("Client-ID", &client.id)
            .send()
            .expect("https://api.twitch.tv refused to connect")
            .json()
            .unwrap();
        json_result
            .get("data").expect(&format!("The channel name: {}", self.name))
            .get(0).unwrap()
            .get("id").unwrap().as_str().unwrap().parse::<u64>().unwrap()
    }

    pub fn vods(self, client: &TwitchClient) -> Vec<TwitchVOD> {
        let id = self.id(&client);
        let client_id = &client.id;
        let access_token = &client.access_token;

        let data: Value = CLIENT.get(format!("https://api.twitch.tv/helix/videos?user_id={}", id))
            .bearer_auth(access_token)
            .header("Client-ID", client_id)
            .send()
            .expect("https://api.twitch.tv refused to connect")
            .json().unwrap();
        let vod_data = data.get("data").expect("invalid user ID").as_array().unwrap();
        let mut vods: Vec<TwitchVOD> = Vec::with_capacity(vod_data.len());
        for vod in vod_data {
            let vod_id = clean_quotes(vod.get("id").unwrap().to_string()).parse::<u32>().unwrap();
            let title = clean_quotes(vod.get("title").unwrap().to_string());
            let vod = TwitchVOD::new_unchecked(vod_id, title);
            vods.push(vod);
        }
        vods
    }
}