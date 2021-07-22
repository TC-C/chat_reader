use std::io::{stdout, stdin, Write};
use crate::afreecatv_video::AfreecaVideo;
use crate::afreecatv_channel::Blog;
use crate::tools::get_filter;
use std::thread::{spawn, JoinHandle};
use std::sync::mpsc::{channel, Sender};

pub(crate) fn main() {
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

pub(crate) fn input_vod() {
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
    video.print_chat_blocking(&filter);
}

pub(crate) fn input_blog() {
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

    let mut threads: Vec<(AfreecaVideo, Sender<()>, JoinHandle<()>)> = Vec::new();
    for video in videos {
        let (tx, rx) = channel();
        let filter = filter.to_owned();
        let video_thread = video.to_owned();
        let thread = spawn(move || video_thread.print_chat(&filter, rx));
        threads.push((video, tx, thread))
    }
    for reader in threads {
        let video = reader.0;
        let tx = reader.1;
        let chat_thread = reader.2;

        println!("\nWorking on: {}", video.title_no);
        tx.send(());
        chat_thread.join();
    }
}