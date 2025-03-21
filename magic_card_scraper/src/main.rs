mod alphaspel_scraper;
mod cards;
mod comparer;
mod dragonslair_scraper;
mod html_generator;
mod mtg_stock_price_checker;
mod scryfall_scraper;
mod test;
mod tradable_cards;
mod utilities;

use std::collections::HashMap;
use std::fs;

use crate::cards::cardname::CardName;
use crate::cards::vendorcard::VendorCard;
use crate::utilities::{config::CONFIG, string_manipulators::date_time_as_string};

use alphaspel_scraper::AlphaspelScraper;
use cards::compared_card::ComparedCard;
use cards::scryfallcard::ScryfallCard;

use comparer::Comparer;
use dragonslair_scraper::DragonslairScraper;

use html_generator::generate_nice_price_page;
use log::info;
use reqwest::Client;
use scryfall_scraper::ScryfallScraper;
use tradable_cards::delver_lense_converter::DelverLenseConverter;
use tradable_cards::html_generator::generate_page_content;
use tradable_cards::tradable_card_comparer::TradableCardsComparer;
use utilities::constants::{
    ALPHASPEL_CARDS_FOLDER, ALPHASPEL_CARDS_PREFIX, ALPHASPEL_URL, COMPARED_CARDS_DIR,
    COMPARED_FILE_PREFIX, DRAGONSLAIR_CARDS_FOLDER, DRAGONSLAIR_CARDS_PREFIX, DRAGONSLAIR_URL,
    MTG_STOCKS_BASE_URL, REPOSITORY_ROOT_PATH, SCRYFALL_CARDS_DIR, SCRYFALL_FILE_PREFIX,
};
use utilities::file_management::load_from_json_file;
use utilities::{file_management::get_newest_file, file_management::save_to_file};

async fn get_alphaspel_cards_and_save_to_file() -> HashMap<CardName, Vec<VendorCard>> {
    let start_time = chrono::prelude::Local::now();
    info!("Starting at {}", start_time);

    let scraper = AlphaspelScraper::new(ALPHASPEL_URL);
    let alphaspel_cards = scraper.scrape_cards().await.unwrap();

    let as_cards_path = format!(
        "{}/{}/{}{}.json",
        REPOSITORY_ROOT_PATH,
        ALPHASPEL_CARDS_FOLDER,
        ALPHASPEL_CARDS_PREFIX,
        date_time_as_string(None, None)
    );

    save_to_file(&as_cards_path, &alphaspel_cards).unwrap();

    let end_time = chrono::prelude::Local::now();
    info!(
        "Alphaspel scrape started at: {}. Finished at: {}. Took: {} seconds and with {} cards on as_cards_path: {}",
        start_time,
        end_time,
        (end_time - start_time).num_seconds(),
        alphaspel_cards.len(),
        as_cards_path
    );
    alphaspel_cards
}

async fn get_dragonslair_cards_and_save_to_file() -> HashMap<CardName, Vec<VendorCard>> {
    let start_time = chrono::prelude::Local::now();
    info!("Starting at {}", start_time);

    let scraper: DragonslairScraper = DragonslairScraper::new(DRAGONSLAIR_URL, None, Client::new());
    let dragoslair_cards = scraper.get_available_cards().await.unwrap();

    let dl_cards_path = format!(
        "{}/{}/{}{}.json",
        REPOSITORY_ROOT_PATH,
        DRAGONSLAIR_CARDS_FOLDER,
        DRAGONSLAIR_CARDS_PREFIX,
        date_time_as_string(None, None)
    );

    save_to_file(&dl_cards_path, &dragoslair_cards).unwrap();

    let end_time = chrono::prelude::Local::now();
    info!(
        "DL scrape started at: {}. Finished at: {}. Took: {} seconds and with {} cards on dl_cards_path: {}",
        start_time,
        end_time,
        (end_time - start_time).num_seconds(),
        dragoslair_cards.len(),
        dl_cards_path
    );
    dragoslair_cards
}

async fn get_scryfall_cards_and_save_to_file(
) -> HashMap<CardName, Vec<cards::scryfallcard::ScryfallCard>> {
    let start_time = chrono::prelude::Local::now();
    info!("Starting at {}", start_time);

    let scryfall_scraper = ScryfallScraper::new(None, reqwest::Client::new(), None);
    let path_to_raw_scryfall_cards_file = scryfall_scraper
        .get_raw_scryfall_cards_file()
        .await
        .unwrap();
    let scryfall_cards = scryfall_scraper
        .convert_raw_to_domain_cards(&path_to_raw_scryfall_cards_file)
        .unwrap();

    let scryfall_cards_path = format!(
        "{}/{}/{}{}.json",
        REPOSITORY_ROOT_PATH,
        SCRYFALL_CARDS_DIR,
        SCRYFALL_FILE_PREFIX,
        date_time_as_string(None, None)
    );

    save_to_file(&scryfall_cards_path, &scryfall_cards).unwrap();

    let end_time = chrono::prelude::Local::now();
    info!(
        "Scryfall scrape started at: {}. Finished at: {}. Took: {} seconds and with {} cards on scryfall_cards_path: {}",
        start_time,
        end_time,
        (end_time - start_time).num_seconds(),
        scryfall_cards.len(),
        scryfall_cards_path
    );
    scryfall_cards
}

