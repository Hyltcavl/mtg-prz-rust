use std::{collections::HashMap, error::Error};
use urlencoding::encode;

use crate::{
    cards::{
        card_parser::fetch_and_parse, cardname::CardName, currency::Currency,
        personalcard::PersonalCard, price::Price, tradable_card::TradeableCard,
        vendorcard::VendorCard,
    },
    dragonslair_scraper::DragonslairScraper,
};

pub struct TradableCardsComparer {
    dl_scraper: DragonslairScraper,
}

impl TradableCardsComparer {
    pub fn new(dl_scraper: DragonslairScraper) -> Self {
        TradableCardsComparer { dl_scraper }
    }

    pub async fn get_tradable_cards(
        &self,
        personal_cards: Vec<PersonalCard>,
        vendor_cards: HashMap<CardName, Vec<VendorCard>>,
    ) -> Result<Vec<TradeableCard>, Box<dyn Error>> {
        let (mut tradable_cards, leftover_personal_cards) =
            self.get_tradable_and_leftover_cards(personal_cards, vendor_cards);

        // Make function for checking the personal cards at the vendor site
        let mut vendor_cards_with_same_name_as_leftover: Vec<VendorCard> = vec![];
        for card in &leftover_personal_cards {
            let card_name_lowercase = card.name.almost_raw.to_lowercase();
            let card_name_encoded = encode(&card_name_lowercase);
            let request_url = format!(
                "/product/card-singles/magic/name:{}/{}",
                card_name_encoded, 0
            );
            let page_count = self
                .dl_scraper
                .get_page_count(&request_url)
                .await
                .unwrap_or(1);
            log::debug!("page_count for {} is {:?}", card_name_encoded, page_count);

            let urls = (1..=page_count)
                .map(|count| {
                    format!(
                        "{}/product/card-singles/magic/name:{}/{}",
                        self.dl_scraper.url, card_name_encoded, count
                    )
                })
                .collect::<Vec<String>>();

            for url in urls {
                log::info!("Fetching url: {}", url);
                let mut cards = fetch_and_parse(&url).await.unwrap();
                log::info!("Fetched cards: {:?}", &cards);
                vendor_cards_with_same_name_as_leftover.append(&mut cards);
            }
        }
        let mut grouped_vendor_cards: HashMap<CardName, Vec<VendorCard>> = HashMap::new();
        for card in &vendor_cards_with_same_name_as_leftover {
            grouped_vendor_cards
                .entry(card.name.clone())
                .or_insert_with(Vec::new)
                .push(card.clone());
        }

        let (mut more_tradable_cards, _unwanted_cards) =
            self.get_tradable_and_leftover_cards(leftover_personal_cards, grouped_vendor_cards);
        tradable_cards.append(&mut more_tradable_cards);

        log::debug!("tradable cards: {:?}", &tradable_cards);

        return Ok(tradable_cards);
    }

