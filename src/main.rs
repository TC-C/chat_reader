mod twitch_reader;
mod twitch_client;

use std::io::{stdin, stdout, Write};

fn main() {
    let mut platform_name = String::new();
    print!("What platform would you link to pull from (Twitch)? >>> ");
    stdout()
        .flush()
        .expect("Could not flush line when preparing for <vod_link>");
    stdin()
        .read_line(&mut platform_name)
        .expect("Could not read response for <vod_link>");
    platform_name = String::from(platform_name.trim_end_matches(&['\r', '\n'][..]));

    if platform_name.eq_ignore_ascii_case("Twitch") {
        twitch_reader::main()
    } else {
        eprintln!("\n'{}' was an unexpected response\nPlease choose between [Twitch]", platform_name)
    }
}
