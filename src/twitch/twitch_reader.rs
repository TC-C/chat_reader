use std::io::{stdin, stdout, Write};
use crate::twitch_client::TwitchClient;
use crate::twitch_vod::TwitchVOD;
use crate::twitch_channel::TwitchChannel;
use crate::twitch_clip::print_clips_from;
use crate::tools::get_filter;
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::thread::JoinHandle;
use std::collections::VecDeque;

pub fn main() {
    let (tx, rx) = channel();
    let get_client_thread = thread::spawn(move || {
        let twitch_client = TwitchClient::new(
            "cuwhphy3xzy01xn60rddmr57x8hzc6",
            "9milc7hacuyl8eg5cdpgllbdqpze9u");
        tx.send(twitch_client)
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
    let client = rx.recv().unwrap();

    match search_type {
        "vod" => input_vod(&client),
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
    let channel = TwitchChannel::new(&channel_name);
    let filter = get_filter();
    print_clips_from(&channel, &filter)
}


fn input_channel(client: TwitchClient) {
    let mut channel_name = String::new();
    print!("Input Channel Name >>> ");
    stdout()
        .flush()
        .expect("Could not flush line when preparing for <channel_name>");
    stdin()
        .read_line(&mut channel_name)
        .expect("Could not read response for <channel_name>");
    channel_name = String::from(channel_name.trim_end_matches(&['\r', '\n'][..]));
    let (tx, rx) = channel();
    let get_filter_thread = thread::spawn(move || {
        let filter = get_filter();
        tx.send(filter)
    });
    let ch = TwitchChannel::new(&channel_name);
    let vods = ch.vods(&client);
    get_filter_thread.join();
    let filter = rx.recv().unwrap();

    let mut threads: VecDeque<(TwitchVOD, Sender<bool>, JoinHandle<()>)> = VecDeque::new();
    for vod in vods {
        //The thread must own all the parameters
        let (tx, rx) = channel();
        let vod_thread = vod.to_owned();
        let filter = filter.to_owned();
        let client = client.to_owned();
        let chat_thread = thread::spawn(move || vod_thread.print_chat(&filter, &client, rx));

        threads.push_back((vod, tx, chat_thread));
    }
    for reader in threads {
        let vod = reader.0;
        let tx = reader.1;
        let chat_thread = reader.2;

        let title = &vod.title;
        let id = vod.id;

        println!("\n{} v{}", title, id);
        println!("{}", vod.m3u8(&client));
        tx.send(true);
        chat_thread.join();
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
    let vod_id = vod_id.parse::<u32>().unwrap();
    let filter = get_filter();
    let vod = TwitchVOD::new(vod_id, client);
    println!("{}", vod.m3u8(client));
    vod.print_chat_blocking(&filter, client)
}