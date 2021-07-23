use lazy_static::lazy_static;
use regex::Regex;
use reqwest::blocking::Client;
use std::{
    io::{stdout, stdin, Write},
    vec::IntoIter,
    process::exit,
};
lazy_static! {pub(crate) static ref CLIENT: Client = Client::new();}

pub(crate) fn clean_quotes(string: &str) -> String {
    string.trim_start_matches("\"").trim_end_matches("\"").to_string()
}

pub(crate) fn format_time_string(seconds: &str) -> String {
    let seconds: f32 = seconds.parse().unwrap();
    let seconds = seconds as u32;
    format_time(seconds)
}

/// function to call println! on all `String`s in a Vec whilst emptying it
pub(crate) fn print_queue(comment_queue: &mut Vec<String>) {
    let cq = comment_queue.into_iter();
    for comment in cq {
        println!("{}", comment)
    }
    comment_queue.clear()
}

pub fn format_time(seconds: u32) -> String {
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

pub(crate) fn get_filter() -> Regex {
    let mut filter = String::new();
    print!("(RegExp) Please enter a phrase you would like to search for >>> ");
    stdout()
        .flush()
        .expect("Could not flush line when preparing for <filter>");
    stdin()
        .read_line(&mut filter)
        .expect("Could not read response for <filter>");
    filter = String::from(filter.trim_end_matches(&['\r', '\n'][..]));
    Regex::new(&format!(r"(?i)({})", filter)).unwrap()
}

pub(crate) fn args_filter(args: &mut IntoIter<String>) -> Regex {
    let re = match args.next() {
        None => String::new(),
        Some(re) => re
    };
    match Regex::new(&format!(r"(?i)({})", re)) {
        Ok(filter) => filter,
        Err(_) => {
            eprintln!("Invalid regex pattern: {}", re);
            exit(-1)
        }
    }
}

pub(crate) fn extract_digits(s: &str) -> u32 {
    let output: String = s.chars().filter(|c| c.is_numeric()).collect();
    output.parse::<u32>().unwrap()
}