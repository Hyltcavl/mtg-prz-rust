use std::{collections::HashMap, error::Error};

use serde::{Deserialize, Serialize};

use crate::{cards::card::{ScryfallCard, Vendor, VendorCard}, utils::price_checker::get_live_card_prices};

use super::price_checker::MtgStocksCard;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ComparedCard {
    name: String,
    foil: bool,
    vendor: Vendor,
    cheapest_set_price_mcm_sek: Option<f64>,
    price_difference_to_cheapest_vendor_card: Option<f64>,
    price_diff_to_cheapest_percentage_vendor_card: Option<f64>,
    vendor_cards: Vec<VendorCard>,
}

#[derive(Debug, Deserialize, Serialize)]
struct CurrencyRate {
    amount: f64,
    base: String,
    date: String,
    rates: Rates,
}

#[derive(Debug, Deserialize, Serialize)]
struct Rates {
    sek: f64,
}

// https://www.frankfurter.app/docs/
// https://api.frankfurter.app/latest?to=SEK,EUR
async fn get_currency_rate_eur_to_sec(base_url: &str) -> Result<f64, Box<dyn Error>> {
    let resp = reqwest::get(&format!("{}/latest?to=SEK", base_url))
        .await?
        .text()
        .await?;
    let currency_rate: CurrencyRate = serde_json::from_str(&resp)?;
    Ok(currency_rate.rates.sek)
}

fn get_cheapest_foil_scryfall_card<'a>(
    scryfall_cards: &'a Vec<ScryfallCard>,
) -> Result<&'a ScryfallCard, Box<dyn std::error::Error>> {
    let scryfall_cards = scryfall_cards
        .iter()
        .min_by(|scryfall_card_1, scryfall_card_2| {
            match (
                scryfall_card_1.prices.eur_foil,
                scryfall_card_2.prices.eur_foil,
            ) {
                (Some(price1), Some(price2)) => price1.partial_cmp(&price2).unwrap(),
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, None) => std::cmp::Ordering::Equal,
            }
        });
    match scryfall_cards {
        Some(card) => Ok(card),
        None => Err(format!("No foil card found for '{}'", card_name).into()),
    }
}

pub async fn compare_prices(
    vendor_card_list: HashMap<String, Vec<VendorCard>>,
    scryfall_card_map: HashMap<String, Vec<ScryfallCard>>,
    base_url: &str,
) -> Vec<ComparedCard> {
    let start_time = chrono::prelude::Local::now();
    let mut cache: HashMap<String, Vec<MtgStocksCard>> = HashMap::new();
    
    //     let start_time = chrono::prelude::Local::now();

    let currency_rate_eur_to_sek = get_currency_rate_eur_to_sec(base_url).await.unwrap_or(11.5);
    //     let mut card_prices = Vec::new();

    for (card_name, vendor_card_list) in vendor_card_list {
        // Separate foil and non-foil cards
        let (foil, non_foil): (Vec<VendorCard>, Vec<VendorCard>) =
            vendor_card_list.into_iter().partition(|card| card.foil);
        // Then for each (foil and non-foil)
        //-Get the cheapest vendor card
        let lowest_price_vendor_card = foil
            .into_iter()
            .min_by(|card_1, card_2| card_1.price.partial_cmp(&card_2.price).unwrap())
            .unwrap();

        //-Get the scryfall card list
        let scryfall_cards = scryfall_card_map
            .get(&card_name)
            .ok_or_else(|| format!("Unable to find card: '{}' among scryfall cards", &card_name));

        let scryfall_cards = match scryfall_cards {
            Ok(scryfall_cards) => scryfall_cards,
            Err(e) => {
                log::error!("Error finding scryfall cards for {}: {}", &card_name, e);
                continue;
            }
        };

        //-Get the cheapest scryfall card
        let cheapest_mcm_card_foil = match get_cheapest_foil_scryfall_card(scryfall_cards) {
            Ok(card) => card,
            Err(e) => {
                log::error!("Error finding cheapest foil scryfall card for {}: {}", card_name, e);
                
                let mtg_stock_prices = match cache.get(&card_name) {
                                Some(prices) => prices,
                                None => {
                                    let mtgstock = get_live_card_prices(&card_name).await;
                                    match mtgstock {
                                        Ok(prices) => &prices,
                                        Err(e) => {
                                            log::error!("Error fetching live prices for {}: {}", card_name, e);
                                            continue;
                                        }
                                }
                            }
                        };

                return mtg_stock_prices
                            .into_iter()
                            .min_by(|card_1, card_2| card_1.price.partial_cmp(&card_2.price).unwrap())
                            .map(|card| card.price)
                            .unwrap_or(1000.0); 

                continue;
            }
        }
        //-Calculate the price difference

        //
    }

    //     for (name, vendor_card_list) in vendor_card_list {
    //         let processed_card = process_cards(
    //             &vendor_card_list,
    //             &scryfall_card_map,
    //             currency_rate_eur_to_sek,
    //             &mut cache,
    //         )
    //         .await;

    //         match processed_card {
    //             Ok(card_price) => card_prices.push(card_price),
    //             Err(e) => log::error!("Error processing card {}: {}", vendor_card.name, e),
    //         }
    //     }

    //     let end_time = chrono::prelude::Local::now();

    //     log::info!(
    //         "Compared prices started at: {}. Finished at: {}. Took: {}",
    //         start_time,
    //         end_time,
    //         (end_time - start_time).num_seconds(),
    //     );
    Vec::new()
}

