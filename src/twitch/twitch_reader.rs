use crate::tools::is_valid_username;
use crate::{
    tools::{args_filter, get_filter},
    twitch_channel::TwitchChannel,
    twitch_client::TwitchClient,
    twitch_clip::print_clips_from,
    twitch_vod::TwitchVOD,
};
use regex::Regex;
use std::{
    io::{stdin, stdout, Write},
    sync::mpsc::{channel, Sender},
    thread::{spawn, JoinHandle},
    vec::IntoIter,
};
use termion::color::{Fg, Red, Reset};

pub(crate) fn main() {
    let client_get_thread = spawn(move || {
        TwitchClient::new_unchecked(
            "kimne78kx3ncx6brgo4mv6wki5h1ko",
            "e5iag6itee2k2j2xwqxyfwebo0yrcg",
        )
    });
    let mut search_type = String::new();
    print!("Would you like to search through entire Channel, single VOD, or clips? >>> ");
    stdout()
        .flush()
        .expect("Could not flush line when preparing for <search_type>");
    stdin()
        .read_line(&mut search_type)
        .expect("Could not read response for <search_type>");
    search_type = search_type
        .trim_end_matches(&['\r', '\n'][..])
        .to_lowercase();
    let search_type = search_type.as_str();
    let client = client_get_thread.join().unwrap(); //client_get_thread only joins here, so unwrap() is safe

    match search_type {
        "vod" => input_vod(&client),
        "channel" => input_channel(client),
        "clips" => get_clips(client),
        _ => {
            eprintln!(
                "\n'{}' was an unexpected response\nPlease choose between [Channel, VOD, Clips]",
                search_type
            );
            main()
        }
    }
}

fn get_clips(client: TwitchClient) {
    let mut channel_name = String::new();
    print!("Input Channel Name >>> ");
    stdout()
        .flush()
        .expect("Could not flush line when preparing for <channel_name>");
    stdin()
        .read_line(&mut channel_name)
        .expect("Could not read response for <channel_name>");
    let filter_get_thread = spawn(get_filter);
    channel_name = String::from(channel_name.trim_end_matches(&['\r', '\n'][..]));
    let channel = TwitchChannel::new(&channel_name);
    let filter = match filter_get_thread.join().unwrap() {
        Ok(filter) => filter,
        Err(e) => {
            panic!("{red}{}{reset}", e, red = Fg(Red), reset = Fg(Reset));
        }
    };
    print_clips_from(&channel, &filter, &client);
}

pub(crate) fn args_channel(args: &mut IntoIter<String>) {
    let client = TwitchClient::new_unchecked(
        "kimne78kx3ncx6brgo4mv6wki5h1ko",
        "hfcm528b89m5eyturgicl5k6jpx2cb",
    );
    let channel_name = match args.next() {
        None => {
            panic!("No channel declared after `-tc`");
        }
        Some(channel_name) => {
            if !is_valid_username(&channel_name) {
                panic!("Channel name: {} is an invalid username", channel_name);
            }
            channel_name
        }
    };

    let has_filter = args_has_filter(args);
    let mut filter = Regex::new("(.*?)").unwrap(); //This is a valid pattern
    if has_filter {
        filter = match args_filter(args) {
            Ok(filter) => filter,
            Err(e) => panic!("{red}{}{reset}", e, red = Fg(Red), reset = Fg(Reset)),
        };
    }
    let ch = TwitchChannel::new(&channel_name);
    let vods = ch.vods(&client);

    display_channel(&client, vods, filter);
}

fn args_has_filter(args: &mut IntoIter<String>) -> bool {
    match args.next() {
        None => false,
        Some(label) => label.eq_ignore_ascii_case("-f"),
    }
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
    if !is_valid_username(&channel_name) {
        eprintln!("Channel name: {} is an invalid username\n", channel_name);
        input_channel(client);
        return;
    }
    let get_filter_thread = spawn(get_filter);
    let ch = TwitchChannel::new(&channel_name);
    let vods = ch.vods(&client);
    let filter = match get_filter_thread.join().unwrap() {
        Ok(filter) => filter,
        Err(e) => panic!("{red}{}{reset}", e, red = Fg(Red), reset = Fg(Reset)),
    };

    display_channel(&client, vods, filter)
}

fn display_channel(client: &TwitchClient, vods: Vec<TwitchVOD>, filter: Regex) {
    let mut threads: Vec<(TwitchVOD, Sender<()>, JoinHandle<()>)> = Vec::new();
    for vod in vods {
        //The thread must own all the parameters
        let (tx, rx) = channel();
        let vod_thread = vod.to_owned();
        let filter = filter.to_owned();
        let client = client.to_owned();
        let chat_thread = spawn(move || vod_thread.print_chat(&filter, &client, rx));

        threads.push((vod, tx, chat_thread));
    }
    for reader in threads {
        let vod = reader.0;
        let tx = reader.1;
        let chat_thread = reader.2;

        println!("\n{} v{}", vod.title, vod.id);
        println!("{}", vod.m3u8(client));
        tx.send(());
        chat_thread.join().unwrap();
    }
}

pub(crate) fn args_vod(args: &mut IntoIter<String>) {
    let client = &TwitchClient::new(
        "cuwhphy3xzy01xn60rddmr57x8hzc6",
        "9milc7hacuyl8eg5cdpgllbdqpze9u",
    );
    let vod_id = match args.next() {
        None => panic!("No VOD ID declared after `-tv`"),
        Some(vod_id) => match vod_id.parse::<u32>() {
            Ok(vod_id) => vod_id,
            Err(e) => panic!("{red}{}{reset}", e, red = Fg(Red), reset = Fg(Reset)),
        },
    };

    let vod = TwitchVOD::new(vod_id, client);
    let has_filter = args_has_filter(args);
    let mut filter = Regex::new("(.*?)").unwrap();
    if has_filter {
        filter = match args_filter(args) {
            Ok(filter) => filter,
            Err(e) => panic!("{red}{}{reset}", e, red = Fg(Red), reset = Fg(Reset)),
        };
    }
    vod.print_chat_blocking(&filter, client)
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
    let get_filter_thread = spawn(get_filter);
    let vod = TwitchVOD::new(vod_id, client);
    let filter = match get_filter_thread.join().unwrap() {
        Ok(filter) => filter,
        Err(e) => panic!("{red}{}{reset}", e, red = Fg(Red), reset = Fg(Reset)),
    };
    println!("{}", vod.m3u8(client));
    vod.print_chat_blocking(&filter, client)
}
