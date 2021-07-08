pub fn clean_quotes(string: String) -> String {
    string.trim_start_matches("\"").trim_end_matches("\"").to_string()
}

pub(crate) fn format_time(seconds: String) -> String {
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