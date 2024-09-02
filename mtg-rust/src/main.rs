mod dl;
mod scryfall;
mod utils;
use dotenv;
use env_logger;
use log;

use std::collections::HashMap;
use std::env;
use std::sync::Arc;

use dl::card_parser::{fetch_and_parse, VendorCard};
use dl::list_links::get_page_count;
use futures::future::join_all;
use tokio::sync::Semaphore;
use utils::compare_prizes::compare_prices;
use utils::file_management::{get_newest_file, read_json_file, write_to_file};
use utils::string_manipulators::date_time_as_string;

use scryfall::scryfall_mcm_cards::{download_scryfall_cards, ScryfallCard};

/// Returns the dl cards file path.
async fn prepare_dl_cards() -> Result<String, Box<dyn std::error::Error>> {
    let start_time = chrono::prelude::Local::now();
    let base_url = "https://astraeus.dragonslair.se".to_owned();
    let semaphore = Arc::new(Semaphore::new(10));
    let cmcs_available = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 15, 16];
    // let cmcs_available = [11, 12, 13, 15, 16];

    let page_fetches_asyncs = cmcs_available.into_iter().map(|cmc| {
        let semaphore_clone = Arc::clone(&semaphore);

        let value = base_url.clone();
        async move {
            let _permit = semaphore_clone.acquire().await.unwrap(); //Get a permit to run in parallel
            let request_url = format_args!(
                "{}/product/magic/card-singles/store:kungsholmstorg/cmc-{}/{}",
                &value, cmc, 0
            )
            .to_string();
            let page_count = get_page_count(&request_url).await.unwrap_or(1);
            log::debug!("cmc: {}, and page_count is {:?}", cmc, page_count);
            (1..=page_count)
                .map(|count| {
                    format!(
                        "{}/product/magic/card-singles/store:kungsholmstorg/cmc-{}/{}",
                        &value, cmc, count
                    )
                })
                .collect::<Vec<String>>()
        }
    });

    let results: Vec<Vec<String>> = join_all(page_fetches_asyncs).await;
    log::debug!("{:?}", results);
    let semaphore = Arc::new(Semaphore::new(10));

    let future_cards = results.into_iter().flatten().map(|link| {
        let semaphore_clone = Arc::clone(&semaphore);
        async move {
            let _permit = semaphore_clone.acquire().await.unwrap(); //Get a permit to run in parallel
            match fetch_and_parse(&link).await {
                Ok(cards) => cards,
                Err(e) => {
                    log::error!("Error fetching cards from {}: {}", link, e);
                    Vec::new()
                }
            }
        }
    });
    let cards: Vec<VendorCard> = join_all(future_cards).await.into_iter().flatten().collect();

    let cards_as_string = serde_json::to_string(&cards).unwrap();
    let dl_cards_path = format!(
        "dragonslair_cards/dl_cards_{}.json",
        date_time_as_string(None, None)
    );
    write_to_file(&dl_cards_path, &cards_as_string)?;
    let end_time = chrono::prelude::Local::now();
    log::info!(
        "DL scan started at: {}. Finished at: {}. Took: {} seconds and with {} cards on dl_cards_path: {}",
        start_time,
        end_time,
        (end_time - start_time).num_seconds(),
        cards.len(),
        dl_cards_path
    );
    return Ok(dl_cards_path);
}

#[tokio::main]
async fn main() {
    env_logger::init();
    log::info!("Starting");
    let _ = dotenv::dotenv();

    let start_time = chrono::prelude::Local::now();

    // Check for environment variables
    let dl = env::var("DL").unwrap_or("1".to_owned()) == "1".to_owned();
    let scryfall = env::var("SCRYFALL").unwrap_or("1".to_owned()) == "1".to_owned();

    let mut scryfall_cards_path: Result<String, Box<dyn std::error::Error>> = Ok("".to_string());
    let mut dl_cards_path: Result<String, Box<dyn std::error::Error>> = Ok("".to_string());
    // Use feature flags in combination with environment variables
    if dl {
        log::info!("Downloading Dragonslair cards...");
        dl_cards_path = prepare_dl_cards().await;
    }

    if scryfall {
        log::info!("Downloading Scryfall cards...");
        scryfall_cards_path = download_scryfall_cards().await;
    }

    let mut dl_cards: Vec<VendorCard> = Vec::new();
    if !dl {
        log::info!("Loading existing Dragonslair cards...");
        let path = get_newest_file(
            "/workspaces/mtg-prz-rust/mtg-rust/dragonslair_cards",
            "dl_cards_",
        )
        .unwrap();
        dl_cards = read_json_file::<Vec<VendorCard>>(path.to_str().unwrap()).unwrap();
    } else {
        dl_cards = read_json_file::<Vec<VendorCard>>(&dl_cards_path.unwrap()).unwrap();
    }

    let mut scryfall_prices: HashMap<String, Vec<ScryfallCard>> = HashMap::new();
    if !scryfall {
        log::info!("Loading existing Scryfall cards...");
        let path = get_newest_file(
            "/workspaces/mtg-prz-rust/mtg-rust/scryfall_prices",
            "parsed_scryfall_cards_",
        )
        .unwrap();
        scryfall_prices =
            read_json_file::<HashMap<String, Vec<ScryfallCard>>>(path.to_str().unwrap()).unwrap();
    } else {
        scryfall_prices =
            read_json_file::<HashMap<String, Vec<ScryfallCard>>>(&scryfall_cards_path.unwrap())
                .unwrap();
    }

    let compared_prices =
        compare_prices(dl_cards, scryfall_prices, "https://api.frankfurter.app/").await;

    let parsed_file_path = format!(
        "compared_prices/compared_prices_{}.json",
        date_time_as_string(None, None)
    );
    let grouped_cards_as_string = serde_json::to_string(&compared_prices).unwrap();
    let compared_prices_file = write_to_file(&parsed_file_path, &grouped_cards_as_string).unwrap();
    let end_time = chrono::prelude::Local::now();

    log::info!(
        "Run started at: {}. Finished at: {}. Took: {}, compared prices in: {:?}",
        start_time,
        end_time,
        (end_time - start_time).num_seconds(),
        compared_prices_file
    );
}
