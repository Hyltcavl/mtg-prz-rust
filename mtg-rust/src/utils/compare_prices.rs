use reqwest::Client;
use std::{collections::HashMap, error::Error};

use serde::{Deserialize, Serialize};

use crate::{
    cards::card::{CardName, ScryfallCard, Vendor, VendorCard},
    config::CONFIG,
    utils::mtg_stock_price_checker::MtgPriceFetcher,
};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ComparedCard {
    pub name: String,
    pub foil: bool,
    pub vendor: Vendor,
    pub cheapest_set_price_mcm_sek: i32,
    // Positive here is a cheaper vendor card then the MCM. The amount is the difference in SEK
    pub price_difference_to_cheapest_vendor_card: i32,
    pub vendor_cards: Vec<VendorCard>,
}

#[derive(Debug, Deserialize, Serialize)]
struct CurrencyRate {
    amount: f64,
    base: String,
    date: String,
    rates: Rates,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Serialize)]
struct Rates {
    SEK: f64,
}

// https://www.frankfurter.app/docs/
// https://api.frankfurter.app/latest?to=SEK,EUR
async fn get_currency_rate_eur_to_sec(base_url: &str) -> Result<f64, Box<dyn Error>> {
    let resp = reqwest::get(&format!("{}/latest?to=SEK", base_url))
        .await?
        .text()
        .await?;
    let currency_rate: CurrencyRate = serde_json::from_str(&resp)?;
    Ok(currency_rate.rates.SEK)
}

fn get_cheapest_foil_scryfall_card<'a>(
    scryfall_cards: &'a Vec<ScryfallCard>,
    card_name: &str,
) -> Result<f64, Box<dyn std::error::Error>> {
    let cheapest_scryfall_card =
        scryfall_cards
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

    cheapest_scryfall_card
        .ok_or_else(|| {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("No card found for '{}'", card_name),
            )) as Box<dyn std::error::Error>
        })?
        .prices
        .eur_foil
        .ok_or_else(|| {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("No price found for card '{}'", card_name),
            )) as Box<dyn std::error::Error>
        })
}

fn get_cheapest_scryfall_card<'a>(
    scryfall_cards: &'a Vec<ScryfallCard>,
    card_name: &str,
) -> Result<f64, Box<dyn std::error::Error>> {
    let cheapest_scryfall_card =
        scryfall_cards
            .iter()
            .min_by(|scryfall_card_1, scryfall_card_2| {
                match (scryfall_card_1.prices.eur, scryfall_card_2.prices.eur) {
                    (Some(price1), Some(price2)) => price1.partial_cmp(&price2).unwrap(),
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, None) => std::cmp::Ordering::Equal,
                }
            });

    cheapest_scryfall_card
        .ok_or_else(|| {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("No card found for '{}'", card_name),
            )) as Box<dyn std::error::Error>
        })?
        .prices
        .eur
        .ok_or_else(|| {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("No price found for card '{}'", card_name),
            )) as Box<dyn std::error::Error>
        })
}

pub async fn compare_foil_card_price(
    vendor_cards: Vec<VendorCard>,
    scryfall_cards: &HashMap<CardName, Vec<ScryfallCard>>,
    currency_rate_eur_to_sek: f64,
    fetcher: &MtgPriceFetcher,
) -> Result<ComparedCard, Box<dyn std::error::Error>> {
    //-Get the cheapest vendor card
    let lowest_price_vendor_card = vendor_cards
        .clone()
        .into_iter()
        .min_by(|card_1, card_2| card_1.price.partial_cmp(&card_2.price).unwrap())
        .ok_or_else(|| {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No vendor cards available to find the lowest price",
            ))
        })?;

    let card_name = &lowest_price_vendor_card.name;
    //-Get the scryfall card list
    let scryfall_cards = scryfall_cards.get(card_name);

    let scryfall_cards = scryfall_cards.ok_or_else(|| {
        Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!(
                "No scryfall cards for card with name '{}'",
                &card_name.almost_raw
            ),
        ))
    })?;

    //-Get the price of the cheapest scryfall card or get the price from MtgStocks
    let cheapest_mcm_card_foil_price =
        match get_cheapest_foil_scryfall_card(scryfall_cards, &card_name.almost_raw) {
            Ok(price) => price,
            Err(e) => {
                log::error!(
                    "Error finding cheapest foil scryfall card for {}: {}",
                    card_name.almost_raw,
                    e
                );

                if CONFIG.external_price_check {
                    return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!(
                            "Won't fetch price for '{}', external price check is disabled",
                            card_name.almost_raw
                        ),
                    )));
                }

                // No cache checking
                let mtg_stock_prices = fetcher
                    .get_live_card_prices(&card_name.almost_raw, "https://api.mtgstocks.com")
                    .await
                    .map_err(|e| {
                        Box::new(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!(
                                "Error fetching live prices for '{}' with error: {}",
                                card_name.almost_raw, e
                            ),
                        ))
                    })?;

                mtg_stock_prices
                    .into_iter()
                    .min_by(|card_1, card_2| card_1.price.partial_cmp(&card_2.price).unwrap())
                    .map(|card| card.price)
                    .unwrap()
            }
        };

    //-Calculate the price difference
    let cheapest_mcm_price_sek = cheapest_mcm_card_foil_price * currency_rate_eur_to_sek;
    Ok(ComparedCard {
        name: lowest_price_vendor_card.name.almost_raw.to_owned(),
        foil: true,
        vendor: lowest_price_vendor_card.vendor.to_owned(),
        cheapest_set_price_mcm_sek: cheapest_mcm_price_sek.ceil() as i32,
        price_difference_to_cheapest_vendor_card: cheapest_mcm_price_sek.ceil() as i32
            - lowest_price_vendor_card.price,
        vendor_cards,
    })
}

