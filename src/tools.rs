use std::io::{stdout, stdin, Write};
use regex::Regex;

pub fn clean_quotes(string: &String) -> String {
    string.trim_start_matches("\"").trim_end_matches("\"").to_string()
}

pub fn format_time(seconds: String) -> String {
    let seconds: f64 = seconds.parse().unwrap();
    let seconds = seconds as i16;

    let mut hours = (seconds / (60 * 60)).to_string();
    if hours.len() == 1 {
        hours = format!("0{}", hours);
    }
    let mut minutes = (seconds / 60 % 60).to_string();
    if minutes.len() == 1 {
        minutes = format!("0{}", minutes);
    }
    let mut seconds = (seconds % 60).to_string();
    if seconds.len() == 1 {
        seconds = format!("0{}", seconds);
    }
    return format!("{}:{}:{}", hours, minutes, seconds);
}

pub fn get_filter() -> Regex {
    let mut filter = String::new();
    print!("Please enter a phrase you would like to search for >>> ");
    stdout()
        .flush()
        .expect("Could not flush line when preparing for <filter>");
    stdin()
        .read_line(&mut filter)
        .expect("Could not read response for <filter>");
    filter = String::from(filter.trim_end_matches(&['\r', '\n'][..]));
    Regex::new(&format!(r"(?i)({})", filter)).expect("Invalid Regex pattern!")
}