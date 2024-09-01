use serde::{Deserialize, Serialize};

use std::{collections::HashMap, error::Error};

use crate::{dl::card_parser::VendorCard, scryfall::scryfall_mcm_cards::ScryfallCard};

use super::{constants::Vendor, price_checker::get_live_card_prices};

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

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct CardPrice {
    name: String,
    vendor: Vendor,
    set: String,
    foil: bool,
    vendor_price: f64,
    same_set_price_mcm: Option<f64>,
    cheapest_set_price_mcm: Option<f64>,
    price_difference: Option<f64>,
    price_diff_percentage: Option<f64>,
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

fn get_same_set_scryfall_card<'a>(
    scryfall_cards: &'a Vec<ScryfallCard>,
    vendor_card: &'a VendorCard,
) -> &'a ScryfallCard {
    let same_set_card = scryfall_cards
        .iter()
        .find(|scryfall_card| scryfall_card.set == vendor_card.set)
        .unwrap();
    same_set_card
}

fn get_cheapest_scryfall_card<'a>(scryfall_cards: &'a Vec<ScryfallCard>) -> &'a ScryfallCard {
    scryfall_cards
        .iter()
        .min_by(|scryfall_card_1, scryfall_card_2| {
            match (scryfall_card_1.prices.eur, scryfall_card_2.prices.eur) {
                (Some(price1), Some(price2)) => price1.partial_cmp(&price2).unwrap(),
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, None) => std::cmp::Ordering::Equal,
            }
        })
        .unwrap()
}

fn get_cheapest_foil_scryfall_card<'a>(scryfall_cards: &'a Vec<ScryfallCard>) -> &'a ScryfallCard {
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
        })
        .unwrap()
}

pub async fn compare_prices(
    vendor_card_list: Vec<VendorCard>,
    scryfall_card_map: HashMap<String, Vec<ScryfallCard>>,
    base_url: &str,
) -> Vec<CardPrice> {
    let currency_rate_eur_to_sek = get_currency_rate_eur_to_sec(base_url).await.unwrap_or(11.5);
    let mut card_prices = Vec::new();

    for vendor_card in vendor_card_list {
        let all_card_versions = scryfall_card_map.get(&vendor_card.name);
        let foil = vendor_card.foil;

        if let Some(scryfall_cards) = all_card_versions {
            let same_set_card = get_same_set_scryfall_card(scryfall_cards, &vendor_card);

            let cheapest_card = get_cheapest_scryfall_card(scryfall_cards);

            let cheapest_card_foil = get_cheapest_foil_scryfall_card(scryfall_cards);

            let cheapest_price_sek = cheapest_card.prices.eur.unwrap_or({
                get_live_card_prices(&cheapest_card.name)
                    .unwrap()
                    .into_iter()
                    .min_by(|card_1, card_2| card_1.price.partial_cmp(&card_2.price).unwrap())
                    .unwrap()
                    .price
            }) * currency_rate_eur_to_sek;

            let price_difference = vendor_card.price as f64 - cheapest_price_sek;

            let same_set_price_mcm = if foil {
                Some(same_set_card.prices.eur_foil.unwrap() * currency_rate_eur_to_sek)
            } else {
                Some(same_set_card.prices.eur.unwrap() * currency_rate_eur_to_sek)
            };

            let cheapest_set_price_mcm = if foil {
                Some(cheapest_card_foil.prices.eur_foil.unwrap() * currency_rate_eur_to_sek)
            } else {
                Some(cheapest_card.prices.eur.unwrap() * currency_rate_eur_to_sek)
            };

            let card_price = CardPrice {
                name: vendor_card.name.to_owned(),
                vendor: vendor_card.vendor.to_owned(),
                set: vendor_card.set.to_owned(),
                foil: vendor_card.foil.to_owned(),
                vendor_price: vendor_card.price.to_owned() as f64,
                same_set_price_mcm: same_set_price_mcm,
                cheapest_set_price_mcm: cheapest_set_price_mcm,
                price_difference: Some(price_difference),
                price_diff_percentage: Some(
                    ((vendor_card.price as f64 - cheapest_price_sek) / vendor_card.price as f64)
                        * 100.0,
                ),
            };
            card_prices.push(card_price);
        } else {
            log::info!(
                "Unable to find card: {} among scryfall cards",
                vendor_card.name
            );
        }
    }

    // vendor_card_list.iter().for_each(|vendor_card| {
    //     let all_card_versions = scryfall_card_map.get(&vendor_card.name);
    //     let foil = vendor_card.foil;

    //     if let Some(scryfall_cards) = all_card_versions {
    //         let same_set_card = get_same_set_scryfall_card(scryfall_cards, vendor_card);

    //         let cheapest_card = get_cheapest_scryfall_card(scryfall_cards);

    //         let cheapest_card_foil = get_cheapest_foil_scryfall_card(scryfall_cards);

    //         let cheapest_price_sek = cheapest_card.prices.eur.unwrap_or({
    //             get_live_card_prices(&cheapest_card.name)
    //                 .unwrap()
    //                 .into_iter()
    //                 .min_by(|card_1, card_2| card_1.price.partial_cmp(&card_2.price).unwrap())
    //                 .unwrap()
    //                 .price
    //         }) * currency_rate_eur_to_sek;

    //         let price_difference = vendor_card.price as f64 - cheapest_price_sek;

    //         let same_set_price_mcm = if foil {
    //             Some(same_set_card.prices.eur_foil.unwrap() * currency_rate_eur_to_sek)
    //         } else {
    //             Some(same_set_card.prices.eur.unwrap() * currency_rate_eur_to_sek)
    //         };

    //         let cheapest_set_price_mcm = if foil {
    //             Some(cheapest_card_foil.prices.eur_foil.unwrap() * currency_rate_eur_to_sek)
    //         } else {
    //             Some(cheapest_card.prices.eur.unwrap() * currency_rate_eur_to_sek)
    //         };

    //         let card_price = CardPrice {
    //             name: vendor_card.name.to_owned(),
    //             vendor: vendor_card.vendor.to_owned(),
    //             set: vendor_card.set.to_owned(),
    //             foil: vendor_card.foil.to_owned(),
    //             vendor_price: vendor_card.price.to_owned() as f64,
    //             same_set_price_mcm: same_set_price_mcm,
    //             cheapest_set_price_mcm: cheapest_set_price_mcm,
    //             price_difference: Some(price_difference),
    //             price_diff_percentage: Some(
    //                 ((vendor_card.price as f64 - cheapest_price_sek) / vendor_card.price as f64)
    //                     * 100.0,
    //             ),
    //         };
    //         card_prices.push(card_price);
    //     } else {
    //         log::info!(
    //             "Unable to find card: {} among scryfall cards",
    //             vendor_card.name
    //         );
    //     }
    // });

    return card_prices;
}

