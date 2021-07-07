use std::io::{stdin, stdout, Write};
use crate::twitch_client::TwitchClient;

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
    let client = TwitchClient::new(
        String::from("cuwhphy3xzy01xn60rddmr57x8hzc6"),
        String::from("9milc7hacuyl8eg5cdpgllbdqpze9u"));
}