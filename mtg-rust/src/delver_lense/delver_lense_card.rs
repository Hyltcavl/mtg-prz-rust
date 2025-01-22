use csv::Reader;
use serde::{Deserialize, Serialize};
use std::{error::Error, fs::File};

// use crate::cards::card::{CardName, SetName};

// #[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
// pub struct DelverLenseCard {
//     pub name: CardName,
//     pub foil: bool,
//     pub edition: SetName,
//     pub price: i32,
// }

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct DelverLenseCardRaw {
    pub Name: String,
    pub Foil: String,
    pub Edition: String,
    pub Price: String,
}

// Function to read a CSV file and parse it into a vector of MyData
fn read_csv(file_path: &str) -> Result<Vec<DelverLenseCardRaw>, Box<dyn Error>> {
    // Open the CSV file
    let file = File::open(file_path)?;

    // Create a CSV reader
    let mut rdr = Reader::from_reader(file);

    // Deserialize each record into a MyData object
    let mut results = Vec::new();
    for result in rdr.deserialize() {
        let record: DelverLenseCardRaw = result?; // Deserialize the row
        results.push(record);
    }

    Ok(results)
}

fn main() -> Result<(), Box<dyn Error>> {
    // Path to the CSV file
    let file_path = "/workspaces/mtg-prz-rust/mtg-rust/src/delver_lense/delver_lense_list.csv";

    // Read the CSV file and parse it into a vector of MyData
    let data = read_csv(file_path)?;

    // Print the parsed data
    for entry in data {
        println!("{:?}", entry);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    // use env_logger;

    // fn init() {
    //     let _ = env_logger::builder().is_test(true).try_init();
    // }

    #[test]
    fn test_reading_delver_lense_cards() {
        // init(); // Initialize logger
        // init();
        assert!(main().is_ok());
    }
}
