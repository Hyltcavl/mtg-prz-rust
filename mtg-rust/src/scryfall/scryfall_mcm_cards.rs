use regex::Regex;
use reqwest;
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

use crate::utils::date_to_string::date_time_as_string;
use crate::utils::file_management::write_to_file;
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
    println!("{}", price_files_available);
    let json: Value = serde_json::from_str(&price_files_available)?;
    // Get the latest price list uri of the smallest size
    let download_uri = json["data"][2]["download_uri"].as_str().unwrap();
    println!("Downloading 'smaller list' from URL: {:?}", download_uri);
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
        "scryfall_prices/{}_original_scryfall_prices_{}.json",
        prefix, current_time
    );

    write_to_file(&path, all_cards.as_str())?;

    return Ok(path);
}

#[derive(Debug, PartialEq, Clone, Serialize)]
struct Prices {
    eur: Option<f64>,
    eur_foil: Option<f64>,
}
#[derive(Debug, PartialEq, Clone, Serialize)]
struct ScryfallCard {
    name: String,
    set: String,
    image_url: String,
    prices: Prices,
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
            if obj["layout"] != "token" {
                let name = re
                    .replace_all(obj["name"].as_str().unwrap().to_lowercase().as_str(), "")
                    .to_string();
                let set = obj["set_name"].to_string();
                let prices = obj["prices"].clone();
                // println!("Error: Failed to get image_url for object: {:?}", obj);
                let prices = Prices {
                    eur: prices["eur"].as_f64(),
                    eur_foil: prices["eur_foil"].as_f64(),
                };
                let image_url = obj["image_uris"]["normal"]
                    .as_str()
                    .unwrap_or("https://www.google.com/url?sa=i&url=https%3A%2F%2Fanswers.microsoft.com%2Fen-us%2Fwindows%2Fforum%2Fall%2Fhigh-ram-usage-40-50-without-any-program%2F1dcf1e4d-f78e-4a06-a4e8-71f3972cc852&psig=AOvVaw0f3g3-hf1qnv6thWr6iQC2&ust=1724858067666000&source=images&cd=vfe&opi=89978449&ved=0CBQQjRxqFwoTCNjH-Ja7lYgDFQAAAAAdAAAAABAE")
                    .to_string();

                let card = ScryfallCard {
                    name: name,
                    set: set,
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
    Ok(parsed_file_path)
}

// const UNWANTED_PATTERNS: [&str; 4] = [
//     r"(?i)\(skadad\)",
//     r"(?i)\( Skadad \)",
//     r"(?i)\(Spelad\)",
//     r"(?i)\[Token\]",
// ];

// const FOIL_PATTERNS: [&str; 3] = [
//     r"(?i)\(Foil\)",
//     r"(?i)\(Etched Foil\)",
//     r"(?i)\(Foil Etched\)",
// ];

// #[derive(Debug, PartialEq, Clone, Serialize)]
// pub struct Card {
//     name: String,
//     foil: bool,
//     image_url: String,
//     extended_art: bool,
//     prerelease: bool,
//     showcase: bool,
//     set: String,
//     price: i32,
//     trade_in_price: i32,
//     current_stock: i8,
//     max_stock: i8,
// }

// fn create_regex_patterns(patterns: &[&str]) -> Result<Vec<Regex>, Box<dyn Error>> {
//     patterns
//         .iter()
//         .map(|&p| Regex::new(p).map_err(|e| Box::new(e) as Box<dyn Error>)) // Convert regex::Error to Box<dyn Error>
//         .collect()
// }
// fn parse_price(price_str: Option<&str>) -> i32 {
//     price_str
//         .and_then(|s| s.replace(" kr", "").trim().parse::<i32>().ok())
//         .unwrap_or(0)
// }

// fn get_price(tr_elements: scraper::ElementRef) -> Result<i32, Box<dyn Error>> {
//     let price_if_item_is_in_store = tr_elements
//         .select(&Selector::parse("td.align-right span.format-bold")?)
//         .next()
//         .map(|element| parse_price(Some(element.text().collect::<String>().as_str())));
//     let price_if_item_is_not_in_store = tr_elements
//         .select(&Selector::parse("td.align-right span.format-subtle")?)
//         .next()
//         .map(|element| parse_price(Some(element.text().collect::<String>().as_str())));
//     let price = price_if_item_is_in_store
//         .or(price_if_item_is_not_in_store)
//         .unwrap_or(0);
//     Ok(price)
// }

// pub async fn fetch_and_parse(url: &str) -> Result<Vec<Card>, Box<dyn Error>> {
//     let start = Instant::now();
//     let response = reqwest::get(url).await?;
//     println!("fetching {} took {:?} sec", url, start.elapsed().as_secs());
//     let html_content = response.text().await?;
//     let parse_document = Html::parse_document(&html_content);
//     let table_selector = Selector::parse("tr[id*='product-row-']")?;
//     let selected_elements = parse_document.select(&table_selector);

//     let unwanted_patterns = create_regex_patterns(&UNWANTED_PATTERNS)?;
//     let foil_patterns = create_regex_patterns(&FOIL_PATTERNS)?;

//     let mut results = Vec::new();

//     for tr_elements in selected_elements {
//         let name = tr_elements
//             .select(&Selector::parse("a.fancybox")?)
//             .next()
//             .map(|el| {
//                 el.text()
//                     .collect::<String>()
//                     .replace(
//                         "\n                                                                ",
//                         " ",
//                     )
//                     .trim()
//                     .to_string()
//             })
//             .unwrap_or_default();

//         if unwanted_patterns
//             .iter()
//             .any(|pattern| pattern.is_match(&name))
//         {
//             continue; // Skip unwanted cards
//         }

//         let foil = foil_patterns.iter().any(|pattern| pattern.is_match(&name));
//         let prerelease = Regex::new(r"(?i)\(Prerelease\)")?.is_match(&name);
//         let showcase = Regex::new(r"(?i)\(Showcase\)")?.is_match(&name);
//         let extended_art = Regex::new(r"(?i)\(Extended Art\)")?.is_match(&name);

//         let mut set = tr_elements
//             .select(&Selector::parse("img[title]")?)
//             .next()
//             .and_then(|attributes| attributes.value().attr("title").map(String::from))
//             .unwrap_or("UNKNOWN".to_string())
//             .replace(
//                 "\n                                                                ",
//                 " ",
//             )
//             .to_owned();

//         let other_set = tr_elements
//             .select(&Selector::parse("td.align-right a")?)
//             .next()
//             .map(|el| el.text().collect::<String>().trim().to_string())
//             .unwrap_or("UNKNOWN".to_string())
//             .replace(
//                 "\n                                                                ",
//                 " ",
//             )
//             .to_owned();

//         if set == "UNKNOWN" {
//             set = other_set;
//         }

//         let price = get_price(tr_elements)?;

//         let trade_in_price = tr_elements
//             .select(&Selector::parse("td.align-right")?)
//             .nth(2)
//             .and_then(|element| {
//                 Some(parse_price(Some(
//                     element.text().collect::<String>().as_str(),
//                 )))
//             });

//         let stock = tr_elements
//             .select(&Selector::parse("td.align-right")?)
//             .nth(3)
//             .and_then(|element| {
//                 let stock_str = element.text().collect::<String>();
//                 let stock_numbers: Vec<i8> = stock_str
//                     .replace("/", "")
//                     .replace("st", "")
//                     .trim()
//                     .split_whitespace()
//                     .filter_map(|s| s.parse::<i8>().ok())
//                     .collect();
//                 Some(stock_numbers)
//             })
//             .unwrap_or_else(|| vec![0, 0]); // Default to a vector of two zeros if parsing fails

//         let card = Card {
//             name,
//             foil,
//             image_url: "https://astraeus.dragonslair.se/images/4026/product".to_string(),
//             extended_art,
//             prerelease,
//             showcase,
//             set,
//             price,
//             trade_in_price: trade_in_price.unwrap_or(0),
//             current_stock: stock.first().unwrap_or(&0).to_owned(),
//             max_stock: stock.last().unwrap_or(&0).to_owned(),
//         };

//         results.push(card);
//     }

//     Ok(results)
// }

// #[cfg(test)]
// mod tests {
//     use std::fs;

//     use super::*;
//     use tokio;
//     // use mockito::{mock, Matcher};

//     #[tokio::test]
//     async fn test_fetch_and_parse() {
//         let html_content =
//             fs::read_to_string("/workspaces/mtg-prz-rust/product_search_page.html").unwrap();

//         let mut server = std::thread::spawn(|| mockito::Server::new())
//             .join()
//             .unwrap();
//         let url = server.url();
//         // Create a mock
//         let mock = server
//             .mock("GET", "/product/card-singles/magic?name=reaper+king")
//             .with_status(200)
//             .with_header("content-type", "text/plain")
//             .with_header("x-api-key", "1234")
//             .with_body(html_content.clone())
//             .create();

//         let result = fetch_and_parse(&format!(
//             "{}/product/card-singles/magic?name=reaper+king",
//             url
//         ))
//         .await
//         .unwrap();

//         mock.assert();
//         assert_eq!(result.len(), 8);

//         let regular_card = Card {
//             name: "Reaper King".to_string(),
//             foil: false,
//             image_url: "https://astraeus.dragonslair.se/images/4026/product".to_string(),
//             extended_art: false,
//             prerelease: false,
//             showcase: false,
//             set: "Shadowmoor".to_string(),
//             price: 100,
//             trade_in_price: 50,
//             current_stock: 1,
//             max_stock: 2,
//         };
//         assert_eq!(result.first().unwrap(), &regular_card);
//         assert_eq!(result.get(1).unwrap().foil, true);
//         assert_eq!(result.get(2).unwrap().foil, true);
//         assert_eq!(result.get(3).unwrap().foil, true);
//         assert_eq!(result.get(4).unwrap().prerelease, true);
//         assert_eq!(result.get(5).unwrap().showcase, true);
//         assert_eq!(result.get(6).unwrap().extended_art, true);
//         assert_eq!(
//             result.get(7).unwrap().set,
//             "Mystery Booster Retail Edition Foils"
//         );
//     }
// }
