use std::io::{stdin, stdout, Write};
use crate::twitch_client::TwitchClient;
use crate::twitch_vod::TwitchVOD;
use crate::twitch_channel::TwitchChannel;
use lazy_static::lazy_static;
use crate::twitch_clip::print_clips_from;
use crate::tools::get_filter;

lazy_static! {static ref client: TwitchClient = TwitchClient::new(
        String::from("cuwhphy3xzy01xn60rddmr57x8hzc6"),
        String::from("9milc7hacuyl8eg5cdpgllbdqpze9u"));
}


pub fn main() {
    let mut search_type = String::new();
    print!("Would you like to search through entire Channel, single VOD, or clips? >>> ");
    stdout()
        .flush()
        .expect("Could not flush line when preparing for <search_type>");
    stdin()
        .read_line(&mut search_type)
        .expect("Could not read response for <search_type>");
    search_type = String::from(search_type.trim_end_matches(&['\r', '\n'][..]));

    if search_type.eq_ignore_ascii_case("VOD") {
        input_vod()
    } else if search_type.eq_ignore_ascii_case("Channel") {
        input_channel()
    } else if search_type.eq_ignore_ascii_case("Clips") {
        get_clips()
    } else {
        eprintln!("\n'{}' was an unexpected response\nPlease choose between [Channel, VOD, Clips]", search_type)
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
    print_clips_from(channel, filter)
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
    let channel = TwitchChannel::new(channel_name);
    let vods = channel.vods(&client);
    for vod in vods {
        let id = vod.id;
        let title = &vod.title;
        println!("\n{} v{}", title, id);
        let vod = vod;
        let filter = get_filter();
        vod.print_chat(filter, &client);
    }
}

fn input_vod() {
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

    let vod = TwitchVOD::new(vod_id, &client);
    vod.m3u8(&client);
    //vod.print_chat(String::from(""), client)
}