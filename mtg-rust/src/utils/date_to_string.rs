use chrono::{DateTime, Local};

/// Converts a DateTime object to a string formatted according to the provided format.
/// Defaults to the current date and time if the input is None and the default format is "%d_%m_%Y-%H:%M".
pub fn date_time_as_string(dt: Option<DateTime<Local>>, format: Option<&str>) -> String {
    dt.unwrap_or(Local::now())
        .format(format.unwrap_or("%d_%m_%Y-%H:%M"))
        .to_string()
}
