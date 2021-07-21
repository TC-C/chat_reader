use crate::tools::CLIENT;
use serde_json::Value;
use crate::afreecatv_video::AfreecaVideo;
use std::thread::{spawn, JoinHandle};

#[derive(Clone)]
pub struct Blog {
    user_id: String,
}

impl Blog {
    pub fn new(user_id: &str) -> Blog {
        Blog {
            user_id: user_id.to_string()
        }
    }
    pub fn videos(self) -> Vec<AfreecaVideo> {
        let vod_list_url = format!("https://bjapi.afreecatv.com/api/{}/vods/all?per_page=60", self.user_id);
        let vod_list_xml: Value = CLIENT.get(vod_list_url)
            .header("Connection", "keep-alive")
            .send()
            .expect("https://bjapi.afreecatv.com refused to connect")
            .json().unwrap();
        let limit = vod_list_xml
            .get("meta").unwrap()
            .get("last_page").unwrap().as_u64().unwrap();
        let size = vod_list_xml
            .get("meta").unwrap()
            .get("total").unwrap().as_u64().unwrap();
        let mut videos: Vec<AfreecaVideo> = Vec::with_capacity(size as usize);

        let mut page_chunks: Vec<JoinHandle<Vec<AfreecaVideo>>> = Vec::with_capacity(limit as usize);

        for i in 1..limit + 1 {
            let blog = self.to_owned();
            let retrieval_thread = spawn(move || blog.load_videos_chunk(i));
            page_chunks.push(retrieval_thread)
        }
        for page_chunk in page_chunks {
            let mut thread_videos = page_chunk.join().unwrap();
            videos.append(&mut thread_videos)
        }
        videos
    }

    fn load_videos_chunk(&self, i: u64) -> Vec<AfreecaVideo> {
        let mut videos: Vec<AfreecaVideo> = Vec::with_capacity(60);
        //let mut video_threads: Vec<JoinHandle<AfreecaVideo>> = Vec::with_capacity(60);
        let vod_list_url = format!("https://bjapi.afreecatv.com/api/{}/vods/all?page={}&per_page=60", self.user_id, i);
        let vod_list_xml: Value = CLIENT.get(vod_list_url)
            .send()
            .unwrap()
            .json().unwrap();
        //println!("{}", vod_list_xml);
        let vods = vod_list_xml
            .get("data").unwrap()
            .as_array().unwrap();
        for vod in vods {
            let title_no = vod.get("title_no").unwrap().to_string();
            let station_no = vod.get("station_no").unwrap().to_string();
            let bbs_no = vod.get("bbs_no").unwrap().to_string();
            let video = AfreecaVideo::new_unchecked(&title_no, &station_no, &bbs_no);
            videos.push(video);
        }
        videos
    }
}