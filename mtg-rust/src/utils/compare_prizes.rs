// // use scraper::CaseSensitivity;
// use serde::{Deserialize, Serialize};

// use std::{collections::HashMap, error::Error};

// use crate::{dl::card_parser::VendorCard, scryfall::scryfall_mcm_cards::ScryfallCard};

// use super::{
//     constants::Vendor,
//     price_checker::{get_live_card_prices, MtgStocksCard},
// };

// // #[derive(Debug, Deserialize, Serialize)]
// // struct CachedMtgStockCards {
// //     pub cards: HashMap<String, Vec<MtgStocksCard>>,
// // }

// #[derive(Debug, Deserialize, Serialize)]
// struct CurrencyRate {
//     amount: f64,
//     base: String,
//     date: String,
//     rates: Rates,
// }

// #[derive(Debug, Deserialize, Serialize)]
// struct Rates {
//     sek: f64,
// }

// #[derive(Debug, PartialEq, Clone, Serialize)]
// pub struct CardPrice {
//     name: String,
//     vendor: Vendor,
//     set: String,
//     foil: bool,
//     vendor_price: f64,
//     same_set_price_mcm: Option<f64>,
//     cheapest_set_price_mcm: Option<f64>,
//     price_difference: Option<f64>,
//     price_diff_percentage: Option<f64>,
// }

// // https://www.frankfurter.app/docs/
// // https://api.frankfurter.app/latest?to=SEK,EUR
// async fn get_currency_rate_eur_to_sec(base_url: &str) -> Result<f64, Box<dyn Error>> {
//     let resp = reqwest::get(&format!("{}/latest?to=SEK", base_url))
//         .await?
//         .text()
//         .await?;
//     let currency_rate: CurrencyRate = serde_json::from_str(&resp)?;
//     Ok(currency_rate.rates.sek)
// }

// fn get_same_set_scryfall_card<'a>(
//     scryfall_cards: &'a Vec<ScryfallCard>,
//     vendor_card: &'a VendorCard,
// ) -> Option<&'a ScryfallCard> {
//     scryfall_cards
//         .iter()
//         .find(|scryfall_card| scryfall_card.set == vendor_card.set)
// }

// fn get_cheapest_scryfall_card<'a>(scryfall_cards: &'a Vec<ScryfallCard>) -> &'a ScryfallCard {
//     scryfall_cards
//         .iter()
//         .min_by(|scryfall_card_1, scryfall_card_2| {
//             match (scryfall_card_1.prices.eur, scryfall_card_2.prices.eur) {
//                 (Some(price1), Some(price2)) => price1.partial_cmp(&price2).unwrap(),
//                 (None, Some(_)) => std::cmp::Ordering::Greater,
//                 (Some(_), None) => std::cmp::Ordering::Less,
//                 (None, None) => std::cmp::Ordering::Equal,
//             }
//         })
//         .unwrap()
// }

// fn get_cheapest_foil_scryfall_card<'a>(scryfall_cards: &'a Vec<ScryfallCard>) -> &'a ScryfallCard {
//     scryfall_cards
//         .iter()
//         .min_by(|scryfall_card_1, scryfall_card_2| {
//             match (
//                 scryfall_card_1.prices.eur_foil,
//                 scryfall_card_2.prices.eur_foil,
//             ) {
//                 (Some(price1), Some(price2)) => price1.partial_cmp(&price2).unwrap(),
//                 (None, Some(_)) => std::cmp::Ordering::Greater,
//                 (Some(_), None) => std::cmp::Ordering::Less,
//                 (None, None) => std::cmp::Ordering::Equal,
//             }
//         })
//         .unwrap()
// }
// pub async fn compare_prices(
//     vendor_card_list: HashMap<String, Vec<VendorCard>>,
//     scryfall_card_map: HashMap<String, Vec<ScryfallCard>>,
//     base_url: &str,
// ) -> Vec<CardPrice> {
//     let mut cache: HashMap<String, Vec<MtgStocksCard>> = HashMap::new();

//     let start_time = chrono::prelude::Local::now();

//     let currency_rate_eur_to_sek = get_currency_rate_eur_to_sec(base_url).await.unwrap_or(11.5);
//     let mut card_prices = Vec::new();

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

//     card_prices
// }

// // Ta listan av vendor kort

// // Se vad billigaste kortet i foil och ikke foil är, oavsett set bland vendor korten

// // Jämför dem med samma set kort och billigaste kort set i scryfall

// // returnera ett card price som är

// /*
// pub struct ComparedCardPrices {
//     name: String,
//     vendor: Vendor,
//     prices: [CardPrice

//     ]
//     }
// pub struct CardPrice {
//         set: String,
//         foil: bool,
//         vendor_price_sek: f64,
//         same_set_price_mcm_sek: Option<f64>,
//         cheapest_set_price_mcm_sek: Option<f64>,
//         price_difference: Option<f64>,
//         price_diff_percentage: Option<f64>,
//     }
// */

// async fn process_cards(
//     vendor_cards: &Vec<VendorCard>,
//     scryfall_card_map: &HashMap<String, Vec<ScryfallCard>>,
//     currency_rate_eur_to_sek: f64,
//     cache: &mut HashMap<String, Vec<MtgStocksCard>>,
// ) -> Result<CardPrice, Box<dyn std::error::Error>> {
//     log::debug!(
//         "Comparing prices for card: '{}', set: '{}', foil: '{}'",
//         vendor_card.name,
//         vendor_card.set,
//         vendor_card.foil
//     );

