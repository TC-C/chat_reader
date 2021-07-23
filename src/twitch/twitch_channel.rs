use serde_json::Value;
use crate::{
    twitch_client::TwitchClient,
    twitch_vod::TwitchVOD,
    tools::{clean_quotes, CLIENT}
};


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
        let json_result: Value = CLIENT.get(format!("https://api.twitch.tv/helix/users?login={}", self.name))
            .bearer_auth(&client.access_token)
            .header("Client-ID", &client.id)
            .send()
            .expect("https://api.twitch.tv refused to connect")
            .json()
            .unwrap();

        match json_result
            .get("data").unwrap()
            .get(0) {
            None => {
                eprintln!("\nThe channel name {} could not be found", self.name);
                std::process::exit(-1);
            }
            Some(result) => clean_quotes(
                &result.get("id").unwrap()
                    .to_string()).parse::<u64>().unwrap()
        }
    }
    /// Returns an list of `TwitchVOD`'s that are associated with a channel
    ///
    /// The max size of the returned`Vec` will be 100, which is the limit for an API query
    pub(crate) fn vods(&self, client: &TwitchClient) -> Vec<TwitchVOD> {
        let id = self.id(client);
        let client_id = &client.id;
        let access_token = &client.access_token;

        let data: Value = CLIENT.get(format!("https://api.twitch.tv/helix/videos?user_id={}&first=100", id))
            .bearer_auth(access_token)
            .header("Client-ID", client_id)
            .send()
            .unwrap()
            .json().unwrap();
        let vod_data = data.get("data").unwrap().as_array().unwrap();
        let mut vods: Vec<TwitchVOD> = Vec::with_capacity(vod_data.len());
        for vod in vod_data {
            let vod_id = clean_quotes(&vod.get("id").unwrap().to_string()).parse::<u32>().unwrap();
            let title = &clean_quotes(&vod.get("title").unwrap().to_string());
            let vod = TwitchVOD::new_unchecked(vod_id, title);
            vods.push(vod);
        }
        vods
    }
}