#[cfg(test)]
mod tests {

    use std::collections::HashMap;

    use crate::{
        cards::card::Vendor,
        test::helpers::{
            reaper_king_scryfall_card, reaper_king_scryfall_card_2, reaper_king_vendor_card,
            reaper_king_vendor_card_2, reaper_king_vendor_card_foil, REAPER_KING_STRING,
        },
    };

    use super::*;

    #[tokio::test]
    async fn test_compare_prices() {
        let response =
            "{\"amount\":1.0,\"base\":\"EUR\",\"date\":\"2024-08-30\",\"rates\":{\"SEK\":11.3355}}";

        let mut server = std::thread::spawn(|| mockito::Server::new())
            .join()
            .unwrap();
        let url = server.url();
        // Create a mock
        let mock = server
            .mock("GET", "/latest?to=SEK")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_header("x-api-key", "1234")
            .with_body(response)
            .create();

        let vendor_card_list = HashMap::from([(
            REAPER_KING_STRING.to_string(),
            vec![
                reaper_king_vendor_card(),
                reaper_king_vendor_card_2(),
                reaper_king_vendor_card_foil(),
            ],
        )]);

        let scryfall_cards = HashMap::from([(
            REAPER_KING_STRING.to_string(),
            vec![reaper_king_scryfall_card(), reaper_king_scryfall_card_2()],
        )]);

        let result = compare_prices(vendor_card_list, scryfall_cards, &url).await;

        let non_foil_card = ComparedCard {
            name: REAPER_KING_STRING.to_string(),
            foil: false,
            vendor: Vendor::Dragonslair,
            cheapest_set_price_mcm_sek: Some(11.3355),
            price_difference_to_cheapest_vendor_card: Some(11.3355 - 10.0),
            price_diff_to_cheapest_percentage_vendor_card: Some((11.3355 - 10.0) / 10.0 * 100.0),
            vendor_cards: vec![reaper_king_vendor_card(), reaper_king_vendor_card_2()],
        };

        let foil_card = ComparedCard {
            name: REAPER_KING_STRING.to_string(),
            foil: true,
            vendor: Vendor::Dragonslair,
            cheapest_set_price_mcm_sek: Some(11.3355),
            price_difference_to_cheapest_vendor_card: Some(11.3355 - 10.0),
            price_diff_to_cheapest_percentage_vendor_card: Some((11.3355 - 10.0) / 10.0 * 100.0),
            vendor_cards: vec![reaper_king_vendor_card_foil()],
        };
        mock.assert();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], non_foil_card);
        assert_eq!(result[0], foil_card);
    }
}
