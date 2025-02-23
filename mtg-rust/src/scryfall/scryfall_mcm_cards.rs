use log;
use reqwest;
use serde_json::Value;
use std::collections::HashMap;

use crate::cards::card::{CardName, Prices, ScryfallCard, SetName};
use crate::utils::file_management::{
    download_and_save_file, read_json_file, save_to_json_file,
};
use crate::utils::string_manipulators::{clean_string, date_time_as_string};

use chrono::Local;

#[cfg(not(test))]
fn get_existing_scryfall_file() -> Option<String> {
    use std::{fs, path::Path};

    let current_date = Local::now().format("%Y-%m-%d").to_string();
    let directory = Path::new("scryfall_prices_raw");

    if let Ok(entries) = fs::read_dir(directory) {
        for entry in entries {
            if let Ok(entry) = entry {
                let file_name = entry.file_name().into_string().unwrap_or_default();
                if file_name.starts_with(&format!("scryfall_download_{}", current_date))
                    && file_name.ends_with(".json")
                {
                    return Some(entry.path().to_str().unwrap().to_string());
                }
            }
        }
    }

    None
}

#[cfg(test)]
fn get_existing_scryfall_file() -> Option<String> {
    None // Always return None during tests
}

/// Returns path to the downloaded scryfall price file.
/// Defalts to no prefix if None is given
async fn get_scryfall_price_file(
    mut _prefix: Option<String>,
    url: Option<String>,
) -> Result<String, Box<dyn std::error::Error>> {
    // Check for existing file first
    if let Some(existing_file) = get_existing_scryfall_file() {
        log::info!("Using existing Scryfall price file: {}", existing_file);
        return Ok(existing_file);
    }

    let _prefix: String = _prefix.unwrap_or("".to_string());
    // Get available scryfall price files list
    let url = match url {
        Some(url) => url,
        None => "https://api.scryfall.com".to_string(),
    };

    let url = format!("{}/bulk-data", url);

    let client = reqwest::Client::new();

    let mut header_map = reqwest::header::HeaderMap::new();
    header_map.insert(reqwest::header::ACCEPT, "*/*".parse()?);
    header_map.insert(reqwest::header::USER_AGENT, "application/json".parse()?);
    let price_files_available = client
        .get(url)
        .headers(header_map.clone())
        .send()
        .await?
        .text()
        .await?;

    let json: serde_json::Value = serde_json::from_str(&price_files_available)?;
    // Get the latest price list uri of the smallest size
    let download_uri = json["data"][2]["download_uri"].as_str().unwrap();
    log::debug!("Downloading 'smaller list' from URL: {:?}", download_uri);
    let current_time = Local::now().format("%Y-%m-%d_%H:%M:%S").to_string();
    let path = format!(
        "scryfall_prices_raw/scryfall_download_{}.json",
        current_time
    );
    download_and_save_file(download_uri, &path).await?;
    log::info!("Saved raw scryfall price file to: {}", path);

    Ok(path)
}

/// Returns path to the scryfall cards file.
pub async fn download_scryfall_cards(
    url: Option<String>,
) -> Result<String, Box<dyn std::error::Error>> {
    let path = get_scryfall_price_file(None, url).await?;
    let cards: serde_json::Value = read_json_file(&path)?;
    let mut scryfall_card_list = Vec::new();

    // let cards = load_from_json_file::<serde_json::Value>(&path).map_err(|err| {
    //     log::info!("Failed to load raw scryfall price file: {}", err);
    //     log::error!("Failed to load raw scryfall price file: {}", err);
    //     err
    // })?;
    log::info!("Loaded raw scryfall price file, creating parsed scrayfall cards");
    if let Value::Array(cards_array) = cards {
        for obj in cards_array {
            if is_not_token(&obj) && is_not_basic_land(&obj) && is_not_artseries(&obj) {
                let name = clean_string(obj["name"].as_str().unwrap()).to_string();
                let set = clean_string(obj["set_name"].as_str().unwrap()).to_string();
                // let prices = obj["prices"].clone();
                let eur: Option<f64> = obj["prices"]["eur"]
                    .as_str()
                    .and_then(|s| s.parse().ok())
                    .or_else(|| obj["prices"]["eur"].as_f64());

                let eur_foil: Option<f64> = obj["prices"]["eur_foil"]
                    .as_str()
                    .and_then(|s| s.parse().ok())
                    .or_else(|| obj["prices"]["eur_foil"].as_f64());

                let prices = Prices {
                    eur: eur,
                    eur_foil: eur_foil,
                };
                let image_url = obj["image_uris"]["normal"]
                    .as_str()
                    .unwrap_or("https://www.google.com/url?sa=i&url=https%3A%2F%2Fanswers.microsoft.com%2Fen-us%2Fwindows%2Fforum%2Fall%2Fhigh-ram-usage-40-50-without-any-program%2F1dcf1e4d-f78e-4a06-a4e8-71f3972cc852&psig=AOvVaw0f3g3-hf1qnv6thWr6iQC2&ust=1724858067666000&source=images&cd=vfe&opi=89978449&ved=0CBQQjRxqFwoTCNjH-Ja7lYgDFQAAAAAdAAAAABAE")
                    .to_string();

                let card_name = match CardName::new(name.clone()) {
                    Ok(name) => name,
                    Err(e) => {
                        log::debug!("Error parsing card name: '{}', with error: {}", name, e);
                        continue;
                    }
                };

                let set_name = SetName::new(set)?;
                let card = ScryfallCard {
                    name: card_name,
                    set: set_name,
                    image_url: image_url,
                    prices: prices,
                };
                scryfall_card_list.push(card);
            }
        }
    }

    // Group scryfall cards by name
    let mut grouped_cards: HashMap<CardName, Vec<ScryfallCard>> = HashMap::new();
    for card in scryfall_card_list {
        grouped_cards
            .entry(card.name.clone())
            .or_insert_with(Vec::new)
            .push(card);
    }

    let parsed_file_path = format!(
        "scryfall_prices/parsed_scryfall_cards_{}.json",
        date_time_as_string(None, None)
    );
    // let grouped_cards_as_string = serde_json::to_string(&grouped_cards)?;

    // write_to_file(&parsed_file_path, &grouped_cards_as_string)?;
    save_to_json_file(&parsed_file_path, &grouped_cards)?;

    log::info!(
        "Parsed Scryfall data and saved to file {}",
        parsed_file_path
    );
    Ok(parsed_file_path)
}

