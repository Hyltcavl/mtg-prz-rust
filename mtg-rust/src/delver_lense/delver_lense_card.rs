use csv::Reader;
use serde::{Deserialize, Serialize};
use std::{error::Error, fs::File, num::ParseFloatError};

use crate::cards::card::{CardName, PersonalCard, SetName};

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
// DONE Läsa filen
// DONE Göra den till egna objekt
// DONE Göra dem objekten till kort objekt
// Kolla vilka av mina kort objekt som finns på DL, där det finns mindre än 4, och skapa sedan objekt för dem korten med ett mcm pris och inbytespris.
fn main(file_path: &str) -> Result<Vec<PersonalCard>, Box<dyn Error>> {
    // Path to the CSV file

    // Read the CSV file and parse it into a vector of MyData
    let data = read_csv(file_path)?;

    let mut cards: Vec<DelverLenseCardRaw> = Vec::new();

    // Print the parsed data
    for entry in data {
        cards.push(entry);
    }

    let personalCards = convert_raw_card_to_personal_card(cards);

    Ok(personalCards)
}

fn convert_raw_card_to_personal_card(cards: Vec<DelverLenseCardRaw>) -> Vec<PersonalCard> {
    cards
        .iter()
        .map(|card| PersonalCard {
            name: CardName::new(card.Name.clone()).unwrap(),
            set: SetName::new(card.Edition.clone()).unwrap(),
            foil: match card.Foil.as_str() {
                "true" => true,
                "false" => false,
                _ => false,
            },
            price: convert_price_to_number(card.Price.clone()).unwrap(),
        })
        .collect::<Vec<PersonalCard>>()
}

fn convert_price_to_number(price_as_text: String) -> Result<f64, ParseFloatError> {
    let res = price_as_text.replace("\u{a0}€", "").replace(",", ".");
    res.parse()
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
        let card1 = DelverLenseCardRaw {
            Name: "Zagoth Triome".to_string(),
            Foil: "".to_string(),
            Edition: "Ikoria: Lair of Behemoths".to_string(),
            Price: "17,75\u{a0}€".to_string(),
        };
        let card2 = DelverLenseCardRaw {
            Name: "Verdant Catacombs".to_string(),
            Foil: "".to_string(),
            Edition: "Zendikar".to_string(),
            Price: "15,79\u{a0}€".to_string(),
        };
        let cards = vec![card1, card2];
        let expected_cards = convert_raw_card_to_personal_card(cards);
        let file_path = "/workspaces/mtg-prz-rust/mtg-rust/src/delver_lense/delver_lense_list.csv";
        let result = main(file_path);
        assert_eq!(result.unwrap(), expected_cards);
    }
}
