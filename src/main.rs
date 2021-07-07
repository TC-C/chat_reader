use std::io::{stdin, stdout, Write};

fn main() {
    let mut vod_link = String::new();
    println!("This program supports Twitch VOD links");
    print!("Enter link to VOD >>> ");
    stdout()
        .flush()
        .expect("Could not flush line when preparing for <vod_link>");
    stdin()
        .read_line(&mut vod_link)
        .expect("Could not read response <vod_link>");
    vod_link = String::from(vod_link.trim_end_matches(&['\r', '\n'][..]));
}
