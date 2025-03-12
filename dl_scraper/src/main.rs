mod cards;
mod dragonslair_scraper;
mod scryfall_scraper;
mod test;
mod utilities;

use std::collections::HashMap;

use crate::cards::cardname::CardName;
use crate::cards::vendorcard::VendorCard;
use crate::utilities::{config::CONFIG, string_manipulators::date_time_as_string};

use crate::cards::card_parser::fetch_and_parse;
use dragonslair_scraper::DragonslairScraper;
use futures::stream::{self, StreamExt};
use log::info;
use proflogger::profile;
use reqwest::Client;
use utilities::file_management::load_from_json_file;
use utilities::{file_management::get_newest_file, file_management::save_to_file};

async fn get_dragonslair_cards() -> HashMap<CardName, Vec<VendorCard>> {
    let start_time = chrono::prelude::Local::now();
    info!("Starting at {}", start_time);

    let scraper = DragonslairScraper::new(
        "https://astraeus.dragonslair.se",
        Some([0].to_vec()),
        Client::new(),
    );
    let dragoslair_cards = scraper.get_available_cards().await.unwrap();

    let dl_cards_path = format!(
        "/workspaces/mtg-prz-rust/dragonslair_cards/dl_cards_{}.json",
        date_time_as_string(None, None)
    );

    save_to_file(&dl_cards_path, &dragoslair_cards).unwrap();

    let end_time = chrono::prelude::Local::now();
    info!(
        "DL scan started at: {}. Finished at: {}. Took: {} seconds and with {} cards on dl_cards_path: {}",
        start_time,
        end_time,
        (end_time - start_time).num_seconds(),
        dragoslair_cards.len(),
        dl_cards_path
    );
    dragoslair_cards
}

fn get_data_from_most_recent_file<T>(
    folder_name: &str,
    file_prefix: &str,
) -> Result<HashMap<CardName, Vec<T>>, Box<dyn std::error::Error>>
where
    T: serde::de::DeserializeOwned,
{
    let newest_file = get_newest_file(
        &format!("/workspaces/mtg-prz-rust/{}", folder_name),
        file_prefix,
    )?;
    let res = newest_file.to_str().unwrap();

    load_cards(res)
}

fn load_cards<T>(path: &str) -> Result<HashMap<CardName, Vec<T>>, Box<dyn std::error::Error>>
where
    T: serde::de::DeserializeOwned,
{
    load_from_json_file::<HashMap<CardName, Vec<T>>>(path)
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

#[tokio::main]
async fn main() {
    env_logger::init();
    info!("Starting");

    let dl_cards_path = if CONFIG.dl {
        get_dragonslair_cards().await
    } else {
        match get_data_from_most_recent_file("dragonslair_cards", "dl_cards_") {
            Ok(cards) => cards,
            Err(e) => {
                log::error!("Failed to load Dragonslair cards: {}", e);
                HashMap::new()
            }
        }
    };
    // let scryfall_cards_path = if CONFIG.scryfall {
    //     log::info!("Downloading Scryfall cards...");
    //     download_scryfall_cards(None).await
    // } else {
    //     get_newest_file_path("scryfall_prices", "parsed_scryfall_cards_")
    // };
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::test::helpers::reaper_king_vendor_card_expensive;
//     use env_logger;
//     use tokio;
//     fn init() {
//         let _ = env_logger::builder().is_test(true).try_init();
//     }

//     #[tokio::test]
//     async fn test_get_page_count() {
//         let html_content = include_str!("test/get_pages_page.html").to_string();

//         let mut server = std::thread::spawn(|| mockito::Server::new())
//             .join()
//             .unwrap();
//         let url = server.url();
//         let mock = server
//             .mock(
//                 "GET",
//                 "/product/magic/card-singles/store:kungsholmstorg/cmc-0/1",
//             )
//             .with_status(200)
//             .with_header("content-type", "text/plain")
//             .with_header("x-api-key", "1234")
//             .with_body(html_content)
//             .create();
//         let url = format!(
//             "{}/product/magic/card-singles/store:kungsholmstorg/cmc-0/1",
//             url
//         );

//         let res = DragonslairScraper::new(&url, None, reqwest::Client::new())
//             .get_page_count(&url)
//             .await
//             .unwrap();

//         mock.assert();
//         assert_eq!(res, 51);
//     }
// }
