mod alphaspel;
mod cards;
mod delver_lense;
mod dl;
mod html;
mod scryfall;
mod test;
mod utils;
pub mod db;

use alphaspel::card_parser::download_alpha_cards;
use cards::card::{CardName, ScryfallCard, VendorCard};
use dotenv;
use env_logger;
use html::html_generator::generate_html_from_json;
use log;
use utils::compare_prices::{compare_prices, ComparedCard};

use std::collections::HashMap;
use std::env;
use std::sync::Arc;

use dl::card_parser::fetch_and_parse;
use dl::list_links::get_page_count;
use futures::future::join_all;
use tokio::sync::Semaphore;
use utils::file_management::{get_newest_file, load_from_json_file, save_to_json_file};
use utils::string_manipulators::date_time_as_string;

use scryfall::scryfall_mcm_cards::download_scryfall_cards;

/// Returns the dl cards file path.
async fn prepare_dl_cards() -> Result<String, Box<dyn std::error::Error>> {
    let start_time = chrono::prelude::Local::now();
    let base_url = "https://astraeus.dragonslair.se".to_owned();
    let semaphore = Arc::new(Semaphore::new(10));
    let cmcs_available = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 15, 16];

    let page_fetches_asyncs = cmcs_available.into_iter().map(|cmc| {
        log::info!("Fetching DL cards with cmc: {}", cmc);

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

    let mut grouped_cards: HashMap<CardName, Vec<VendorCard>> = HashMap::new();
    for card in &cards {
        grouped_cards
            .entry(card.name.clone())
            .or_insert_with(Vec::new)
            .push(card.clone());
    }
    // let cards_as_string = serde_json::to_string(&grouped_cards).unwrap();
    let dl_cards_path = format!(
        "dragonslair_cards/dl_cards_{}.json",
        date_time_as_string(None, None)
    );

    save_to_json_file(&dl_cards_path, &grouped_cards)?;

    // write_to_file(&dl_cards_path, &cards_as_string)?;
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
    let (dl, scryfall, alpha) = check_env_vars();

    let dl_cards_path = if dl {
        log::info!("Downloading Dragonslair cards...");
        prepare_dl_cards().await
    } else {
        Ok(get_newest_file_path("dragonslair_cards", "dl_cards_"))
    };

    let scryfall_cards_path = if scryfall {
        log::info!("Downloading Scryfall cards...");
        download_scryfall_cards(None).await
    } else {
        Ok(get_newest_file_path(
            "scryfall_prices",
            "parsed_scryfall_cards_",
        ))
    };

    let alpha_cards_path = if alpha {
        log::info!("Downloading Alphaspel cards...");
        download_alpha_cards("https://alphaspel.se").await
    } else {
        Ok(get_newest_file_path("alphaspel_cards", "alphaspel_cards_"))
    };
    let dl_cards = load_cards(dl_cards_path, "Dragonslair").unwrap_or_default();
    let alpha_cards = load_cards(alpha_cards_path, "Alphaspel").unwrap_or_default();
    let scryfall_prices = load_cards(scryfall_cards_path, "Scryfall").unwrap_or_default();

    let vendor_cards = merge_card_maps(dl_cards, alpha_cards);

    let sample_prices: HashMap<CardName, Vec<ScryfallCard>> = scryfall_prices
        .iter()
        .take(10)
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    let parsed_file_path = format!(
        "scryfall_prices/samplecards_{}.json",
        date_time_as_string(None, None)
    );
    save_to_json_file(&parsed_file_path, &sample_prices).unwrap();

    let compared_prices: Vec<ComparedCard> = compare_prices(
        vendor_cards,
        scryfall_prices,
        "https://api.frankfurter.app/",
    )
    .await;

    let compared_cards_path = format!(
        "compared_prices/compared_prices_{}.json",
        date_time_as_string(None, None)
    );

    let _ = save_to_json_file::<Vec<ComparedCard>>(&compared_cards_path, &compared_prices).unwrap();

    let _ = generate_html_from_json(&compared_cards_path, "../");

    let end_time = chrono::prelude::Local::now();
    log::info!(
        "Run started at: {}. Finished at: {}. Took: {}, compared prices in: {:?}",
        start_time,
        end_time,
        (end_time - start_time).num_seconds(),
        compared_cards_path
    );
}

fn check_env_vars() -> (bool, bool, bool) {
    let dl = env::var("DL").unwrap_or("1".to_owned()) == "1".to_owned();
    let scryfall = env::var("SCRYFALL").unwrap_or("1".to_owned()) == "1".to_owned();
    let alpha = env::var("ALPHASPEL").unwrap_or("1".to_owned()) == "1".to_owned();
    (dl, scryfall, alpha)
}

fn get_newest_file_path(folder: &str, prefix: &str) -> String {
    get_newest_file(
        &format!("/workspaces/mtg-prz-rust/mtg-rust/{}", folder),
        prefix,
    )
    .unwrap()
    .to_str()
    .unwrap()
    .to_owned()
}

fn load_cards<T>(
    path: Result<String, Box<dyn std::error::Error>>,
    card_type: &str,
) -> Result<HashMap<CardName, Vec<T>>, Box<dyn std::error::Error>>
where
    T: serde::de::DeserializeOwned,
{
    log::info!("Loading existing {} cards...", card_type);
    path.and_then(|p| load_from_json_file::<HashMap<CardName, Vec<T>>>(&p).map_err(|e| e.into()))
        .or_else(|_| {
            log::warn!(
                "No existing {} cards found, using empty collection.",
                card_type
            );
            Ok(HashMap::new())
        })
}

fn merge_card_maps(
    map1: HashMap<CardName, Vec<VendorCard>>,
    map2: HashMap<CardName, Vec<VendorCard>>,
) -> HashMap<CardName, Vec<VendorCard>> {
    let mut merged_map = map1;

    for (key, mut value) in map2 {
        merged_map
            .entry(key)
            .or_insert_with(Vec::new)
            .append(&mut value);
    }

    merged_map
}
