use csv::Reader;
use serde::{Deserialize, Serialize};
use std::{error::Error, fs::File, num::ParseFloatError};

use crate::{
    cards::card::{CardName, PersonalCard, SetName, Vendor, VendorCard},
    dl::{card_parser::fetch_and_parse, list_links::get_page_count},
};

use super::price::{Currency, Price};

// TODO
// DONE Add functionality to save the peronal cards not found in the saved vendor cards
// Chain all functions in the main function
// Clean the code
// Make a function to look for the personal cards at the vendor site directly
// Add Option the main main function to run the comparison and print out the reuslt

#[allow(non_snake_case)]
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct DelverLenseCard {
    pub Name: String,
    pub Foil: String,
    pub Edition: String,
    pub Price: String,
    pub Quantity: String,
}

#[derive(Debug, PartialEq, Clone)]
struct TradeableCard {
    name: CardName,
    set: SetName,
    foil: bool,
    tradeable_vendor: Vendor,
    trade_in_price: Price,
    mcm_price: Price,
    cards_to_trade: i8,
    card_ammount_requested_by_vendor: i8,
}

// Function to read a CSV file and parse it into a vector of MyData
fn read_csv(file_path: &str) -> Result<Vec<DelverLenseCard>, Box<dyn Error>> {
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

fn get_tradable_and_leftover_cards(
    personal_cards: Vec<PersonalCard>,
    vendor_cards: Vec<VendorCard>,
) -> (Vec<TradeableCard>, Vec<PersonalCard>) {
    let mut leftover_cards = vec![];

    let t_cards = personal_cards
        .iter()
        .filter_map(|p_card| {
            let card = vendor_cards.iter().find(|v_card| {
                v_card.name == p_card.name && v_card.foil == p_card.foil && v_card.set == p_card.set
            });

            match card {
                Some(v_card) => {
                    if v_card.current_stock < v_card.max_stock {
                        Some(TradeableCard {
                            name: v_card.name.clone(),
                            set: v_card.set.clone(),
                            foil: p_card.foil,
                            tradeable_vendor: v_card.vendor.clone(),
                            trade_in_price: Price::new(v_card.trade_in_price.into(), Currency::SEK),
                            mcm_price: Price::new(p_card.price, Currency::SEK),
                            cards_to_trade: p_card.count.clone(),
                            card_ammount_requested_by_vendor: v_card.max_stock
                                - v_card.current_stock,
                        })
                    } else {
                        None
                    }
                }
                None => {
                    leftover_cards.push(p_card.clone());
                    None
                }
            }
        })
        .collect();

    (t_cards, leftover_cards)
}

fn main(file_path: &str) -> Result<Vec<PersonalCard>, Box<dyn Error>> {
    // Path to the CSV file

    // Read the CSV file and parse it into a vector of MyData
    let data = read_csv(file_path)?;

    let mut cards: Vec<DelverLenseCard> = Vec::new();

    // Print the parsed data
    for entry in data {
        cards.push(entry);
    }

    Ok(convert_raw_card_to_personal_card(cards))
}

// problem to soleve. Given that I have several of the same card from delverlense,
// how do I create the PeronalCard object with a count?
fn convert_raw_card_to_personal_card(cards: Vec<DelverLenseCard>) -> Vec<PersonalCard> {
    cards
        .iter()
        .map(|card| PersonalCard {
            name: CardName::new(card.Name.clone()).unwrap(),
            set: SetName::new(card.Edition.clone()).unwrap(),
            foil: match card.Foil.as_str() {
                "Foil" => true,
                "" => false,
                _ => false,
            },
            price: convert_price_to_number(card.Price.clone()).unwrap(),
            count: card.Quantity.parse().unwrap(),
        })
        .collect::<Vec<PersonalCard>>()
}

fn convert_price_to_number(price_as_text: String) -> Result<f64, ParseFloatError> {
    let res = price_as_text.replace("\u{a0}€", "").replace(",", ".");
    res.parse()
}
use urlencoding::encode;

async fn get_tradable_cards(
    file_path: &str,
    vendor_cards: Vec<VendorCard>,
) -> Result<Vec<TradeableCard>, Box<dyn Error>> {
    let personal_cards = main(file_path).unwrap();
    let (mut tradable_cards, leftover_personal_cards) =
        get_tradable_and_leftover_cards(personal_cards, vendor_cards);

    // Make function for checking the personal cards at the vendor site
    let url = "https://astraeus.dragonslair.se".to_owned();
    let mut vendor_cards_with_same_name_as_leftover: Vec<VendorCard> = vec![];
    for card in &leftover_personal_cards {
        let card_name_lowercase = card.name.almost_raw.to_lowercase();
        let card_name_encoded = encode(&card_name_lowercase);
        let request_url = format!(
            "{}/product/card-singles/magic/name:{}/{}",
            url, card_name_encoded, 0
        );
        let page_count = get_page_count(&request_url).await.unwrap_or(1);
        log::debug!("page_count for {} is {:?}", card_name_encoded, page_count);
        let urls = (1..=page_count)
            .map(|count| {
                format!(
                    "{}/product/card-singles/magic/name:{}/{}",
                    url, card_name_encoded, count
                )
            })
            .collect::<Vec<String>>();

        for url in urls {
            log::debug!("Fetching url: {}", url);
            let mut cards = fetch_and_parse(&url).await.unwrap();
            log::debug!("Fetched cards: {:?}", &cards);
            vendor_cards_with_same_name_as_leftover.append(&mut cards);
        }
    }

    let (mut more_tradable_cards, unwanted_cards) = get_tradable_and_leftover_cards(
        leftover_personal_cards,
        vendor_cards_with_same_name_as_leftover,
    );
    tradable_cards.append(&mut more_tradable_cards);

    log::debug!("tradable cards: {:?}", &tradable_cards);

    return Ok(tradable_cards);
}

#[cfg(test)]
mod tests {
    use crate::{
        delver_lense::price::Currency,
        test::helpers::{
            counterspell_forth_e, counterspell_ice_age, reaper_king_vendor_card_cheap,
            reaper_king_vendor_card_expensive, reaper_king_vendor_card_foil,
        },
    };

    use super::*;

    fn card_1() -> DelverLenseCard {
        DelverLenseCard {
            Name: "Reaper King".to_string(),
            Foil: "Foil".to_string(),
            Edition: "Shadowmoor".to_string(),
            Price: "200,00 €".to_string(),
            Quantity: "1".to_string(),
        }
    }
    fn card_2() -> DelverLenseCard {
        DelverLenseCard {
            Name: "Reaper King".to_string(),
            Foil: "".to_string(),
            Edition: "Shadowmoor".to_string(),
            Price: "2,06 €".to_string(),
            Quantity: "1".to_string(),
        }
    }
    fn card_3() -> DelverLenseCard {
        DelverLenseCard {
            Name: "Counterspell".to_string(),
            Foil: "Foil".to_string(),
            Edition: "Fourth Edition".to_string(),
            Price: "0,94 €".to_string(),
            Quantity: "2".to_string(),
        }
    }
    fn card_4() -> DelverLenseCard {
        DelverLenseCard {
            Name: "Counterspell".to_string(),
            Foil: "".to_string(),
            Edition: "Ice Age".to_string(),
            Price: "1,24 €".to_string(),
            Quantity: "1".to_string(),
        }
    }
    fn card_5() -> DelverLenseCard {
        DelverLenseCard {
            Name: "Counterspell".to_string(),
            Foil: "".to_string(),
            Edition: "Masters 25".to_string(),
            Price: "1,10 €".to_string(),
            Quantity: "1".to_string(),
        }
    }

    #[test]
    fn test_reading_delver_lense_cards() {
        let cards = vec![card_1(), card_2(), card_3(), card_4(), card_5()];
        let expected_cards = convert_raw_card_to_personal_card(cards);
        let file_path =
            "/workspaces/mtg-prz-rust/mtg-rust/src/delver_lense/Testlist_2025_Jan_27_17-02.csv";
        let result = main(file_path);
        assert_eq!(result.unwrap(), expected_cards);
    }

    #[test]
    fn get_tradable_cards_should_return_tradable_cards() {
        let card1 = reaper_king_vendor_card_expensive();
        let card2 = reaper_king_vendor_card_cheap();
        let card3 = reaper_king_vendor_card_foil();
        let card4 = counterspell_forth_e();
        let card5 = counterspell_ice_age();

        let vendor_cards = vec![card1, card2, card3, card4, card5];

        let raw_cards = vec![card_1(), card_2(), card_3(), card_4(), card_5()];
        let personal_cards = convert_raw_card_to_personal_card(raw_cards);

        let tradeable_card1 = TradeableCard {
            name: CardName::new("Reaper King".to_string()).unwrap(),
            set: SetName::new("Shadowmoor".to_string()).unwrap(),
            foil: true,
            tradeable_vendor: Vendor::Dragonslair,
            trade_in_price: Price::new(100.0, Currency::SEK),
            mcm_price: Price::new(200.0, Currency::SEK),
            cards_to_trade: 1,
            card_ammount_requested_by_vendor: 1,
        };
        let tradeable_card2 = TradeableCard {
            name: CardName::new("Reaper King".to_string()).unwrap(),
            set: SetName::new("Shadowmoor".to_string()).unwrap(),
            foil: false,
            tradeable_vendor: Vendor::Dragonslair,
            trade_in_price: Price::new(50.0, Currency::SEK),
            mcm_price: Price::new(2.06, Currency::SEK),
            cards_to_trade: 1,
            card_ammount_requested_by_vendor: 1,
        };

        let tradeable_card4 = TradeableCard {
            name: CardName::new("Counterspell".to_string()).unwrap(),
            set: SetName::new("Ice Age".to_string()).unwrap(),
            foil: false,
            tradeable_vendor: Vendor::Dragonslair,
            trade_in_price: Price::new(50.0, Currency::SEK),
            mcm_price: Price::new(1.24, Currency::SEK),
            cards_to_trade: 1,
            card_ammount_requested_by_vendor: 2,
        };

        let expected_cards = vec![tradeable_card1, tradeable_card2, tradeable_card4];
        let (result_v_cards, leftover_cards) =
            get_tradable_and_leftover_cards(personal_cards.clone(), vendor_cards);

        assert_eq!(result_v_cards, expected_cards);
        assert_eq!(
            leftover_cards,
            vec![personal_cards[2].clone(), personal_cards[4].clone()]
        );
    }

    #[test]
    fn should_get_tradable_cards_and_cards_not_found_() {
        let card5 = counterspell_ice_age();

        let vendor_cards = vec![card5];

        let raw_cards = vec![card_1(), card_4()];
        let personal_cards = convert_raw_card_to_personal_card(raw_cards);

        let tradeable_card1 = TradeableCard {
            name: CardName::new("Reaper King".to_string()).unwrap(),
            set: SetName::new("Shadowmoor".to_string()).unwrap(),
            foil: true,
            tradeable_vendor: Vendor::Dragonslair,
            trade_in_price: Price::new(100.0, Currency::SEK),
            mcm_price: Price::new(200.0, Currency::SEK),
            cards_to_trade: 1,
            card_ammount_requested_by_vendor: 1,
        };

        let tradeable_card4 = TradeableCard {
            name: CardName::new("Counterspell".to_string()).unwrap(),
            set: SetName::new("Ice Age".to_string()).unwrap(),
            foil: false,
            tradeable_vendor: Vendor::Dragonslair,
            trade_in_price: Price::new(50.0, Currency::SEK),
            mcm_price: Price::new(1.24, Currency::SEK),
            cards_to_trade: 1,
            card_ammount_requested_by_vendor: 2,
        };

        let expected_tradable_cards = vec![tradeable_card4];
        let expected_leftover_cards = vec![personal_cards[0].clone()];

        let (result_v_cards, leftover_cards) =
            get_tradable_and_leftover_cards(personal_cards, vendor_cards);

        assert_eq!(result_v_cards, expected_tradable_cards);
        assert_eq!(leftover_cards, expected_leftover_cards);
    }
}

#[cfg(test)]
mod tests2 {
    use super::*;
    use crate::{
        cards::card, dl::list_links::get_page_count,
        test::helpers::reaper_king_vendor_card_expensive,
    };
    use std::fs;
    use tokio;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[tokio::test]
    async fn test_fetch_card() {
        init();
        let file_path =
            "/workspaces/mtg-prz-rust/mtg-rust/src/delver_lense/Testlist_2025_Jan_27_17-02.csv";
        let tradable_cards = get_tradable_cards(file_path, vec![]).await;
        for card in tradable_cards.unwrap() {
            log::info!("Card: {:?}", card);
        }
        assert!(true);
    }
}
