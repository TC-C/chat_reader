use crate::{
    tools::{clean_quotes, CLIENT},
    twitch_client::TwitchClient,
    twitch_vod::TwitchVOD,
};
use serde_json::Value;
use termion::color::{Fg, Red, Reset};

pub(crate) struct TwitchChannel {
    pub(crate) name: String,
}

impl TwitchChannel {
    /// Creates a new `TwitchChannel` from an `&str` that represents the `name` of a channel
    ///
    /// A valid name would be "nasa", which can be derived from the channel URL: https://www.twitch.tv/nasa
    pub(crate) fn new(name: &str) -> TwitchChannel {
        TwitchChannel {
            name: String::from(name),
        }
    }
    /// Returns the Channel ID of a `TwitchChannel`
    ///```
    /// let nasa_channel = TwitchChannel::new("NASA");
    /// assert!(nasa_channel.id(twitch_client), 151920918)
    /// ```
    fn id(&self, client: &TwitchClient) -> u64 {
        let json_result: Value = match CLIENT
            .get(format!(
                "https://api.twitch.tv/helix/users?login={}",
                self.name
            ))
            .bearer_auth(&client.access_token)
            .header("Client-ID", &client.id)
            .send()
        {
            Ok(get) => match get.json::<Value>() {
                Ok(json) => json,
                Err(e) => panic!("{red}{}{reset}", e, red = Fg(Red), reset = Fg(Reset)),
            },
            Err(e) => panic!("{red}{}{reset}", e, red = Fg(Red), reset = Fg(Reset)),
        };
        match clean_quotes(
            &json_result
                .get("data")
                .unwrap_or_else(|| {
                    panic!(
                        "\nUnable to retrieve channel data! Does '{}' have any symbols?",
                        &self.name
                    )
                })
                .get(0)
                .unwrap_or_else(|| panic!("\nNo channel was found with the name: '{}'", &self.name))
                .get("id")
                .unwrap_or_else(|| panic!("\nCould not find channel id in data"))
                .to_string(),
        )
        .parse::<u64>()
        {
            Ok(id) => id,
            Err(e) => panic!("{red}{}{reset}", e, red = Fg(Red), reset = Fg(Reset)),
        }
    }

    /// Returns an list of `TwitchVOD`'s that are associated with a channel
    ///
    /// The max size of the returned `Vec<TwitchVOD>` will be 100, which is the limit for an API query
    pub(crate) fn vods(&self, client: &TwitchClient) -> Vec<TwitchVOD> {
        let id = self.id(client);
        let client_id = &client.id;
        let access_token = &client.access_token;

        let data: Value = match CLIENT
            .get(format!(
                "https://api.twitch.tv/helix/videos?user_id={}&first=100",
                id
            ))
            .bearer_auth(access_token)
            .header("Client-ID", client_id)
            .send()
        {
            Ok(get) => match get.json::<Value>() {
                Ok(json) => json,
                Err(e) => panic!("{red}{}{reset}", e, red = Fg(Red), reset = Fg(Reset)),
            },
            Err(e) => panic!("{red}{}{reset}", e, red = Fg(Red), reset = Fg(Reset)),
        };
        let vod_data = data
            .get("data")
            .unwrap_or_else(|| panic!("\nUnable to retrieve channel vod data list!"))
            .as_array()
            .unwrap_or_else(|| panic!("\nChannel data could not be parsed as an array"));
        let mut vods: Vec<TwitchVOD> = Vec::with_capacity(vod_data.len());
        for vod in vod_data {
            let vod_id = match clean_quotes(
                &vod.get("id")
                    .unwrap_or_else(|| panic!("\nCould not find vod id in data"))
                    .to_string(),
            )
            .parse::<u32>()
            {
                Ok(id) => id,
                Err(e) => panic!("{red}{}{reset}", e, red = Fg(Red), reset = Fg(Reset)),
            };
            let title = clean_quotes(
                &vod.get("title")
                    .unwrap_or_else(|| panic!("\nCould not find vod title in data"))
                    .to_string(),
            );
            let vod = TwitchVOD::new_unchecked(vod_id, &title);
            vods.push(vod);
        }
        vods
    }
}
