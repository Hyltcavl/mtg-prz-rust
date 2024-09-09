mod alphaspel;
mod cards;
mod dl;
mod html;
mod scryfall;
mod test;
mod utils;

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
        scryfall_cards_path = download_scryfall_cards(None).await;
    }

    let dl_cards: HashMap<CardName, Vec<VendorCard>>;
    if !dl {
        log::info!("Loading existing Dragonslair cards...");
        let path = get_newest_file(
            "/workspaces/mtg-prz-rust/mtg-rust/dragonslair_cards",
            "dl_cards_",
        )
        .unwrap();
        dl_cards =
            load_from_json_file::<HashMap<CardName, Vec<VendorCard>>>(path.to_str().unwrap())
                .unwrap();
    } else {
        dl_cards =
            load_from_json_file::<HashMap<CardName, Vec<VendorCard>>>(&dl_cards_path.unwrap())
                .unwrap();
    }

    let scryfall_prices: HashMap<CardName, Vec<ScryfallCard>>;
    if !scryfall {
        log::info!("Loading existing Scryfall cards...");
        let path = get_newest_file(
            "/workspaces/mtg-prz-rust/mtg-rust/scryfall_prices",
            "parsed_scryfall_cards_",
        )
        .unwrap();
        scryfall_prices =
            load_from_json_file::<HashMap<CardName, Vec<ScryfallCard>>>(path.to_str().unwrap())
                .unwrap();
    } else {
        scryfall_prices = load_from_json_file::<HashMap<CardName, Vec<ScryfallCard>>>(
            &scryfall_cards_path.unwrap(),
        )
        .unwrap();
    }

    // let dl_keys: Vec<&CardName> = dl_cards.keys().collect();
    // let sampled_keys: Vec<&CardName> = dl_keys.iter().take(10).cloned().collect();

    // let scryfall_keys: Vec<&CardName> = scryfall_prices.keys().collect();
    // let scryfall_sample_keys: Vec<&CardName> = scryfall_keys.iter().take(10).cloned().collect();

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

    let compared_prices: Vec<utils::compare_prices::ComparedCard> =
        compare_prices(dl_cards, scryfall_prices, "https://api.frankfurter.app/").await;

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
