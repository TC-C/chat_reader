use lazy_static::lazy_static;
use regex::Regex;
use roxmltree::{Document, Node, Error};
use reqwest::header::COOKIE;
use crate::tools::{CLIENT, extract_digits, format_time};
lazy_static! {
	//working on initial URL
	static ref title_no_matcher: Regex = Regex::new("STATION/[0-9]{8}").unwrap();
	static ref bbs_no_matcher: Regex = Regex::new("nBbsNo=[0-9]{8}").unwrap();
	static ref station_no_matcher: Regex = Regex::new("nStationNo=[0-9]{8}").unwrap();
	//working on stbbs info page
	static ref row_key_matcher: Regex = Regex::new("key=\".*\">").unwrap();
	static ref row_time_matcher: Regex = Regex::new(r#"file duration=".*" "#).unwrap();
}
//Dummy account for accessing age restricted VODs
const cookie: &str = "PdboxTicket=.A32.7bbT56vyHM9fKZk.SCwwbeEYGl-\
_RK8offHEfHRYug37IvxHp0iHV0ZjIqUgEYDviDxevQx01PU6\
-AIlExXpKM5FEovtC9uP5EjNQPDwZy2I1EjK9l8WItbBrj5hT7jYYNI34878csX4CiR0cVbPPGjlXxk3U_b3F6jxpL7wjHq1\
-Bn7H9-CeE-OCrOn1b_4A-pWHT-\
hESimjmpn4vuuyKPahezPgzUYwUI6aI40vce6AiWkFZDM6314tglYTo0fMpjqJBeAyBlmvEdT7_JGXCAbtp39IQmLMHCchKM2YElF6kvSpQwCAeKlU5EXb4gm92kcnVU6AoIhxTKBDkpMLeVAHF\
-Z70CuVorJoDrfKCyqL6MrSExwEwxH3b6qKttgtSz6BvEzXg1drLUU6gKfg1m1mUDaHuK_wvDOCEZX7sKcdmEoeuyMrC\
-1rTCrwsP3m5a_vTK1UrHmAcRT3H8biTle_u6_pjf8Z0JGLjES_3rzTJ9YNH5UFcZ2FyA0nU2nPReG9wirYCspxG3FoZax7zYkhLcFJWy6j1cVpts2N_5kzybkwQvk03JPVGfS9o0ZP3EeqyRAJAY8g_OX;";

pub(crate) fn get_video_info(url: &String) -> String {
    let view_source = CLIENT.get(url)
        .send().unwrap().text().unwrap();
    let station_no = station_no_matcher.find(&view_source)
        .unwrap().as_str()[11..].parse::<u32>().unwrap();
    let bbs_no = bbs_no_matcher.find(&view_source)
        .unwrap().as_str()[7..].parse::<u32>().unwrap();
    let title_no = title_no_matcher.find(url)
        .expect("Invalid URL!").as_str()[8..].parse::<u32>().unwrap();
    format!("https://stbbs.afreecatv.com/api/video/get_video_info.php?nStationNo={}&nBbsNo={}&nTitleNo={}", station_no, bbs_no, title_no)
}

pub(crate) fn print_video_chat(info_url: &String) {
    let xml = CLIENT.get(info_url)
        .header(COOKIE, cookie)
        .send().expect("https://stbbs.afreecatv.com refused to connect")
        .text().unwrap();
    let mut row_key_iterator = row_key_matcher.find_iter(&xml);
    let mut row_time_iterator = row_time_matcher.find_iter(&xml);
    let mut timestamp_secs_added = 0;
    loop {
        let row_key_regex = match row_key_iterator.next() {
            Some(s) => s,
            None => break
        };
        let row_key = row_key_regex.as_str()[5..34].to_string();
        let row_time = extract_digits(row_time_iterator.next().unwrap().as_str());
        let mut curr_secs = 0;

        loop {
            let transcript_url = format!("https://videoimg.afreecatv.com/php/ChatLoadSplit.php?rowKey={}_c&startTime={}", row_key, curr_secs);
            let xml = CLIENT.get(&transcript_url)
                .send().expect("https://videoimg.afreecatv.com refused to connect")
                .text().unwrap();
            let doc = match Document::parse(&xml) {
                Ok(d) => { d }
                Err(_) => break
            };
            let nodes = doc.root().descendants();
            for node in nodes {
                if node.tag_name().name() != "chat" {
                    continue;
                }
                let comment: Vec<Node> = node.children().map(Node::from).collect();
                let name = comment[3].text().unwrap();
                let message = comment[4].text().unwrap();
                let time = comment[6].text().unwrap().parse::<f32>().unwrap() as u32;
                println!("[{}][{}]: {}", format_time(time + timestamp_secs_added), name, message);
            }
            if curr_secs > row_time {
                timestamp_secs_added += row_time;
                break;
            } else { curr_secs += 300; }
        }
    }
}