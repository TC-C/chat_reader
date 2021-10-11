use crate::{
    tools::{args_filter, error, get_filter, get_input, is_valid_username},
    twitch_channel::TwitchChannel,
    twitch_clip::print_clips_from,
    twitch_vod::TwitchVOD,
};
use regex::Regex;
use std::iter::Skip;
use std::{
    env::Args,
    io::{stdin, stdout, Write},
    sync::mpsc::{channel, Sender},
    thread::{spawn, JoinHandle},
};

pub(crate) fn main() {
    loop {
        print!("Would you like to search through entire Channel, single VOD, or clips? >>> ");
        let mut search_type = get_input();
        search_type = search_type.to_lowercase();
        let search_type = search_type.as_str();
        match search_type {
            "vod" => input_vod(),
            "channel" => input_channel(),
            "clips" => get_clips(),
            _ => {
                error(format!(
                    "\n'{}' was an unexpected response\nPlease choose between [Channel, VOD, Clips]\n",
                    search_type
                ));
                continue;
            }
        }
        break;
    }
}

fn get_clips() {
    print!("Input Channel Name >>> ");
    let channel_name = get_input();
    let channel = TwitchChannel::new(&channel_name);
    let filter = match get_filter() {
        Ok(filter) => filter,
        Err(e) => return error(e),
    };
    print_clips_from(&channel, &filter);
}

pub(crate) fn args_channel(args: &mut Skip<Args>) {
    let channel_name = match args.next() {
        None => return error("-tc\n    ^^^\nNo channel name declared after `-tc`"),
        Some(channel_name) => {
            if !is_valid_username(&channel_name) {
                let mut other_args = String::new();
                for arg in args {
                    other_args += " ";
                    other_args += arg.as_ref()
                }
                return error(format!(
                    "-tc {}{:?}\n    {arrows}\nerror: invalid channel name declared after `-tc`",
                    channel_name,
                    other_args,
                    arrows = "^".repeat(channel_name.len())
                ));
            }
            channel_name
        }
    };

    let has_filter = args_has_filter(args);
    let mut filter = Regex::new("(.*?)").unwrap(); //This is a valid pattern
    if has_filter {
        filter = match args_filter(args) {
            Ok(filter) => filter,
            Err(e) => return error(e),
        };
    }
    let ch = TwitchChannel::new(&channel_name);
    let vods = match ch.vods() {
        Ok(vods) => vods,
        Err(e) => return error(e),
    };
    display_channel(vods, filter);
}

fn args_has_filter(args: &mut Skip<Args>) -> bool {
    match args.next() {
        None => false,
        Some(label) => label.eq_ignore_ascii_case("-f"),
    }
}

fn input_channel() {
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
        error(format!(
            "Channel name: {} is an invalid channel name\n",
            channel_name
        ));
        input_channel();
        return;
    }
    let ch = TwitchChannel::new(&channel_name);
    let vods = match ch.vods() {
        Ok(vods) => vods,
        Err(e) => return error(e),
    };
    let filter = match get_filter() {
        Ok(filter) => filter,
        Err(e) => return error(e),
    };
    display_channel(vods, filter)
}

fn display_channel(vods: Vec<TwitchVOD>, filter: Regex) {
    let mut threads: Vec<(
        (String, u32),
        Sender<()>,
        JoinHandle<()>,
        JoinHandle<String>,
    )> = Vec::new();
    for vod in vods {
        //The thread must own all the parameters
        let (tx, rx) = channel();
        let vod_thread = vod.to_owned();
        let filter = filter.to_owned();
        let chat_thread = spawn(move || vod_thread.print_chat(&filter, rx));
        let vod_thread = vod.to_owned();
        let url_thread = spawn(move || vod_thread.m3u8());
        threads.push(((vod.title, vod.id), tx, chat_thread, url_thread));
    }
    for reader in threads {
        let title = reader.0 .0;
        let id = reader.0 .1;
        let tx = reader.1;
        let chat_thread = reader.2;
        let url_thread = reader.3;

        println!("\n{} v{}", title, id);
        println!("{}", url_thread.join().unwrap());
        tx.send(());
        chat_thread.join().unwrap();
    }
}

pub(crate) fn args_vod(args: &mut Skip<Args>) {
    let vod_id: u32 = match args.next() {
        None => return error("-tv\n    ^^^\nNo VOD ID declared after `-tv`"),
        Some(vod_id) => match vod_id.parse() {
            Ok(vod_id) => vod_id,
            Err(e) => return error(e),
        },
    };

    let vod = match TwitchVOD::new(vod_id) {
        Ok(vod) => vod,
        Err(e) => return error(e),
    };
    let filter;
    if args_has_filter(args) {
        filter = match args_filter(args) {
            Ok(filter) => filter,
            Err(e) => return error(e),
        };
    } else {
        filter = Regex::new("(.*?)").unwrap()
    }
    vod.print_chat_blocking(&filter)
}

fn input_vod() {
    print!("Input VOD ID >>> ");
    let vod_id = get_input();
    let vod_id = vod_id.parse().unwrap();
    let vod = match TwitchVOD::new(vod_id) {
        Ok(vod) => vod,
        Err(e) => return error(e),
    };
    let filter = match get_filter() {
        Ok(filter) => filter,
        Err(e) => return error(e),
    };
    println!("{}", vod.m3u8());
    vod.print_chat_blocking(&filter)
}
