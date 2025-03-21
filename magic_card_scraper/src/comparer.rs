use std::collections::HashMap;

use futures::{stream, StreamExt};
use log::{debug, error, info};
use reqwest::Client;

use crate::{
    cards::{
        cardname::CardName, compared_card::ComparedCard, currency::Currency, price::Price,
        scryfallcard::ScryfallCard, vendorcard::VendorCard,
    },
    mtg_stock_price_checker::MtgPriceFetcher,
    utilities::config::CONFIG,
};

pub struct Comparer {
    mcm_cards: HashMap<CardName, Vec<ScryfallCard>>,
    mtg_stock_url: String, // This struct is empty
}

impl Comparer {
    pub fn new(mcm_cards: HashMap<CardName, Vec<ScryfallCard>>, mtg_stock_url: String) -> Self {
        Comparer {
            mcm_cards,
            mtg_stock_url,
        }
    }

    fn separete_foil_and_non_foil_cards(
        &self,
        vendor_cards: HashMap<CardName, Vec<VendorCard>>,
    ) -> (
        HashMap<CardName, Vec<VendorCard>>,
        HashMap<CardName, Vec<VendorCard>>,
    ) {
        let mut foil_cards = HashMap::new();
        let mut non_foil_cards = HashMap::new();

        for (card_name, vendor_card_list) in vendor_cards {
            let (foil, non_foil): (Vec<VendorCard>, Vec<VendorCard>) =
                vendor_card_list.into_iter().partition(|card| card.foil);

            if !foil.is_empty() {
                foil_cards.insert(card_name.clone(), foil);
            }
            if !non_foil.is_empty() {
                non_foil_cards.insert(card_name, non_foil);
            }
        }

        (foil_cards, non_foil_cards)
    }

    pub async fn compare_vendor_cards(
        &self,
        vendor_cards: HashMap<CardName, Vec<VendorCard>>,
    ) -> HashMap<CardName, Vec<ComparedCard>> {
        // Separate foil and non-foil cardss
        let (foil_cards, non_foil_cards) = self.separete_foil_and_non_foil_cards(vendor_cards);
        let price_fetcher = MtgPriceFetcher::new(Client::new(), self.mtg_stock_url.clone());

        // Process non-foil cards
        info!("Comparing non-foil cards");
        let non_foil_results = self.compare(non_foil_cards, price_fetcher.clone()).await;
        info!("Comparing foil cards");
        let foil_results = self.compare(foil_cards, price_fetcher).await;

        // Return combined results
        let compared_cards = [non_foil_results, foil_results].concat();

        let mut grouped_cards: HashMap<CardName, Vec<ComparedCard>> = HashMap::new();
        for card in compared_cards {
            grouped_cards
                .entry(card.vendor_card.name.clone())
                .or_insert_with(Vec::new)
                .push(card.clone());
        }

        grouped_cards
    }

    async fn compare(
        &self,
        vendor_cards: HashMap<CardName, Vec<VendorCard>>,
        price_fetcher: MtgPriceFetcher,
    ) -> Vec<ComparedCard> {
        stream::iter(vendor_cards.iter())
            .map(|(card_name, vendor_card_list)| {
                let scryfall_cards = match self.mcm_cards.get(card_name) {
                    Some(cards) => cards.clone(),
                    None => {
                        error!("No Scryfall card found for: {}", card_name.almost_raw);
                        Vec::new()
                    }
                };

                let fetcher = price_fetcher.clone();

                async move {
                    self.compare_vendorcards_to_mcm_cards(vendor_card_list, scryfall_cards, fetcher)
                        .await
                }
            })
            .buffered(25)
            .collect::<Vec<Vec<ComparedCard>>>()
            .await
            .into_iter()
            .flatten()
            .collect()
    }