//     let scryfall_cards = scryfall_card_map.get(&vendor_card.name).ok_or_else(|| {
//         format!(
//             "Unable to find card: '{}' among scryfall cards",
//             vendor_card.name
//         )
//     })?;

//     let same_set_card =
//         get_same_set_scryfall_card(scryfall_cards, vendor_card).ok_or_else(|| {
//             format!(
//                 "Unable to find a card from the same set for '{}'",
//                 vendor_card.name
//             )
//         })?;

//     let cheapest_card = get_cheapest_scryfall_card(scryfall_cards);
//     let cheapest_card_foil = get_cheapest_foil_scryfall_card(scryfall_cards);

//     let cheapest_price = cheapest_card.prices.eur.unwrap_or({
//         let possible_cache = cache.get(&cheapest_card.name);

//         let mtg_stock_prices = match possible_cache {
//             Some(prices) => prices,
//             None => &get_live_card_prices(&cheapest_card.name).await?,
//         };

//         let stock_prices = mtg_stock_prices
//             .into_iter()
//             .min_by(|card_1, card_2| card_1.price.partial_cmp(&card_2.price).unwrap())
//             .map(|card| card.price)
//             .unwrap_or(1000.0); // Fallback price if no price is found

//         cache.insert(cheapest_card.name.clone(), mtg_stock_prices.to_vec());
//         stock_prices
//     });

//     let price_difference = vendor_card.price as f64 - (cheapest_price * currency_rate_eur_to_sek);

//     let same_set_price_mcm = if vendor_card.foil {
//         same_set_card.prices.eur_foil.unwrap_or(1000.0) * currency_rate_eur_to_sek
//     } else {
//         same_set_card.prices.eur.unwrap_or(cheapest_price) * currency_rate_eur_to_sek
//     };

//     let cheapest_set_price_mcm = if vendor_card.foil {
//         cheapest_card_foil.prices.eur_foil.unwrap_or(1000.0) * currency_rate_eur_to_sek
//     } else {
//         cheapest_price * currency_rate_eur_to_sek
//     };

//     Ok(CardPrice {
//         name: vendor_card.name.to_owned(),
//         vendor: vendor_card.vendor.to_owned(),
//         set: vendor_card.set.to_owned(),
//         foil: vendor_card.foil,
//         vendor_price: vendor_card.price as f64,
//         same_set_price_mcm: Some(same_set_price_mcm),
//         cheapest_set_price_mcm: Some(cheapest_set_price_mcm),
//         price_difference: Some(price_difference),
//         price_diff_percentage: Some(
//             ((vendor_card.price as f64 - cheapest_price * currency_rate_eur_to_sek)
//                 / vendor_card.price as f64)
//                 * 100.0,
//         ),
//     })
// }

// #[cfg(test)]
// mod tests {

//     use crate::scryfall::scryfall_mcm_cards::Prices;

//     use super::*;

//     #[tokio::test]
//     async fn test_compare_prices() {
//         let response =
//             "{\"amount\":1.0,\"base\":\"EUR\",\"date\":\"2024-08-30\",\"rates\":{\"SEK\":11.3355}}";

//         let mut server = std::thread::spawn(|| mockito::Server::new())
//             .join()
//             .unwrap();
//         let url = server.url();
//         // Create a mock
//         let mock = server
//             .mock("GET", "/latest?to=SEK")
//             .with_status(200)
//             .with_header("content-type", "application/json")
//             .with_header("x-api-key", "1234")
//             .with_body(response)
//             .create();

//         let brainstorm = VendorCard {
//             name: "brainstorm".to_string(),
//             vendor: Vendor::Dragonslair,
//             set: "ice age".to_string(),
//             foil: false,
//             image_url: "www.google.com".to_string(),
//             extended_art: false,
//             prerelease: false,
//             showcase: false,
//             price: 10,
//             trade_in_price: 5,
//             current_stock: 2,
//             max_stock: 3,
//         };

//         let scryfall_brainstorm = ScryfallCard {
//             name: "brainstorm".to_string(),
//             set: "ice age".to_string(),
//             image_url: "www.google.com".to_string(),
//             prices: Prices {
//                 eur: Some(1.0),
//                 eur_foil: Some(2.0),
//             },
//         };

//         let scryfall_brainstorm_modern = ScryfallCard {
//             name: "brainstorm".to_string(),
//             set: "modern masters".to_string(),
//             image_url: "www.google.com".to_string(),
//             prices: Prices {
//                 eur: Some(0.5),
//                 eur_foil: Some(1.0),
//             },
//         };

//         let mut scryfall_cards = HashMap::new();
//         scryfall_cards.insert(
//             "brainstorm".to_string(),
//             [scryfall_brainstorm, scryfall_brainstorm_modern].to_vec(),
//         );

//         let vendor_card_list = vec![brainstorm];

//         let result = compare_prices(vendor_card_list, scryfall_cards, &url).await;
//         mock.assert();
//         assert_eq!(result.len(), 1);

//         let expected_card_price = CardPrice {
//             name: "brainstorm".to_string(),
//             vendor: Vendor::Dragonslair,
//             set: "ice age".to_string(),
//             foil: false,
//             vendor_price: 10.0,
//             same_set_price_mcm: Some(11.5),
//             cheapest_set_price_mcm: Some(5.75),
//             price_difference: Some(4.25),
//             price_diff_percentage: Some(42.5),
//         };

//         assert_eq!(result[0], expected_card_price);
//     }
// }
