use crate::{
    tools::{args_filter, error, get_filter, is_valid_username},
    twitch_channel::TwitchChannel,
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

pub(crate) fn main() {
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

    match search_type {
        "vod" => input_vod(),
        "channel" => input_channel(),
        "clips" => get_clips(),
        _ => {
            error(&format!(
                "\n'{}' was an unexpected response\nPlease choose between [Channel, VOD, Clips]\n",
                search_type
            ));
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
    let filter_get_thread = spawn(get_filter);
    channel_name = String::from(channel_name.trim_end_matches(&['\r', '\n'][..]));
    let channel = TwitchChannel::new(&channel_name);
    let filter = match filter_get_thread.join().unwrap() {
        Ok(filter) => filter,
        Err(e) => return error(&e.to_string()),
    };
    print_clips_from(&channel, &filter);
}

pub(crate) fn args_channel(args: &mut IntoIter<String>) {
    let channel_name = match args.next() {
        None => return error("-tc\n    ^^^\nNo channel name declared after `-tc`"),
        Some(channel_name) => {
            if !is_valid_username(&channel_name) {
                let mut other_args = String::new();
                for arg in args {
                    other_args += " ";
                    other_args += &arg
                }
                return error(&format!(
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
            Err(e) => return error(&e.to_string()),
        };
    }
    let ch = TwitchChannel::new(&channel_name);
    let vods = match ch.vods() {
        Ok(vods) => vods,
        Err(e) => return error(&e),
    };

    display_channel(vods, filter);
}

fn args_has_filter(args: &mut IntoIter<String>) -> bool {
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
        error(&format!(
            "Channel name: {} is an invalid channel name\n",
            channel_name
        ));
        input_channel();
        return;
    }
    let get_filter_thread = spawn(get_filter);
    let ch = TwitchChannel::new(&channel_name);
    let vods = match ch.vods() {
        Ok(vods) => vods,
        Err(e) => return error(&e),
    };
    let filter = match get_filter_thread.join().unwrap() {
        Ok(filter) => filter,
        Err(e) => return error(&e.to_string()),
    };
    display_channel(vods, filter)
}

fn display_channel(vods: Vec<TwitchVOD>, filter: Regex) {
    let mut threads: Vec<(TwitchVOD, Sender<()>, JoinHandle<()>)> = Vec::new();
    for vod in vods {
        //The thread must own all the parameters
        let (tx, rx) = channel();
        let vod_thread = vod.to_owned();
        let filter = filter.to_owned();
        let chat_thread = spawn(move || vod_thread.print_chat(&filter, rx));

        threads.push((vod, tx, chat_thread));
    }
    for reader in threads {
        let vod = reader.0;
        let tx = reader.1;
        let chat_thread = reader.2;

        println!("\n{} v{}", vod.title, vod.id);
        println!("{}", vod.m3u8());
        tx.send(());
        chat_thread.join().unwrap();
    }
}

pub(crate) fn args_vod(args: &mut IntoIter<String>) {
    let vod_id = match args.next() {
        None => return error("-tv\n    ^^^\nNo VOD ID declared after `-tv`"),
        Some(vod_id) => match vod_id.parse::<u32>() {
            Ok(vod_id) => vod_id,
            Err(e) => return error(&e.to_string()),
        },
    };

    let vod = match TwitchVOD::new(vod_id) {
        Ok(vod) => vod,
        Err(e) => return error(&e),
    };
    let has_filter = args_has_filter(args);
    let filter;
    if has_filter {
        filter = match args_filter(args) {
            Ok(filter) => filter,
            Err(e) => return error(&e.to_string()),
        };
    } else {
        filter = Regex::new("(.*?)").unwrap()
    }
    vod.print_chat_blocking(&filter)
}

fn input_vod() {
    let mut vod_id = String::new();
    print!("Input VOD ID >>> ");
    stdout().flush().unwrap();
    stdin().read_line(&mut vod_id).unwrap();
    vod_id = String::from(vod_id.trim_end_matches(&['\r', '\n'][..]));
    let vod_id = vod_id.parse::<u32>().unwrap();
    let get_filter_thread = spawn(get_filter);
    let vod = match TwitchVOD::new(vod_id) {
        Ok(vod) => vod,
        Err(e) => return error(&e),
    };
    let filter = match get_filter_thread.join().unwrap() {
        Ok(filter) => filter,
        Err(e) => return error(&e.to_string()),
    };
    println!("{}", vod.m3u8());
    vod.print_chat_blocking(&filter)
}
