#[path = "twitch/twitch_reader.rs"]
mod twitch_reader;
#[path = "twitch/twitch_client.rs"]
mod twitch_client;
#[path = "twitch/twitch_vod.rs"]
mod twitch_vod;
#[path = "twitch/twitch_channel.rs"]
mod twitch_channel;
#[path = "twitch/twitch_clip.rs"]
mod twitch_clip;
#[path = "afreecatv/afreecatv_channel.rs"]
mod afreecatv_channel;
#[path = "afreecatv/afreecatv_video.rs"]
mod afreecatv_video;
#[path = "afreecatv/afreecatv_reader.rs"]
mod afreecatv_reader;
#[path = "youtube/youtube_channel.rs"]
mod youtube_channel;
#[path = "youtube/youtube_reader.rs"]
mod youtube_reader;

mod tools;

use std::{
    env,
    io::{stdin, stdout, Write},
    vec::IntoIter,
    process::exit,
};

fn main_args(mut args: IntoIter<String>) {
    loop {
        match args.next() {
            None => break,
            Some(arg) => {
                let arg = arg.as_str();
                match arg {
                    "-tc" => twitch_reader::args_channel(&mut args),
                    "-tv" => twitch_reader::args_vod(&mut args),
                    &_ => exit(-1)
                }
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut args = args.into_iter();
    args.next();
    if args.len() > 0 {
        main_args(args);
        return;
    }
    let mut platform_name = String::new();
    print!("What platform would you link to pull from (Twitch, AfreecaTV, YouTube)? >>> ");
    stdout()
        .flush()
        .expect("Could not flush line when preparing for <vod_link>");
    stdin()
        .read_line(&mut platform_name)
        .expect("Could not read response for <vod_link>");
    platform_name = platform_name.trim_end_matches(&['\r', '\n'][..]).to_lowercase();
    let platform_name = platform_name.as_str();

    match platform_name {
        "twitch" => twitch_reader::main(),
        "afreecatv" => afreecatv_reader::main(),
        "youtube" => youtube_reader::main(),
        _ => {
            eprintln!("\n'{}' was an unexpected response\nPlease choose between [Twitch, AfreecaTV, YouTube]\n", platform_name);
            main()
        }
    }
}