#[cfg(test)]
mod tests {

    use crate::scryfall::scryfall_mcm_cards::Prices;

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

        let brainstorm = VendorCard {
            name: "brainstorm".to_string(),
            vendor: Vendor::dragonslair,
            set: "ice age".to_string(),
            foil: false,
            image_url: "www.google.com".to_string(),
            extended_art: false,
            prerelease: false,
            showcase: false,
            price: 10,
            trade_in_price: 5,
            current_stock: 2,
            max_stock: 3,
        };

        let scryfall_brainstorm = ScryfallCard {
            name: "brainstorm".to_string(),
            set: "ice age".to_string(),
            image_url: "www.google.com".to_string(),
            prices: Prices {
                eur: Some(1.0),
                eur_foil: Some(2.0),
            },
        };

        let scryfall_brainstorm_modern = ScryfallCard {
            name: "brainstorm".to_string(),
            set: "modern masters".to_string(),
            image_url: "www.google.com".to_string(),
            prices: Prices {
                eur: Some(0.5),
                eur_foil: Some(1.0),
            },
        };

        let mut scryfall_cards = HashMap::new();
        scryfall_cards.insert(
            "brainstorm".to_string(),
            [scryfall_brainstorm, scryfall_brainstorm_modern].to_vec(),
        );

        let vendor_card_list = vec![brainstorm];

        let result = compare_prices(vendor_card_list, scryfall_cards, &url).await;
        mock.assert();
        assert_eq!(result.len(), 1);

        let expected_card_price = CardPrice {
            name: "brainstorm".to_string(),
            vendor: Vendor::dragonslair,
            set: "ice age".to_string(),
            foil: false,
            vendor_price: 10.0,
            same_set_price_mcm: Some(11.5),
            cheapest_set_price_mcm: Some(5.75),
            price_difference: Some(4.25),
            price_diff_percentage: Some(50.0),
        };

        assert_eq!(result[0], expected_card_price);
    }
}