    /// Process all vendor cards for a specific card name
    async fn compare_vendorcards_to_mcm_cards(
        &self,
        vendor_cards: &[VendorCard],
        scryfall_cards: Vec<ScryfallCard>,
        price_fetcher: MtgPriceFetcher,
    ) -> Vec<ComparedCard> {
        if scryfall_cards.is_empty() {
            debug!(
                "Scryfall card list empty for card: {}",
                vendor_cards[0].name.almost_raw
            );
            return Vec::new();
        }

        stream::iter(vendor_cards.iter())
            .map(|vendor_card| {
                let scryfall_list = scryfall_cards.clone();
                let fetcher = price_fetcher.clone();

                async move {
                    self.compare_price_of_specific_vendor_card(
                        vendor_card,
                        &scryfall_list,
                        &fetcher,
                    )
                    .await
                }
            })
            .buffered(25)
            .filter_map(|result| async move { result })
            .collect()
            .await
    }

    /// Compare a vendor card with a matching Scryfall card
    async fn compare_price_of_specific_vendor_card(
        &self,
        vendor_card: &VendorCard,
        scryfall_cards: &[ScryfallCard],
        price_fetcher: &MtgPriceFetcher,
    ) -> Option<ComparedCard> {
        // Find matching Scryfall card by set
        let matching_scryfall_card = scryfall_cards
            .iter()
            .find(|card| card.collector_number == vendor_card.collector_number)
            .or_else(|| {
                scryfall_cards
                    .iter()
                    .find(|card| card.set == vendor_card.set)
            })
            .cloned()
            .or_else(|| {
                debug!(
                    "No matching Scryfall card found for vendor card: {} on set: {}",
                    vendor_card.name.almost_raw, vendor_card.set.cleaned
                );
                None
            })?;

        let mcm_price = if vendor_card.foil {
            // Get price data - either from Scryfall or fetch live price
            match matching_scryfall_card.prices.eur_foil {
                Some(price) => price,
                None => self.fetch_live_price(vendor_card, price_fetcher).await,
            }
        } else {
            // Get price data - either from Scryfall or fetch live price
            match matching_scryfall_card.prices.eur {
                Some(price) => price,
                None => self.fetch_live_price(vendor_card, price_fetcher).await,
            }
        };

        // Create comparison data
        Some(ComparedCard {
            vendor_card: vendor_card.clone(),
            scryfall_card: matching_scryfall_card,
            price_difference_to_cheapest_vendor_card: (vendor_card.price.convert_to(Currency::SEK)
                - mcm_price.convert_to(Currency::SEK))
                as i32,
        })
    }

