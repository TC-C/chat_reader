use std::io::{stdin, stdout, Write};
use lazy_static::lazy_static;
use crate::twitch_client::TwitchClient;
use crate::twitch_vod::TwitchVOD;
use crate::twitch_channel::TwitchChannel;
use crate::twitch_clip::print_clips_from;
use crate::tools::get_filter;
use std::sync::mpsc::channel;
use std::thread;
use std::thread::JoinHandle;

pub fn main() {
    let (send, receive) = channel();
    let get_client_thread = thread::spawn(move || {
        let twitch_client = TwitchClient::new(
            String::from("cuwhphy3xzy01xn60rddmr57x8hzc6"),
            String::from("9milc7hacuyl8eg5cdpgllbdqpze9u"));
        send.send(twitch_client)
    });
    let mut search_type = String::new();
    print!("Would you like to search through entire Channel, single VOD, or clips? >>> ");
    stdout()
        .flush()
        .expect("Could not flush line when preparing for <search_type>");
    stdin()
        .read_line(&mut search_type)
        .expect("Could not read response for <search_type>");
    search_type = search_type.trim_end_matches(&['\r', '\n'][..]).to_lowercase();
    let search_type = search_type.as_str();
    get_client_thread.join();
    let client = &receive.recv().unwrap();

    match search_type {
        "vod" => input_vod(client),
        "channel" => input_channel(client),
        "clips" => get_clips(),
        _ => {
            eprintln!("\n'{}' was an unexpected response\nPlease choose between [Channel, VOD, Clips]", search_type);
            main()
        }
    }
}

fn get_clips() {
    let mut channel_name = String::new();
    print!("Input Channel Name >>> ");
    stdout()
        .flush()
        .expect("Could not flush line when preparing for <channel_name>");
    stdin()
        .read_line(&mut channel_name)
        .expect("Could not read response for <channel_name>");
    channel_name = String::from(channel_name.trim_end_matches(&['\r', '\n'][..]));
    let channel = TwitchChannel::new(channel_name);
    let filter = get_filter();
    print_clips_from(&channel, &filter)
}


fn input_channel(client: &TwitchClient) {
    let mut channel_name = String::new();
    print!("Input Channel Name >>> ");
    stdout()
        .flush()
        .expect("Could not flush line when preparing for <channel_name>");
    stdin()
        .read_line(&mut channel_name)
        .expect("Could not read response for <channel_name>");
    channel_name = String::from(channel_name.trim_end_matches(&['\r', '\n'][..]));
    let (send, receive) = std::sync::mpsc::channel();
    let get_filter_thread = thread::spawn(move || {
        let filter = get_filter();
        send.send(filter)
    });
    let channel = TwitchChannel::new(channel_name);
    let vods = channel.vods(client);
    get_filter_thread.join();
    let filter = receive.recv().unwrap();
    for vod in vods {
        let id = vod.id;
        let title = &vod.title;
        println!("\n{} v{}", title, id);
        println!("{}", vod.m3u8(&client));
        vod.print_chat(&filter, &client);
    }
}

fn input_vod(client: &TwitchClient) {
    let mut vod_id = String::new();
    print!("Input VOD ID >>> ");
    stdout()
        .flush()
        .expect("Could not flush line when preparing for <vod_id>");
    stdin()
        .read_line(&mut vod_id)
        .expect("Could not read response for <vod_id>");
    vod_id = String::from(vod_id.trim_end_matches(&['\r', '\n'][..]));
    let vod_id = vod_id.parse::<u32>().expect("Invalid vod ID, all characters must be numeric");
    let filter = get_filter();
    let vod = TwitchVOD::new(vod_id, &client);
    println!("{}", vod.m3u8(&client));
    vod.print_chat(&filter, &client)
}