use crate::tools::CLIENT;
use serde_json::Value;
use crate::afreecatv_video::AfreecaVideo;

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
            .get("meta").expect(&vod_list_xml.to_string())
            .get("last_page").unwrap().as_u64().unwrap();
        let size = vod_list_xml
            .get("meta").unwrap()
            .get("total").unwrap().as_u64().unwrap();
        let mut videos: Vec<AfreecaVideo> = Vec::with_capacity(size as usize);
        for i in 1..limit + 1 {
            let vod_list_url = format!("https://bjapi.afreecatv.com/api/{}/vods/all?page={}&per_page=60", self.user_id, i);
            let vod_list_xml: Value = CLIENT.get(vod_list_url)
                .send()
                .expect("https://bjapi.afreecatv.com refused to connect")
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
        }
        videos
    }
}