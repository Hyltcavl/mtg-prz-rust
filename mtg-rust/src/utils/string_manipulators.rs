use chrono::{DateTime, Local};

/// Converts a DateTime object to a string formatted according to the provided format.
/// Defaults to the current date and time if the input is None and the default format is "%d_%m_%Y-%H:%M".
pub fn date_time_as_string(dt: Option<DateTime<Local>>, format: Option<&str>) -> String {
    dt.unwrap_or(Local::now())
        .format(format.unwrap_or("%d_%m_%Y-%H:%M"))
        .to_string()
}
pub fn clean_word(word: &str) -> String {
    word.to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || c == &' ')
        .collect::<String>()
        .replace("Æ", "ae")
        .trim()
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
