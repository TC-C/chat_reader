use crate::twitch_client::TwitchClient;
use serde_json::Value;
use crate::twitch_vod::TwitchVOD;
use crate::tools::clean_quotes;
use crate::tools::CLIENT;


pub struct TwitchChannel {
    pub name: String,
}

impl TwitchChannel {
    /// Creates a new `TwitchChannel` from a String object that represents the `name` of a channel
    ///
    /// A valid name would be "nasa", which can be derived from the channel URL: https://www.twitch.tv/nasa
    pub fn new(name: String) -> TwitchChannel {
        TwitchChannel {
            name
        }
    }
    /// Returns the Channel ID of a `TwitchChannel`
    ///```
    /// let nasa_channel = TwitchChannel::new(String::from("NASA"));
    /// assert!(nasa_channel.id(twitch_client), 151920918)
    /// ```
    /// 151920918 is the channel ID of NASA
    fn id(&self, client: &TwitchClient) -> u64 {
        let json_result: Value = CLIENT.get(format!("https://api.twitch.tv/helix/users?login={}", self.name))
            .bearer_auth(&client.access_token)
            .header("Client-ID", &client.id)
            .send()
            .expect("https://api.twitch.tv refused to connect")
            .json()
            .unwrap();

        clean_quotes(&json_result
            .get("data").expect(&format!("The channel name: {}", self.name))
            .get(0).unwrap()
            .get("id").unwrap().to_string()).parse::<u64>().unwrap()
    }
    /// Returns an list of `TwitchVOD`'s that are associated with a channel
    ///
    /// The max size of the returned`Vec` will be 100, which is the limit for an API query
    pub fn vods(&self, client: &TwitchClient) -> Vec<TwitchVOD> {
        let id = self.id(client);
        let client_id = &client.id;
        let access_token = &client.access_token;

        let data: Value = CLIENT.get(format!("https://api.twitch.tv/helix/videos?user_id={}&first=100", id))
            .bearer_auth(access_token)
            .header("Client-ID", client_id)
            .send()
            .expect("https://api.twitch.tv refused to connect")
            .json().unwrap();
        let vod_data = data.get("data").expect("invalid user ID").as_array().unwrap();
        let mut vods: Vec<TwitchVOD> = Vec::with_capacity(vod_data.len());
        for vod in vod_data {
            let vod_id = clean_quotes(&vod.get("id").unwrap().to_string()).parse::<u32>().unwrap();
            let title = clean_quotes(&vod.get("title").unwrap().to_string());
            let vod = TwitchVOD::new_unchecked(vod_id, &title);
            vods.push(vod);
        }
        vods
    }
}