use crate::{
    tools::{exit_error, get_input},
    {afreecatv_channel::Blog, afreecatv_video::AfreecaVideo, tools::get_filter},
};

use std::{
    sync::mpsc::channel,
    thread::spawn,
};

pub(crate) fn main() {
    loop {
        print!("Would you like to search through entire Blog or single Video? >>> ");
        let mut search_type = get_input();
        search_type = search_type.to_lowercase();
        let search_type = search_type.as_str();

        match search_type {
            "video" => input_vod(),
            "blog" => input_blog(),
            _ => {
                eprintln!(
                    "\n'{}' was an unexpected response\nPlease choose between [Blog, Video]\n",
                    search_type
                );
                continue;
            }
        }
        break;
    }
}

pub(crate) fn input_vod() {
    print!("Input VOD Link >>> ");
    let vod_link = get_input();
    dbg!(&vod_link);
    let video_get_thread = spawn(move || AfreecaVideo::new(&vod_link));
    let filter = match get_filter() {
        Ok(filter) => filter,
        Err(e) => exit_error(e),
    };
    let video = match video_get_thread.join().unwrap() {
        Ok(video) => video,
        Err(e) => exit_error(e),
    };
    video.print_chat_blocking(&filter);
}

pub(crate) fn input_blog() {
    print!("Input Blog Name >>> ");
    let blog_name = get_input();
    let videos_get_thread = spawn(move || Blog::new(&blog_name).videos());
    let filter = match get_filter() {
        Ok(filter) => filter,
        Err(e) => exit_error(e),
    };
    let videos = videos_get_thread.join().unwrap();
    let mut threads = Vec::new();
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
