use csv::Reader;
use std::{error::Error, fs::File, num::ParseFloatError};

use crate::cards::{
    cardname::CardName, currency::Currency, delver_lense_card::DelverLenseCard,
    personalcard::PersonalCard, price::Price, rarity::Rarity, setname::SetName,
};

pub struct DelverLenseConverter {}

impl DelverLenseConverter {
    pub fn new() -> Self {
        DelverLenseConverter {}
    }

    /// Takes the path of the delver lense csv file as input
    pub fn get_delver_lense_cards_from_file(
        &self,
        file_path: &str,
    ) -> Result<Vec<PersonalCard>, Box<dyn Error>> {
        // Read the CSV file and parse it into a vector of MyData
        let delver_lense_cards = self.read_csv(file_path)?;

        Ok(self.convert_delver_lense_card_to_personal_card(delver_lense_cards))
    }

    // problem to soleve. Given that I have several of the same card from delverlense,
    // how do I create the PeronalCard object with a count?
    pub fn convert_delver_lense_card_to_personal_card(
        &self,
        cards: Vec<DelverLenseCard>,
    ) -> Vec<PersonalCard> {
        cards
            .iter()
            .map(|card| PersonalCard {
                name: CardName::new(card.Name.clone()).unwrap(),
                set: SetName::new(card.Edition.clone()).unwrap(),
                foil: !card.Foil.is_empty(),
                price: self
                    .convert_string_price_to_price(card.Price.clone())
                    .unwrap(),
                count: card.Quantity.parse().unwrap(),
                color: card.Color.as_str().parse().unwrap(),
                rarity: match card.Rarity.as_str() {
                    "C" => Rarity::Common,
                    "U" => Rarity::Uncommon,
                    "R" => Rarity::Rare,
                    "M" => Rarity::Mythic,
                    _ => Rarity::Common,
                },
            })
            .collect::<Vec<PersonalCard>>()
    }

    fn convert_string_price_to_price(
        &self,
        price_as_text: String,
    ) -> Result<Price, ParseFloatError> {
        let res = price_as_text.replace("\u{a0}€", "").replace(",", ".");
        res.parse::<f64>()
            .map(|amount| Price::new(amount, Currency::EUR))
    }

    // Function to read a CSV file and parse it into a vector of MyData
    fn read_csv(&self, file_path: &str) -> Result<Vec<DelverLenseCard>, Box<dyn Error>> {
        // Open the CSV file
        let file = File::open(file_path)?;

        // Create a CSV reader
        let mut rdr = Reader::from_reader(file);

        // Deserialize each record into a MyData object
        let mut results = Vec::new();
        for result in rdr.deserialize() {
            let record: DelverLenseCard = result?; // Deserialize the row
            results.push(record);
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn card_1() -> DelverLenseCard {
        DelverLenseCard {
            Name: "Reaper King".to_string(),
            Foil: "Foil".to_string(),
            Edition: "Shadowmoor".to_string(),
            Price: "200,00 €".to_string(),
            Quantity: "1".to_string(),
            Color: "wuberg".to_string(),
            Rarity: "R".to_string(),
        }
    }
    fn card_2() -> DelverLenseCard {
        DelverLenseCard {
            Name: "Reaper King".to_string(),
            Foil: "".to_string(),
            Edition: "Shadowmoor".to_string(),
            Price: "2,06 €".to_string(),
            Quantity: "1".to_string(),
            Color: "wuberg".to_string(),
            Rarity: "R".to_string(),
        }
    }
    fn card_3() -> DelverLenseCard {
        DelverLenseCard {
            Name: "Counterspell".to_string(),
            Foil: "Foil".to_string(),
            Edition: "Fourth Edition".to_string(),
            Price: "0,94 €".to_string(),
            Quantity: "2".to_string(),
            Color: "blue".to_string(),
            Rarity: "C".to_string(),
        }
    }
    fn card_4() -> DelverLenseCard {
        DelverLenseCard {
            Name: "Counterspell".to_string(),
            Foil: "".to_string(),
            Edition: "Ice Age".to_string(),
            Price: "1,24 €".to_string(),
            Quantity: "1".to_string(),
            Color: "blue".to_string(),
            Rarity: "C".to_string(),
        }
    }
    fn card_5() -> DelverLenseCard {
        DelverLenseCard {
            Name: "Counterspell".to_string(),
            Foil: "".to_string(),
            Edition: "Masters 25".to_string(),
            Price: "1,10 €".to_string(),
            Quantity: "1".to_string(),
            Color: "blue".to_string(),
            Rarity: "C".to_string(),
        }
    }

    #[test]
    fn test_reading_delver_lense_cards_and_converting_to_personal_card() {
        let cards = vec![card_1(), card_2(), card_3(), card_4(), card_5()];
        let delver_lense_converter = DelverLenseConverter::new();
        let expected_cards =
            delver_lense_converter.convert_delver_lense_card_to_personal_card(cards);
        let file_path = "src/test/list_of_cards_from_delver_lens.csv";
        let result = delver_lense_converter.get_delver_lense_cards_from_file(file_path);
        assert_eq!(result.unwrap(), expected_cards);
    }
}
