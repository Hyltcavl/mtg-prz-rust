use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::{
    fs::{self, OpenOptions},
    path::Path,
};

use chrono::NaiveDateTime;
use serde::de::DeserializeOwned;

pub fn write_to_file(path: &str, content: &str) -> std::io::Result<()> {
    // Create all parent directories if they don't exist
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)?;
    }

    // Open the file in write mode, creating it if it doesn't exist
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(path)?;

    // Write the content to the file
    writeln!(file, "{}", content)?;

    Ok(())
}

pub fn read_json_file<T>(file_path: &str) -> Result<T, Box<dyn std::error::Error>>
where
    T: DeserializeOwned,
{
    // Check if the file exists
    let path = Path::new(file_path);
    if !path.exists() {
        return Err(format!("File not found: {}", file_path).into());
    }

    // Read the file content
    let mut file = File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    // Parse the JSON content
    let data: T = serde_json::from_str(&content)?;

    Ok(data)
}

pub fn get_newest_file(
    folder_path: &str,
    prefix: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let path = Path::new(folder_path);

    if !path.is_dir() {
        return Err(format!("{} is not a directory", folder_path).into());
    }

    let newest_file = fs::read_dir(path)?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let file_path = entry.path();

            if file_path.is_file() {
                log::info!("Found file: {}", file_path.display());
                file_path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .and_then(|name| {
                        name.split(prefix)
                            .nth(1)
                            .and_then(|date_time_str| {
                                NaiveDateTime::parse_from_str(
                                    &date_time_str.replace(".json", ""),
                                    "%d_%m_%Y-%H:%M",
                                )
                                .ok()
                            })
                            .map(|date_time| (file_path.clone(), date_time))
                    })
            } else {
                log::error!("Failed to read file: {}", file_path.display());
                None
            }
        })
        .max_by_key(|&(_, date_time)| date_time)
        .map(|(file_path, _)| file_path);

    newest_file.ok_or_else(|| "No valid files found".into())
}