fn is_not_artseries(obj: &Value) -> bool {
    obj["layout"] != "art_series"
}

fn is_not_basic_land(obj: &Value) -> bool {
    !obj["type_line"]
        .as_str()
        .unwrap_or("Not what we are looking for")
        .starts_with("Basic Land")
}

fn is_not_token(obj: &Value) -> bool {
    obj["layout"] != "token"
}

#[cfg(test)]
mod tests {

    use std::{fs, io::BufReader};

    use super::*;
    use env_logger;
    use fs::File;
    use mockito;
    use serde_json::json;
    use tempfile::tempdir;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[tokio::test]
    async fn test_download_scryfall_cards() {
        init();
        // Set up a mock server
        let mut server = std::thread::spawn(|| mockito::Server::new())
            .join()
            .unwrap();
        let url = server.url();

        // Create a temporary directory for the test
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path();

        // Mock the bulk data endpoint
        let _m = server
            .mock("GET", "/bulk-data")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "data": [
                        {},
                        {},
                        {"download_uri": url.clone() + "/all-cards"}
                    ]
                })
                .to_string(),
            )
            .create();

        // load_from_json_file::<String>("scryfall_card_resp.json").unwrap();
        let html_content = fs::read_to_string(
            "/workspaces/mtg-prz-rust/mtg-rust/src/scryfall/scryfall_card_resp.json",
        )
        .unwrap();
        // Mock the all-cards endpoint
        let _m2 = server
            .mock("GET", "/all-cards")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(html_content)
            .create();

        // Override the directory paths for the test
        std::env::set_var(
            "SCRYFALL_PRICES_RAW",
            temp_path.join("scryfall_prices_raw").to_str().unwrap(),
        );
        std::env::set_var(
            "SCRYFALL_PRICES",
            temp_path.join("scryfall_prices").to_str().unwrap(),
        );

        // Run the function
        let result = download_scryfall_cards(Some(url)).await;

        // Assert the result
        assert!(result.is_ok(), "{}", result.unwrap());

        // Read the parsed file
        let parsed_file_path = result.unwrap();
        let file = File::open(parsed_file_path).unwrap();
        let reader = BufReader::new(file);
        let parsed_cards: HashMap<CardName, Vec<ScryfallCard>> =
            serde_json::from_reader(reader).unwrap();

        // Assert the parsed data
        assert_eq!(parsed_cards.len(), 2); // one basic land, one token and 2 cards
        let kor_card = parsed_cards
            .get(&CardName::new("Kor Outfitter".to_string()).unwrap())
            .unwrap()
            .first()
            .unwrap();
        assert_eq!(kor_card.set.raw, "Zendikar");
        assert_eq!(kor_card.prices.eur, Some(0.19));
        assert_eq!(kor_card.prices.eur_foil, Some(1.83));
        assert_eq!(kor_card.image_url, "https://cards.scryfall.io/normal/front/0/0/00006596-1166-4a79-8443-ca9f82e6db4e.jpg?1562609251");
    }
}