pub async fn compare_card_price(
    vendor_cards: Vec<VendorCard>,
    scryfall_cards: &HashMap<CardName, Vec<ScryfallCard>>,
    currency_rate_eur_to_sek: f64,
    fetcher: &MtgPriceFetcher,
) -> Result<ComparedCard, Box<dyn std::error::Error>> {
    //-Get the cheapest vendor card
    let lowest_price_vendor_card = vendor_cards
        .clone()
        .into_iter()
        .min_by(|card_1, card_2| card_1.price.partial_cmp(&card_2.price).unwrap())
        .ok_or_else(|| {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No vendor cards available to find the lowest priced one",
            ))
        })?;

    let card_name = &lowest_price_vendor_card.name;
    //-Get the scryfall card list
    let scryfall_cards = scryfall_cards.get(card_name);

    let scryfall_cards = scryfall_cards.ok_or_else(|| {
        Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("No scryfall cards found for '{}'", card_name.almost_raw),
        ))
    })?;

    //-Get the price of the cheapest scryfall card or get the price from MtgStocks
    let cheapest_mcm_card_price =
        match get_cheapest_scryfall_card(scryfall_cards, &card_name.almost_raw) {
            Ok(card_price) => card_price,
            Err(e) => {
                log::error!(
                    "Error finding cheapest scryfall card '{}' because of: {}",
                    card_name.almost_raw,
                    e
                );

                if CONFIG.external_price_check {
                    return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!(
                            "Won't fetch price for '{}', external price check is disabled",
                            card_name.almost_raw
                        ),
                    )));
                }

                let mtg_stock_prices = fetcher
                    .get_live_card_prices(&card_name.almost_raw, "https://api.mtgstocks.com/")
                    .await
                    .map_err(|e| {
                        Box::new(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!(
                                "Error fetching live prices for '{}' because of: {}",
                                card_name.almost_raw, e
                            ),
                        ))
                    })?;

                mtg_stock_prices
                    .into_iter()
                    .min_by(|card_1, card_2| card_1.price.partial_cmp(&card_2.price).unwrap())
                    .map(|card| card.price)
                    .unwrap()
            }
        };

    //-Calculate the price difference
    let cheapest_price_sek = cheapest_mcm_card_price * currency_rate_eur_to_sek;
    let price_diff = cheapest_price_sek.ceil() as i32 - lowest_price_vendor_card.price;
    Ok(ComparedCard {
        name: lowest_price_vendor_card.name.almost_raw.to_owned(),
        foil: false,
        vendor: lowest_price_vendor_card.vendor.to_owned(),
        cheapest_set_price_mcm_sek: cheapest_price_sek.ceil() as i32,
        price_difference_to_cheapest_vendor_card: price_diff,
        vendor_cards,
    })
}

pub async fn compare_prices(
    vendor_card_list: HashMap<CardName, Vec<VendorCard>>,
    scryfall_card_map: HashMap<CardName, Vec<ScryfallCard>>,
    base_url: &str,
) -> Vec<ComparedCard> {
    log::info!("Comparing prices...");
    let start_time = chrono::prelude::Local::now();
    let client = Client::new();
    let fetcher = MtgPriceFetcher::new(client);

    let mut compared_cards = Vec::new();
    let currency_rate_eur_to_sek = get_currency_rate_eur_to_sec(base_url).await.unwrap_or(11.5);

    for (card_name, vendor_card_list) in vendor_card_list {
        log::debug!("Comparing prices for '{}'", card_name.almost_raw);

        // Separate foil and non-foil cards
        let (foil, non_foil): (Vec<VendorCard>, Vec<VendorCard>) =
            vendor_card_list.into_iter().partition(|card| card.foil);

        if foil.len() > 0 {
            let compare_foil_card_price = compare_foil_card_price(
                foil,
                &scryfall_card_map,
                currency_rate_eur_to_sek,
                &fetcher,
            )
            .await;

            match compare_foil_card_price {
                Ok(card_price) => {
                    compared_cards.push(card_price);
                }
                Err(e) => log::debug!(
                    "Unable to get price for FOIL version of '{}' because of: {}",
                    card_name.almost_raw,
                    e
                ),
            }
        } else {
            log::debug!(
                "No foil version of '{}' found in vendor cards",
                card_name.almost_raw
            );
        }

        if non_foil.len() > 0 {
            let compare_card_price = compare_card_price(
                non_foil,
                &scryfall_card_map,
                currency_rate_eur_to_sek,
                &fetcher,
            )
            .await;

            match compare_card_price {
                Ok(card_price) => {
                    compared_cards.push(card_price);
                }
                Err(e) => log::debug!(
                    "Unable to get price for NON foil version of '{}' because of: {}",
                    card_name.almost_raw,
                    e
                ),
            }
        } else {
            log::debug!(
                "No non-foil version of '{}' found in vendor cards",
                card_name.almost_raw
            );
        }
    }
    let end_time = chrono::prelude::Local::now();

    log::info!(
        "Compared prices started at: {}. Finished at: {}. Took: {}, ComparedPrices list size: {}",
        start_time,
        end_time,
        (end_time - start_time).num_seconds(),
        compared_cards.len()
    );

    compared_cards
}

