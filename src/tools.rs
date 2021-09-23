use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
};
use lazy_static::lazy_static;
use regex::{Error, Regex};
use reqwest::blocking::Client;
use std::fmt::Display;
use std::{
    io::{stdin, stdout, Write},
    num::{ParseFloatError, ParseIntError},
    process::exit,
};

pub(crate) const CLIENT_ID: &str = "kimne78kx3ncx6brgo4mv6wki5h1ko";
lazy_static! {
    pub(crate) static ref CLIENT: Client = Client::new();
    static ref USERNAME_VALIDATE: Regex = Regex::new(r#"^[a-zA-Z0-9][\w]{3,24}$"#).unwrap();
}

pub(crate) fn clean_quotes<S: AsRef<str>>(str: S) -> String {
    str.as_ref()
        .trim_start_matches('"')
        .trim_end_matches('"')
        .to_string()
}

pub(crate) fn format_time_string<S: AsRef<str>>(seconds: S) -> Result<String, ParseFloatError> {
    let seconds: u32 = match seconds.as_ref().parse::<f32>() {
        Ok(seconds) => seconds as u32,
        Err(e) => return Err(e),
    };
    Ok(format_time(seconds))
}

/// Function to call println! on all `String`s in a Vec whilst emptying it
pub(crate) fn print_queue<S: AsRef<str>>(comment_queue: &mut Vec<S>) {
    for comment in comment_queue.iter() {
        println!("{}", comment.as_ref())
    }
    comment_queue.clear()
}

pub(crate) fn hex_to_rgb<S: AsRef<str>>(hex: S) -> Result<Color, ParseIntError> {
    let hex = hex.as_ref().trim_start_matches('#');
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
    stdout().flush().unwrap();
    stdin().read_line(&mut re).unwrap();
    re = String::from(re.trim_end_matches(&['\r', '\n'][..]));
    Regex::new(&format!(r"(?i)({})", re))
}

pub(crate) fn is_valid_username<S: AsRef<str>>(username: S) -> bool {
    USERNAME_VALIDATE.is_match(username.as_ref())
}

pub(crate) fn args_filter<S: AsRef<str> + Display, A: Iterator<Item = S>>(
    args: &mut A,
) -> Result<Regex, Error> {
    match args.next() {
        None => Regex::new(&String::new()),
        Some(re) => Regex::new(&format!(r#"(?i)({})"#, re)),
    }
}
pub(crate) fn extract_digits<S: AsRef<str>>(s: S) -> u32 {
    s.as_ref()
        .chars()
        .filter(|c| c.is_numeric())
        .collect::<String>()
        .parse()
        .unwrap()
}

pub(crate) fn error<S: Display>(message: S) {
    execute!(
        stdout(),
        SetForegroundColor(Color::Red),
        Print(format!("\n{}", message)),
        ResetColor
    )
    .unwrap()
}

pub(crate) fn exit_error<S: Display>(message: S) -> ! {
    error(message);
    exit(-1)
}
