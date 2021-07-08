use std::io::{stdout, stdin, Write};
use crate::afreecatv_video::{get_video_info, print_video_chat};

pub fn main() {
    let mut vod_link = String::new();
    print!("Input VOD Link >>> ");
    stdout()
        .flush()
        .expect("Could not flush line when preparing for <channel_name>");
    stdin()
        .read_line(&mut vod_link)
        .expect("Could not read response for <channel_name>");
    vod_link = String::from(vod_link.trim_end_matches(&['\r', '\n'][..]));
    let video_info_url = get_video_info(&vod_link);
    print_video_chat(&video_info_url);
}