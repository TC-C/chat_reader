use std::io::{stdout, stdin, Write};
use crate::afreecatv_video::AfreecaVideo;
use crate::afreecatv_channel::Blog;
use crate::tools::get_filter;
use std::thread::spawn;
use std::sync::mpsc::{Receiver, channel};

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
        _ => {
            eprintln!("\n'{}' was an unexpected response\nPlease choose between [Blog, Video]\n", search_type);
            main()
        }
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
    let video_get_thread = spawn(move || AfreecaVideo::new(&vod_link));
    let filter = get_filter();
    let video = video_get_thread.join().unwrap();
    let (tx, rx) = channel();
    tx.send(());
    video.print_chat(&filter, rx);
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
    let videos_get_thread = spawn(move || {
        let blog = Blog::new(&blog_name);
        blog.videos()
    });

    let filter = get_filter();
    let videos = videos_get_thread.join().unwrap();
    for video in videos {
        let (tx, rx) = channel();
        tx.send(());

        println!("\nWorking on: {}", video.title_no);
        video.print_chat(&filter, rx);
    }
}