#[cfg(test)]
mod tests {

    use std::collections::HashMap;

    use crate::{
        cards::card::Vendor,
        test::helpers::{
            lifecraft_c_name, lifecraft_c_scryfall_card, lifecraft_c_vendor_card,
            reaper_king_card_name, reaper_king_scryfall_card_cheap,
            reaper_king_scryfall_card_expensive, reaper_king_vendor_card_cheap,
            reaper_king_vendor_card_expensive, reaper_king_vendor_card_foil,
        },
    };

    use env_logger;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    use super::*;

    #[tokio::test]
    async fn test_compare_prices() {
        init(); // Initialize logger
        let currency = 11.3355;
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

        let vendor_card_list = HashMap::from([
            (
                reaper_king_card_name(),
                vec![
                    reaper_king_vendor_card_expensive(),
                    reaper_king_vendor_card_cheap(),
                    reaper_king_vendor_card_foil(),
                ],
            ),
            (lifecraft_c_name(), vec![lifecraft_c_vendor_card()]),
        ]);

        let scryfall_cards = HashMap::from([
            (
                reaper_king_card_name(),
                vec![
                    reaper_king_scryfall_card_expensive(),
                    reaper_king_scryfall_card_cheap(),
                ],
            ),
            (lifecraft_c_name(), vec![lifecraft_c_scryfall_card()]),
        ]);

        let result = compare_prices(vendor_card_list, scryfall_cards, &url).await;

        let cheapest_set_price_mcm_sek =
            (reaper_king_scryfall_card_cheap().prices.eur.unwrap() * currency).ceil() as i32;
        let price_diff = cheapest_set_price_mcm_sek - reaper_king_vendor_card_cheap().price;
        let non_foil_card = ComparedCard {
            name: reaper_king_vendor_card_cheap().name.almost_raw,
            foil: false,
            vendor: Vendor::Dragonslair,
            cheapest_set_price_mcm_sek: cheapest_set_price_mcm_sek,
            price_difference_to_cheapest_vendor_card: price_diff,
            vendor_cards: vec![
                reaper_king_vendor_card_expensive(),
                reaper_king_vendor_card_cheap(),
            ],
        };

        let cheapest_set_price_mcm_sek =
            (reaper_king_scryfall_card_cheap().prices.eur_foil.unwrap() * currency).ceil() as i32;
        let price_diff = cheapest_set_price_mcm_sek - reaper_king_vendor_card_foil().price;
        let foil_card = ComparedCard {
            name: reaper_king_vendor_card_foil().name.almost_raw,
            foil: true,
            vendor: Vendor::Dragonslair,
            cheapest_set_price_mcm_sek: cheapest_set_price_mcm_sek,
            price_difference_to_cheapest_vendor_card: price_diff,
            vendor_cards: vec![reaper_king_vendor_card_foil()],
        };

        let cheapest_set_price_mcm_sek =
            (lifecraft_c_scryfall_card().prices.eur_foil.unwrap() * currency).ceil() as i32;
        let price_diff = cheapest_set_price_mcm_sek - lifecraft_c_vendor_card().price;
        let lifecraft = ComparedCard {
            name: lifecraft_c_vendor_card().name.almost_raw,
            foil: true,
            vendor: Vendor::Dragonslair,
            cheapest_set_price_mcm_sek: cheapest_set_price_mcm_sek,
            price_difference_to_cheapest_vendor_card: price_diff,
            vendor_cards: vec![lifecraft_c_vendor_card()],
        };

        mock.assert();
        assert_eq!(result.len(), 3);

        let mut sorted_result = result.clone();
        sorted_result.sort_by(|a, b| a.name.cmp(&b.name).then(a.foil.cmp(&b.foil)));

        assert_eq!(sorted_result[0], lifecraft);
        assert_eq!(sorted_result[1], non_foil_card);
        assert_eq!(sorted_result[2], foil_card);
    }
}
