use log;
use regex::Regex;
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

use crate::utils::file_management::write_to_file;
use crate::utils::string_manipulators::{clean_word, date_time_as_string};
/// Returns path to the downloaded scryfall price file.
/// Defalts to no prefix if None is given
async fn get_scryfall_price_file(
    mut prefix: Option<String>,
) -> Result<String, Box<dyn std::error::Error>> {
    let prefix: String = prefix.unwrap_or("".to_string());
    // Get available scryfall price files list
    let url = "https://api.scryfall.com/bulk-data";
    let client = reqwest::Client::new();
    // reqwest::async_impl::request::RequestBuilder

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

    // let price_files_available = reqwest::get(url).await?.text().await?;
    log::debug!("{}", price_files_available);
    let json: Value = serde_json::from_str(&price_files_available)?;
    // Get the latest price list uri of the smallest size
    let download_uri = json["data"][2]["download_uri"].as_str().unwrap();
    log::debug!("Downloading 'smaller list' from URL: {:?}", download_uri);
    let all_cards = client
        .get(download_uri)
        .headers(header_map)
        .send()
        .await?
        .text()
        .await?;

    // Write the downloaded scryfall price file to a local json file
    let current_time = date_time_as_string(None, None);
    let path = format!(
        "scryfall_prices_raw/scryfall_download_{}.json",
        current_time
    );

    write_to_file(&path, all_cards.as_str())?;

    return Ok(path);
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Prices {
    pub eur: Option<f64>,
    pub eur_foil: Option<f64>,
}
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ScryfallCard {
    pub name: String,
    pub set: String,
    pub image_url: String,
    pub prices: Prices,
}
/// Returns path to the scryfall cards file.
pub async fn download_scryfall_cards() -> Result<String, Box<dyn std::error::Error>> {
    let path = get_scryfall_price_file(None).await?;
    // let path = "/workspaces/mtg-prz-rust/mtg-rust/scryfall_prices/_original_scryfall_prices_27_08_2024-15:04.json".to_owned();
    let mut scryfall_card_list = Vec::new();
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let cards: Value = serde_json::from_reader(reader)?;

    let re = Regex::new(r"[^a-zA-Z]").unwrap();

    if let Value::Array(cards_array) = cards {
        for obj in cards_array {
            if is_not_token(&obj) || is_not_basic_land(&obj) || is_not_artseries(&obj) {
                let name = re
                    .replace_all(obj["name"].as_str().unwrap().to_lowercase().as_str(), "")
                    .to_string();
                let set = obj["set_name"].to_string();
                let prices = obj["prices"].clone();

                let prices = Prices {
                    eur: prices["eur"].as_f64(),
                    eur_foil: prices["eur_foil"].as_f64(),
                };
                let image_url = obj["image_uris"]["normal"]
                    .as_str()
                    .unwrap_or("https://www.google.com/url?sa=i&url=https%3A%2F%2Fanswers.microsoft.com%2Fen-us%2Fwindows%2Fforum%2Fall%2Fhigh-ram-usage-40-50-without-any-program%2F1dcf1e4d-f78e-4a06-a4e8-71f3972cc852&psig=AOvVaw0f3g3-hf1qnv6thWr6iQC2&ust=1724858067666000&source=images&cd=vfe&opi=89978449&ved=0CBQQjRxqFwoTCNjH-Ja7lYgDFQAAAAAdAAAAABAE")
                    .to_string();

                let card = ScryfallCard {
                    name: clean_word(&name),
                    set: clean_word(&set),
                    image_url: image_url,
                    prices: prices,
                };
                scryfall_card_list.push(card);
            }
        }
    }

    // Group scryfall cards by name
    let mut grouped_cards: HashMap<String, Vec<ScryfallCard>> = HashMap::new();
    for card in scryfall_card_list {
        // grouped_cards.entry(key)
        grouped_cards
            .entry(card.name.to_string())
            .or_insert_with(Vec::new)
            .push(card.clone());
    }

    let parsed_file_path = format!(
        "scryfall_prices/parsed_scryfall_cards_{}.json",
        date_time_as_string(None, None)
    );
    let grouped_cards_as_string = serde_json::to_string(&grouped_cards)?;
    write_to_file(&parsed_file_path, &grouped_cards_as_string)?;
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
