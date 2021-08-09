use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
};
use lazy_static::lazy_static;
use regex::{Error, Regex};
use reqwest::blocking::Client;
use std::{
    io::{stdin, stdout, Write},
    num::{ParseFloatError, ParseIntError},
    process::exit,
    vec::IntoIter,
};

pub(crate) const CLIENT_ID: &str = "kimne78kx3ncx6brgo4mv6wki5h1ko";
lazy_static! {
    pub(crate) static ref CLIENT: Client = Client::new();
    static ref USERNAME_VALIDATE: Regex = Regex::new(r#"^[a-zA-Z0-9][\w]{3,24}$"#).unwrap();
}

pub(crate) fn clean_quotes(str: &str) -> String {
    str.trim_start_matches('"')
        .trim_end_matches('"')
        .to_string()
}

pub(crate) fn format_time_string(seconds: &str) -> Result<String, ParseFloatError> {
    let seconds: f32 = match seconds.parse() {
        Ok(seconds) => seconds,
        Err(e) => return Err(e),
    };
    let seconds = seconds as u32;
    Ok(format_time(seconds))
}

/// Function to call println! on all `String`s in a Vec whilst emptying it
pub(crate) fn print_queue(comment_queue: &mut Vec<String>) {
    for comment in comment_queue.iter_mut() {
        println!("{}", comment)
    }
    comment_queue.clear()
}

pub(crate) fn hex_to_rgb(hex: &str) -> Result<Color, ParseIntError> {
    let hex = hex.trim_start_matches('#');
    const RADIX: u32 = 16;
    let r = match u8::from_str_radix(&hex[0..2], RADIX) {
        Ok(r) => r,
        Err(e) => return Err(e),
    };
    let g = match u8::from_str_radix(&hex[2..4], RADIX) {
        Ok(g) => g,
        Err(e) => return Err(e),
    };
    let b = match u8::from_str_radix(&hex[4..6], RADIX) {
        Ok(b) => b,
        Err(e) => return Err(e),
    };
    Ok(Color::parse_ansi(&format!("2;{};{};{}", r, g, b)).unwrap())
}

pub(crate) fn format_time(seconds: u32) -> String {
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

pub(crate) fn get_filter() -> Result<Regex, Error> {
    let mut re = String::new();
    print!("(RegExp) Please enter a phrase you would like to search for >>> ");
    stdout()
        .flush()
        .expect("Could not flush line when preparing for <filter>");
    stdin()
        .read_line(&mut re)
        .expect("Could not read response for <filter>");
    re = String::from(re.trim_end_matches(&['\r', '\n'][..]));
    Regex::new(&format!(r"(?i)({})", re))
}

pub(crate) fn is_valid_username(username: &str) -> bool {
    USERNAME_VALIDATE.is_match(username)
}

pub(crate) fn args_filter(args: &mut IntoIter<String>) -> Result<Regex, Error> {
    let re = args.next().unwrap_or_else(String::new);
    Regex::new(&format!(r#"(?i)({})"#, re))
}

pub(crate) fn extract_digits(s: &str) -> Result<u32, ParseIntError> {
    let output: String = s.chars().filter(|c| c.is_numeric()).collect();
    output.parse::<u32>()
}
pub(crate) fn error(message: &str) {
    execute!(
        stdout(),
        SetForegroundColor(Color::Red),
        Print(format!("\n{}", message)),
        ResetColor
    )
    .unwrap()
}
pub(crate) fn exit_error(message: &str) -> ! {
    error(message);
    exit(-1)
}
