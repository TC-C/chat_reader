use crate::youtube_channel::YouTubeChannel;
use std::io::{stdin, stdout, Write};

pub fn main() {
    let mut channel_name = String::new();
    print!("Input Channel Name >>> ");
    stdout()
        .flush()
        .expect("Could not flush line when preparing for <channel_name>");
    stdin()
        .read_line(&mut channel_name)
        .expect("Could not read response for <channel_name>");
    channel_name = String::from(channel_name.trim_end_matches(&['\r', '\n'][..]));
    let channel = YouTubeChannel::new(channel_name);
    let filter = get_filter();
    channel.comments(&filter)
}

fn get_filter() -> String {
    let mut filter = String::new();
    print!("Please enter a query you would like to search for >>> ");
    stdout()
        .flush()
        .expect("Could not flush line when preparing for <filter>");
    stdin()
        .read_line(&mut filter)
        .expect("Could not read response for <filter>");
    String::from(filter.trim_end_matches(&['\r', '\n'][..]))
}