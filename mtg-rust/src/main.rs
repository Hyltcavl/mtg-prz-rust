mod dl;
mod scryfall;
mod utils;

use std::fs::{self, OpenOptions};
use std::path::Path;
use std::sync::Arc;

use dl::card_parser::{fetch_and_parse, Card};
use dl::list_links::get_page_count;
use futures::future::join_all;
use std::io::Write;
use tokio::sync::Semaphore;
use utils::date_to_string::date_time_as_string;
use utils::file_management::write_to_file;

use scryfall::scryfall_mcm_cards::download_scryfall_cards;

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
            println!("cmc: {}, and page_count is {:?}", cmc, page_count);
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

    println!("{:?}", results);
    let semaphore = Arc::new(Semaphore::new(10));

    let future_cards = results.into_iter().flatten().map(|link| {
        let semaphore_clone = Arc::clone(&semaphore);
        async move {
            let _permit = semaphore_clone.acquire().await.unwrap(); //Get a permit to run in parallel
            match fetch_and_parse(&link).await {
                Ok(cards) => cards,
                Err(e) => {
                    eprintln!("Error fetching cards from {}: {}", link, e);
                    Vec::new()
                }
            }
        }
    });
    let cards: Vec<Card> = join_all(future_cards).await.into_iter().flatten().collect();

    let cards_as_string = serde_json::to_string(&cards).unwrap();
    let dl_cards_path = format!(
        "dragonslair_cards/dl_cards_{}.json",
        date_time_as_string(None, None)
    );
    write_to_file(&dl_cards_path, &cards_as_string)?;
    let end_time = chrono::prelude::Local::now();
    println!(
        "DL scan started at: {}. Finished at: {}. Took: {} seconds",
        start_time,
        end_time,
        (end_time - start_time).num_seconds()
    );
    return Ok(dl_cards_path);
}

#[tokio::main]
async fn main() {
    // let dl_cards_path = prepare_dl_cards();
    let scryfall_cards_path = download_scryfall_cards();
    // dl_cards_path.await;
    scryfall_cards_path.await;
}