    fn get_tradable_and_leftover_cards(
        &self,
        personal_cards: Vec<PersonalCard>,
        vendor_cards: HashMap<CardName, Vec<VendorCard>>,
    ) -> (Vec<TradeableCard>, Vec<PersonalCard>) {
        let mut leftover_cards = vec![];

        let t_cards = personal_cards
            .iter()
            .filter_map(|p_card| {
                let card = vendor_cards
                    .get(&p_card.name)
                    .iter()
                    .flat_map(|v| v.iter())
                    .find(|v_card| {
                        v_card.name == p_card.name
                            && v_card.foil == p_card.foil
                            && v_card.set == p_card.set
                    });

                match card {
                    Some(v_card) => {
                        if v_card.current_stock < v_card.max_stock {
                            Some(TradeableCard {
                                name: v_card.name.clone(),
                                set: v_card.set.clone(),
                                foil: p_card.foil,
                                prerelease: v_card.prerelease,
                                tradeable_vendor: v_card.vendor.clone(),
                                trade_in_price: Price::new(
                                    v_card.trade_in_price.into(),
                                    Currency::SEK,
                                ),
                                mcm_price: p_card.price,
                                cards_to_trade: p_card.count.clone(),
                                card_ammount_requested_by_vendor: v_card.max_stock
                                    - v_card.current_stock,
                                image_url: v_card.image_url.clone(),
                                color: p_card.color.clone(),
                                rarity: p_card.rarity.clone(),
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
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        cards::{
            colour::Colour, currency::Currency, delver_lense_card::DelverLenseCard, price::Price,
            rarity::Rarity, setname::SetName, tradable_card::TradeableCard, vendor::Vendor,
            vendorcard::VendorCard,
        },
        test::helpers::{
            counterspell_forth_e, counterspell_ice_age, reaper_king_vendor_card_cheap,
            reaper_king_vendor_card_expensive, reaper_king_vendor_card_foil,
        },
        tradable_cards::delver_lense_converter::DelverLenseConverter,
    };

    use super::*;

    fn card_1() -> DelverLenseCard {
        DelverLenseCard {
            Name: "Reaper King".to_string(),
            Foil: "Foil".to_string(),
            Edition: "Shadowmoor".to_string(),
            Price: "200,00 €".to_string(),
            Quantity: "1".to_string(),
            Color: "WUBRG".to_string(),
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
            Color: "WUBRG".to_string(),
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
            Color: "Blue".to_string(),
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
            Color: "Blue".to_string(),
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
            Color: "Blue".to_string(),
            Rarity: "C".to_string(),
        }
    }

    #[test]
    fn get_tradable_cards_should_return_tradable_cards() {
        //Prepare vendor cards
        let card1 = reaper_king_vendor_card_expensive();
        let card2 = reaper_king_vendor_card_cheap();
        let card3 = reaper_king_vendor_card_foil();
        let card4 = counterspell_forth_e();
        let card5 = counterspell_ice_age();
        let vendor_cards = vec![card1, card2, card3, card4, card5];
        let mut vendor_cards_map: HashMap<CardName, Vec<VendorCard>> = HashMap::new();
        for card in &vendor_cards {
            vendor_cards_map
                .entry(card.name.clone())
                .or_insert_with(Vec::new)
                .push(card.clone());
        }

        // prepare delver cards
        let raw_cards = vec![card_1(), card_2(), card_3(), card_4(), card_5()];
        let delver_lense_converter = DelverLenseConverter::new();
        let personal_cards =
            delver_lense_converter.convert_delver_lense_card_to_personal_card(raw_cards);

        // prepare result
        let tradeable_card1 = TradeableCard {
            name: CardName::new("Reaper King".to_string()).unwrap(),
            set: SetName::new("Shadowmoor".to_string()).unwrap(),
            foil: true,
            prerelease: false,
            tradeable_vendor: Vendor::Dragonslair,
            trade_in_price: Price::new(100.0, Currency::SEK),
            mcm_price: Price::new(200.0, Currency::EUR),
            cards_to_trade: 1,
            card_ammount_requested_by_vendor: 1,
            image_url: "https://astraeus.dragonslair.se/images/4026/product".to_string(),
            color: Colour::WUBRG,
            rarity: Rarity::Rare,
        };
        let tradeable_card2 = TradeableCard {
            name: CardName::new("Reaper King".to_string()).unwrap(),
            set: SetName::new("Shadowmoor".to_string()).unwrap(),
            foil: false,
            prerelease: false,
            tradeable_vendor: Vendor::Dragonslair,
            trade_in_price: Price::new(50.0, Currency::SEK),
            mcm_price: Price::new(2.06, Currency::EUR),
            cards_to_trade: 1,
            card_ammount_requested_by_vendor: 1,
            image_url: "https://astraeus.dragonslair.se/images/4026/product".to_string(),
            color: Colour::WUBRG,
            rarity: Rarity::Rare,
        };

        let tradeable_card4 = TradeableCard {
            name: CardName::new("Counterspell".to_string()).unwrap(),
            set: SetName::new("Ice Age".to_string()).unwrap(),
            foil: false,
            prerelease: false,
            tradeable_vendor: Vendor::Dragonslair,
            trade_in_price: Price::new(50.0, Currency::SEK),
            mcm_price: Price::new(1.24, Currency::EUR),
            cards_to_trade: 1,
            card_ammount_requested_by_vendor: 2,
            image_url: "https://astraeus.dragonslair.se/images/4026/product".to_string(),
            color: Colour::Blue,
            rarity: Rarity::Common,
        };
        let expected_cards = vec![tradeable_card1, tradeable_card2, tradeable_card4];

        let dl_scraper = DragonslairScraper::new("www.test.com", None, reqwest::Client::new());
        let tradable_cards_comparer = TradableCardsComparer::new(dl_scraper);

        let (result_v_cards, leftover_cards) = tradable_cards_comparer
            .get_tradable_and_leftover_cards(personal_cards.clone(), vendor_cards_map);

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
        let mut vendor_cards_map: HashMap<CardName, Vec<VendorCard>> = HashMap::new();
        for card in &vendor_cards {
            vendor_cards_map
                .entry(card.name.clone())
                .or_insert_with(Vec::new)
                .push(card.clone());
        }

        let raw_cards = vec![card_1(), card_4()];
        let delver_lense_converter = DelverLenseConverter::new();
        let personal_cards =
            delver_lense_converter.convert_delver_lense_card_to_personal_card(raw_cards);

        let _tradeable_card1 = TradeableCard {
            name: CardName::new("Reaper King".to_string()).unwrap(),
            set: SetName::new("Shadowmoor".to_string()).unwrap(),
            foil: true,
            prerelease: false,
            tradeable_vendor: Vendor::Dragonslair,
            trade_in_price: Price::new(100.0, Currency::SEK),
            mcm_price: Price::new(200.0, Currency::EUR),
            cards_to_trade: 1,
            card_ammount_requested_by_vendor: 1,
            image_url: "https://astraeus.dragonslair.se/images/4026/product".to_string(),
            color: Colour::WUBRG,
            rarity: Rarity::Rare,
        };

        let tradeable_card4 = TradeableCard {
            name: CardName::new("Counterspell".to_string()).unwrap(),
            set: SetName::new("Ice Age".to_string()).unwrap(),
            foil: false,
            prerelease: false,
            tradeable_vendor: Vendor::Dragonslair,
            trade_in_price: Price::new(50.0, Currency::SEK),
            mcm_price: Price::new(1.24, Currency::EUR),
            cards_to_trade: 1,
            card_ammount_requested_by_vendor: 2,
            image_url: "https://astraeus.dragonslair.se/images/4026/product".to_string(),
            color: Colour::Blue,
            rarity: Rarity::Common,
        };

        let expected_tradable_cards = vec![tradeable_card4];
        let expected_leftover_cards = vec![personal_cards[0].clone()];

        let dl_scraper = DragonslairScraper::new("www.test.com", None, reqwest::Client::new());
        let tradable_cards_comparer = TradableCardsComparer::new(dl_scraper);
        let (result_v_cards, leftover_cards) = tradable_cards_comparer
            .get_tradable_and_leftover_cards(personal_cards, vendor_cards_map);

        assert_eq!(result_v_cards, expected_tradable_cards);
        assert_eq!(leftover_cards, expected_leftover_cards);
    }

    #[tokio::test]
    async fn test_get_tradable_cards() {
        let html_content = include_str!("../test/get_pages_page.html").to_string();

        let param = format!(
            "/product/card-singles/magic/name:{}/{}",
            encode("personalcard"),
            0
        );

        let mut server = std::thread::spawn(|| mockito::Server::new())
            .join()
            .unwrap();
        let url = server.url();
        let mock = server
            .mock("GET", &*param)
            .with_status(200)
            .with_header("content-type", "text/plain")
            .with_header("x-api-key", "1234")
            .with_body(html_content.clone())
            .create();

        let param = format!(
            "/product/card-singles/magic/name:{}/{}",
            encode("personalcard"),
            1
        );
        server
            .mock("GET", &*param)
            .with_status(200)
            .with_header("content-type", "text/plain")
            .with_header("x-api-key", "1234")
            .with_body(html_content.clone())
            .create();
        // for i in 1..=51 {
        //     let param = format!(
        //         "/product/card-singles/magic/name:{}/{}",
        //         encode("personalcard"),
        //         i
        //     );
        //     server
        //         .mock("GET", &*param)
        //         .with_status(200)
        //         .with_header("content-type", "text/plain")
        //         .with_header("x-api-key", "1234")
        //         .with_body(html_content.clone())
        //         .create();
        // }

        let vendor_cards_map: HashMap<CardName, Vec<VendorCard>> = HashMap::new();
        let personal_cards = PersonalCard {
            name: CardName::new("personalcard".to_string()).unwrap(),
            set: SetName::new("personal card set".to_string()).unwrap(),
            foil: false,
            price: Price::new(10.0, Currency::SEK),
            count: 2,
            color: Colour::Blue,
            rarity: Rarity::Rare,
        };

        let tradable_cards_comparer =
            TradableCardsComparer::new(DragonslairScraper::new(&url, None, reqwest::Client::new()));

        let tradable_cards = tradable_cards_comparer
            .get_tradable_cards([personal_cards].to_vec(), vendor_cards_map)
            .await
            .unwrap();

        mock.assert();

        assert!(tradable_cards[0].name.raw == "personalcard");

        // env_logger::builder().is_test(true).try_init();
        // let file_path = "/workspaces/mtg-prz-rust/Draftshaft_2025_Mar_10_17-33.csv";
        // let vendor_cards: HashMap<CardName, Vec<VendorCard>> = load_from_json_file::<
        //     HashMap<CardName, Vec<VendorCard>>,
        // >(
        //     "/workspaces/mtg-prz-rust/mtg-rust/dragonslair_cards/dl_cards_25_02_2025-16-14.json",
        // )
        // .unwrap();

        // let tradable_cards = get_tradable_cards(file_path, vendor_cards).await;
        // for card in tradable_cards.unwrap() {
        //     log::info!("Card: {:?}", card);
        // }
        // assert!(true);
    }

    // #[tokio::test]
    // #[ignore]
    // async fn test_fetch_card() {
    //     env_logger::builder().is_test(true).try_init();
    //     let file_path = "/workspaces/mtg-prz-rust/Draftshaft_2025_Mar_10_17-33.csv";
    //     let vendor_cards: HashMap<CardName, Vec<VendorCard>> = load_from_json_file::<
    //         HashMap<CardName, Vec<VendorCard>>,
    //     >(
    //         "/workspaces/mtg-prz-rust/mtg-rust/dragonslair_cards/dl_cards_25_02_2025-16-14.json",
    //     )
    //     .unwrap();

    //     let tradable_cards = get_tradable_cards(file_path, vendor_cards).await;
    //     for card in tradable_cards.unwrap() {
    //         log::info!("Card: {:?}", card);
    //     }
    //     assert!(true);
    // }
}
