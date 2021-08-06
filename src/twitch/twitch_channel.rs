use crate::tools::CLIENT_ID;
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
    fn id(&self) -> u64 {
        let request = r#"[{"operationName":"ChannelRoot_AboutPanel","variables":{"channelLogin":""#
            .to_owned()
            + &self.name
            + r#""},"extensions":{"persistedQuery":{"version":1,"sha256Hash":"6d25b3692d788e7a251aa1febf74f5cafb1a917142abd743fe1f65329404e07f"}}}]"#;
        let json_result: Value = CLIENT
            .post("https://gql.twitch.tv/gql")
            .header("Client-Id", CLIENT_ID)
            .body(request)
            .send()
            .unwrap()
            .json()
            .unwrap();
        let id = clean_quotes(
            &json_result
                .get(0)
                .unwrap()
                .get("data")
                .unwrap()
                .get("user")
                .unwrap()
                .get("channel")
                .unwrap_or_else(|| panic!("This channel does not exist"))
                .get("id")
                .unwrap()
                .to_string(),
        );
        id.parse::<u64>().unwrap()
    }

    /// Returns an list of `TwitchVOD`'s that are associated with a channel
    ///
    /// The max size of the returned `Vec<TwitchVOD>` will be 100, which is the limit for an API query
    pub(crate) fn vods(&self, client: &TwitchClient) -> Vec<TwitchVOD> {
        let id = self.id();
        let client_id = CLIENT_ID;
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
            .unwrap_or_else(|| {
                dbg!(&data);
                panic!("\nUnable to retrieve channel vod data list!");
            })
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
