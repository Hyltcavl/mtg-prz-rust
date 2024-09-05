use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::{
    fs::{self, OpenOptions},
    path::Path,
};

use serde::Serialize;
use std::io::{self, BufWriter};

use chrono::NaiveDateTime;
use serde::de::DeserializeOwned;

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

pub fn save_to_json_file<T: Serialize>(path: &str, data: &T) -> io::Result<()> {
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

pub fn load_from_json_file<T: DeserializeOwned>(filename: &str) -> io::Result<T> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let data = serde_json::from_reader(reader)?;
    Ok(data)
}

#[cfg(test)]
mod tests {

    use std::collections::HashMap;

    use crate::{
        cards::card::{CardName, VendorCard},
        test::helpers::reaper_king_vendor_card_cheap,
    };

    use env_logger;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    use super::*;

    #[test]
    fn test_save_file() {
        init(); // Initialize logger

        let mut vendor_cards = Vec::new();
        vendor_cards.push(reaper_king_vendor_card_cheap());

        let mut map = HashMap::new();
        map.insert(reaper_king_vendor_card_cheap().name, vendor_cards);

        // Save to JSON file
        save_to_json_file::<HashMap<CardName, Vec<VendorCard>>>("cards.json", &map).unwrap();

        // Load from JSON file
        let loaded_map =
            load_from_json_file::<HashMap<CardName, Vec<VendorCard>>>("cards.json").unwrap();

        assert_eq!(loaded_map, map);
        let _ = fs::remove_file("cards.json");
    }
}
