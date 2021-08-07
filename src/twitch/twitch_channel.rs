use serde_json::Value;

use crate::tools::exit_error;
use crate::{
    tools::{clean_quotes, CLIENT, CLIENT_ID},
    twitch_vod::TwitchVOD,
};
use serde_json::value::Value::Null;

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

    /// Returns an list of `TwitchVOD`'s that are associated with a channel
    ///
    /// The max size of the returned `Vec<TwitchVOD>` will be 100, which is the limit for a single API query

    pub(crate) fn vods(&self) -> Vec<TwitchVOD> {
        let request = r#"[
   {
      "operationName":"FilterableVideoTower_Videos",
      "variables":{
         "limit":100,
         "channelOwnerLogin":""#
            .to_owned()
            + &self.name
            + r#"",
         "broadcastType":null,
         "videoSort":"TIME",
         "cursor":""
      },
      "extensions":{
         "persistedQuery":{
            "version":1,
            "sha256Hash":"a937f1d22e269e39a03b509f65a7490f9fc247d7f83d6ac1421523e3b68042cb"
         }
      }
   }
]"#;
        let data: Value = CLIENT
            .post("https://gql.twitch.tv/gql")
            .header("Client-Id", CLIENT_ID)
            .body(request)
            .send()
            .unwrap()
            .json()
            .unwrap();
        let user = data
            .get(0)
            .unwrap()
            .get("data")
            .unwrap()
            .get("user")
            .unwrap();
        if user == &Null {
            exit_error("No user was found\nExiting...")
        }
        let vod_data = user
            .get("videos")
            .unwrap()
            .get("edges")
            .unwrap()
            .as_array()
            .unwrap();
        let mut vods = Vec::with_capacity(vod_data.len());
        for vod in vod_data {
            let vod = vod.get("node").unwrap();
            let id = clean_quotes(&vod.get("id").unwrap().to_string())
                .parse::<u32>()
                .unwrap();
            let title = clean_quotes(&vod.get("title").unwrap().to_string());
            let animated_preview_url =
                clean_quotes(&vod.get("animatedPreviewURL").unwrap().to_string());
            let v = TwitchVOD::new_unchecked(id, title, animated_preview_url);
            vods.push(v);
        }
        vods
    }
}
