use std::io::{stdout, stdin, Write};
use crate::afreecatv_video::AfreecaVideo;
use crate::afreecatv_channel::Blog;
use crate::tools::get_filter;

pub fn main() {
    let mut search_type = String::new();
    print!("Would you like to search through entire Blog or single Video? >>> ");
    stdout()
        .flush()
        .expect("Could not flush line when preparing for <search_type>");
    stdin()
        .read_line(&mut search_type)
        .expect("Could not read response for <search_type>");
    search_type = search_type.trim_end_matches(&['\r', '\n'][..]).to_lowercase();
    let search_type = search_type.as_str();

    match search_type {
        "video" => input_vod(),
        "blog" => input_blog(),
        _ => eprintln!("\n'{}' was an unexpected response\nPlease choose between [Blog, Video]", search_type)
    }
}

pub fn input_vod() {
    let mut vod_link = String::new();
    print!("Input VOD Link >>> ");
    stdout()
        .flush()
        .expect("Could not flush line when preparing for <vod_link>");
    stdin()
        .read_line(&mut vod_link)
        .expect("Could not read response for <vod_link>");
    vod_link = String::from(vod_link.trim_end_matches(&['\r', '\n'][..]));
    let video = AfreecaVideo::new(&vod_link);
    let filter = get_filter();
    video.print_chat(&filter);
}

pub fn input_blog() {
    let mut blog_name = String::new();
    print!("Input Blog Name >>> ");
    stdout()
        .flush()
        .expect("Could not flush line when preparing for <blog_name>");
    stdin()
        .read_line(&mut blog_name)
        .expect("Could not read response for <blog_name>");
    blog_name = String::from(blog_name.trim_end_matches(&['\r', '\n'][..]));
    let blog = Blog::new(&blog_name);
    let videos = blog.videos();
    let filter = get_filter();
    for video in videos {
        println!("\nWorking on: {}", video.title_no);
        video.print_chat(&filter);
    }
}