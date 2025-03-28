use chrono::NaiveDateTime;
use log::{error, info};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fs::File;
use std::io::BufReader;
use std::io::{self, BufWriter, Write};
use std::path::PathBuf;
use std::{
    fs::{self, OpenOptions},
    path::Path,
};

pub fn load_from_json_file<T: DeserializeOwned>(filename: &str) -> io::Result<T> {
    info!("Loading from file: {}", filename);
    let file = File::open(filename).map_err(|e| {
        error!("Failed to open file: {}", e);
        e
    })?;
    let reader = BufReader::new(file);
    let data = serde_json::from_reader(reader).map_err(|e| {
        error!("Failed to read from file: {}", e);
        e
    })?;
    Ok(data)
}

pub async fn download_and_save_file(
    url: &str,
    path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let response = client.get(url).send().await?.bytes().await?;
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)?;
    }
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(path)?;
    let mut writer = BufWriter::new(file);
    writer.write_all(&response)?;
    Ok(())
}

pub fn save_to_file<T: Serialize>(path: &str, data: &T) -> io::Result<()> {
    info!(
        "Saving data type: {} to file: {} with",
        std::any::type_name::<T>(),
        path
    );
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)?;
    }

    // Open the file in write mode, creating it if it doesn't exist
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(path)?;
    // let file = File::create(filename)?;

    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, data)?;
    Ok(())
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
                info!("Found file: {}", file_path.display());
                file_path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .and_then(|name| {
                        name.split(prefix)
                            .nth(1)
                            .and_then(|date_time_str| {
                                NaiveDateTime::parse_from_str(
                                    &date_time_str.replace(".json", ""),
                                    "%d_%m_%Y-%H-%M",
                                )
                                .ok()
                            })
                            .map(|date_time| (file_path.clone(), date_time))
                    })
            } else {
                error!("Failed to read file: {}", file_path.display());
                None
            }
        })
        .max_by_key(|&(_, date_time)| date_time)
        .map(|(file_path, _)| file_path);

    newest_file.ok_or_else(|| "No valid files found".into())
}