    /// Fetch live price for a card
    async fn fetch_live_price(
        &self,
        vendor_card: &VendorCard,
        price_fetcher: &MtgPriceFetcher,
    ) -> Price {
        if !CONFIG.external_price_check {
            return Price::new(0.0, Currency::EUR);
        }

        match price_fetcher
            .get_live_card_price(vendor_card.name.clone(), vendor_card.set.clone())
            .await
        {
            Ok(price) => price,
            Err(e) => {
                log::error!(
                    "Price fetch failed for {}: {}",
                    vendor_card.name.almost_raw,
                    e
                );
                Price::new(0.0, Currency::SEK)
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use std::collections::HashMap;

    use tracing_test::traced_test;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    use crate::{
        cards::currency::Currency,
        test::helpers::{
            cardname_sunken_ruins, lifecraft_c_name, lifecraft_c_scryfall_card,
            lifecraft_c_vendor_card, lifecraft_scryfall_card_no_price, reaper_king_card_name,
            reaper_king_scryfall_card_cheap, reaper_king_scryfall_card_expensive,
            reaper_king_vendor_card_cheap, reaper_king_vendor_card_expensive,
            reaper_king_vendor_card_foil, scryfall_card_sunken_ruins,
            vendor_card_sunken_ruins_foil,
        },
    };

    use super::*;

    #[tokio::test]
    #[traced_test]
    async fn test_compare_prices() {
        init();
        // tracing_test::internal::logs_assert("ERROR", |logs| {
        //     assert!(logs.is_empty());
        //     Ok(())
        // });

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
            (
                cardname_sunken_ruins(),
                vec![vendor_card_sunken_ruins_foil()],
            ),
        ]);

        let scryfall_cards = HashMap::from([
            (
                reaper_king_card_name(),
                vec![
                    reaper_king_scryfall_card_expensive(),
                    reaper_king_scryfall_card_cheap(),
                ],
            ),
            (
                lifecraft_c_name(),
                vec![
                    lifecraft_c_scryfall_card(),
                    lifecraft_scryfall_card_no_price(),
                ],
            ),
            (cardname_sunken_ruins(), vec![scryfall_card_sunken_ruins()]),
        ]);

        let comparer = Comparer::new(scryfall_cards, "url".to_string());

        // Initialize the logger for capturing logs during the test
        // let _ = env_logger::builder().is_test(true).try_init();

        let result = comparer.compare_vendor_cards(vendor_card_list).await;

        let price_diff = (reaper_king_vendor_card_expensive()
            .price
            .convert_to(Currency::SEK)
            - reaper_king_scryfall_card_expensive()
                .prices
                .eur
                .unwrap()
                .convert_to(Currency::SEK)) as i32;
        let non_foil_card = ComparedCard {
            vendor_card: reaper_king_vendor_card_expensive(),
            scryfall_card: reaper_king_scryfall_card_expensive(),
            price_difference_to_cheapest_vendor_card: price_diff,
        };

        let price_diff_foil = (reaper_king_vendor_card_cheap()
            .price
            .convert_to(Currency::SEK)
            - reaper_king_scryfall_card_cheap()
                .prices
                .eur
                .unwrap()
                .convert_to(Currency::SEK)) as i32;
        let foil_card = ComparedCard {
            vendor_card: reaper_king_vendor_card_cheap(),
            scryfall_card: reaper_king_scryfall_card_cheap(),
            price_difference_to_cheapest_vendor_card: price_diff_foil,
        };

        let price_diff_lifecraft = (lifecraft_c_vendor_card().price.convert_to(Currency::SEK)
            - lifecraft_c_scryfall_card()
                .prices
                .eur_foil
                .unwrap()
                .convert_to(Currency::SEK)) as i32;
        let lifecraft = ComparedCard {
            vendor_card: lifecraft_c_vendor_card(),
            scryfall_card: lifecraft_c_scryfall_card(),
            price_difference_to_cheapest_vendor_card: price_diff_lifecraft,
        };

        let price_diff_sunken_ruins = (vendor_card_sunken_ruins_foil()
            .price
            .convert_to(Currency::SEK)
            - scryfall_card_sunken_ruins()
                .prices
                .eur_foil
                .unwrap()
                .convert_to(Currency::SEK)) as i32;
        let sunken_ruins_diff = ComparedCard {
            vendor_card: vendor_card_sunken_ruins_foil(),
            scryfall_card: scryfall_card_sunken_ruins(),
            price_difference_to_cheapest_vendor_card: price_diff_sunken_ruins,
        };

        assert_eq!(result.len(), 3);

        assert_eq!(
            result.get(&lifecraft.vendor_card.name).unwrap()[0],
            lifecraft
        );
        assert_eq!(
            result.get(&non_foil_card.vendor_card.name).unwrap()[0],
            non_foil_card
        );
        assert_eq!(
            result.get(&foil_card.vendor_card.name).unwrap()[1],
            foil_card
        );

        assert_eq!(
            result.get(&sunken_ruins_diff.vendor_card.name).unwrap()[0],
            sunken_ruins_diff
        );
        // tracing_test::internal::logs_assert("error", )

        // logs_assert(|lines: &[&str]| {
        //     lines.iter().for_each(|line| {
        //         assert!(
        //             !line.contains("No Scryfall card found for:"),
        //             "{}", &format!("{}", line)
        //         )
        //     });
        //     Ok(())
        // });
    }
}
