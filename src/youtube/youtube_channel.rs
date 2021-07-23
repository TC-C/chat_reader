use serde_json::Value;
use crate::tools::{clean_quotes, CLIENT};

const KEY: &str = "AIzaSyCOUG9NmlrerQC5OnS3Erbh5K34PobYDQE";

pub struct YouTubeChannel {
    id: String,
}

impl YouTubeChannel {
    pub fn new(query: String) -> YouTubeChannel {
        let json: Value = CLIENT.get(format!("https://www.googleapis.com/youtube/v3/search?\
        part=snippet&\
        q={}&\
        order=relevance&\
        maxResults=1&\
        pageToken=&\
        key={}", query, KEY))
            .header("Referer", "https://ytcomment.kmcat.uk/")
            .send()
            .expect("https://www.googleapis.com refused to connect")
            .json().unwrap();
        let id = clean_quotes(&json
            .get("items").unwrap()
            .get(0).unwrap()
            .get("id").unwrap()
            .get("channelId").unwrap().to_string());
        YouTubeChannel {
            id,
        }
    }
    pub fn comments(&self, search_terms: &str) {
        let id = &self.id;
        loop {
            let comment_json: Value = CLIENT.get(format!("https://www.googleapis.com/youtube/v3/commentThreads?\
        part=id,snippet&\
        allThreadsRelatedToChannelId={}&\
        pageToken=&\
        order=Relevance&maxResults=100&\
        search_terms={}&\
        key={}", id, search_terms, KEY))
                .header("Referer", "https://ytcomment.kmcat.uk/")
                .send()
                .expect("https://www.googleapis.com refused to connect")
                .json().unwrap();
            let comments = comment_json
                .get("items").unwrap()
                .as_array().unwrap();
            for comment in comments {
                let comment = comment
                    .get("snippet").unwrap()
                    .get("topLevelComment").unwrap();
                let video_id = comment
                    .get("snippet").unwrap()
                    .get("videoId").unwrap().to_string();
                let comment_id = comment.get("id").unwrap().to_string();
                let link = format!("https://www.youtube.com/watch?v={}&google_comment_id={}", clean_quotes(&video_id), clean_quotes(&comment_id));
                let name = comment
                    .get("snippet").unwrap()
                    .get("authorDisplayName").unwrap().to_string();
                let text = comment
                    .get("snippet").unwrap()
                    .get("textOriginal").unwrap().to_string();

                println!("[{}] {}\n{}\n", clean_quotes(&name), clean_quotes(&text), link);
            }
            break;
        }
    }
}