async fn compare_cards_and_save_to_file(
    scryfall_cards: HashMap<CardName, Vec<ScryfallCard>>,
    vendor_cards: HashMap<CardName, Vec<VendorCard>>,
) -> HashMap<CardName, Vec<ComparedCard>> {
    let start_time = chrono::prelude::Local::now();
    info!("Starting at {}", start_time);

    let comparer = Comparer::new(scryfall_cards, MTG_STOCKS_BASE_URL.to_string());
    let compared_cards = comparer.compare_vendor_cards(vendor_cards).await;

    let cards_path = format!(
        "{}/{}/{}{}.json",
        REPOSITORY_ROOT_PATH,
        COMPARED_CARDS_DIR,
        COMPARED_FILE_PREFIX,
        date_time_as_string(None, None)
    );

    save_to_file(&cards_path, &compared_cards).unwrap();

    let end_time = chrono::prelude::Local::now();
    info!(
        "Comparing cards started at: {}. Finished at: {}. Took: {} seconds and with {} cards in dir: {}",
        start_time,
        end_time,
        (end_time - start_time).num_seconds(),
        compared_cards.len(),
        cards_path
    );
    compared_cards
}

fn get_data_from_most_recent_file<T>(
    folder_name: &str,
    file_prefix: &str,
) -> Result<HashMap<CardName, Vec<T>>, Box<dyn std::error::Error>>
where
    T: serde::de::DeserializeOwned,
{
    let newest_file = get_newest_file(&format!("../{}", folder_name), file_prefix)?;
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
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    info!("Starting");

    let mut dl_cards = if CONFIG.dragonslair {
        get_dragonslair_cards_and_save_to_file().await
    } else {
        match get_data_from_most_recent_file(DRAGONSLAIR_CARDS_FOLDER, DRAGONSLAIR_CARDS_PREFIX) {
            Ok(cards) => cards,
            Err(e) => {
                log::error!("Failed to load Dragonslair cards: {}", e);
                HashMap::new()
            }
        }
    };

    if !CONFIG.delver_lense_path.is_empty() {
        let delver_lense_converter = DelverLenseConverter::new();
        let cards = delver_lense_converter
            .get_delver_lense_cards_from_file(&CONFIG.delver_lense_path)
            .unwrap();

        let comparer = TradableCardsComparer::new(DragonslairScraper::new(
            DRAGONSLAIR_URL,
            None,
            reqwest::Client::new(),
        ));

        let tradable_cards = comparer
            .get_tradable_cards(cards, dl_cards.clone())
            .await
            .unwrap();
        let path = format!(
            "../tradable_cards/tradable_cards_{}.json",
            date_time_as_string(None, None)
        );
        save_to_file(&path, &tradable_cards)?;

        let html = generate_page_content(&tradable_cards);

        fs::write("../cards.html", html)?;
    }

    let alphaspel_cards = if CONFIG.alpha {
        get_alphaspel_cards_and_save_to_file().await
    } else {
        match get_data_from_most_recent_file(ALPHASPEL_CARDS_FOLDER, ALPHASPEL_CARDS_PREFIX) {
            Ok(cards) => cards,
            Err(e) => {
                log::error!("Failed to load alphaspel cards: {}", e);
                HashMap::new()
            }
        }
    };

    let scryfall_cards_path = if CONFIG.scryfall {
        log::info!("Downloading Scryfall cards...");
        get_scryfall_cards_and_save_to_file().await
    } else {
        match get_data_from_most_recent_file(SCRYFALL_CARDS_DIR, SCRYFALL_FILE_PREFIX) {
            Ok(cards) => cards,
            Err(e) => {
                log::error!("Failed to load Scryfall cards: {}", e);
                HashMap::new()
            }
        }
    };

    for (name, alpha_cards) in alphaspel_cards {
        dl_cards
            .entry(name)
            .and_modify(|e| e.extend(alpha_cards.clone()))
            .or_insert(alpha_cards);
    }
    let compared_cards = compare_cards_and_save_to_file(scryfall_cards_path, dl_cards).await;

    let _ = generate_nice_price_page(compared_cards, "../", "index.html", CONFIG.nice_price_diff);

    Ok(())
}
