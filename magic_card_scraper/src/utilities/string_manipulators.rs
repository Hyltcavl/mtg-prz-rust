use chrono::{DateTime, Local};

pub fn date_time_as_string(dt: Option<DateTime<Local>>, format: Option<&str>) -> String {
    dt.unwrap_or(Local::now())
        .format(format.unwrap_or("%d_%m_%Y-%H-%M"))
        .to_string()
}

pub fn clean_string(input: &str) -> String {
    input
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .trim()
        .to_string()
}
