use reqwest::header::{HeaderMap, HeaderValue};
use serde_json::Value;
use url::form_urlencoded;

// Define a custom error type for our module
#[derive(Debug)]
pub enum MtgPriceError {
    RequestError(reqwest::Error),
    NoCardFound,
    BadApiResponse(String),
}

// Implement conversions from other error types to our custom error type
impl From<reqwest::Error> for MtgPriceError {
    fn from(err: reqwest::Error) -> Self {
        MtgPriceError::RequestError(err)
    }
}

impl From<serde_json::Error> for MtgPriceError {
    fn from(err: serde_json::Error) -> Self {
        MtgPriceError::JsonError(err)
    }
}

pub struct MtgStocksCard {
    pub set: String,
    pub price: f64,
}

// Main function to get live card prices
pub fn get_live_card_prices(card_name: &str) -> Result<Vec<MtgStocksCard>, MtgPriceError> {
    log::info!("Featching live prices for card: {}", card_name);
    let slug = get_card_search_uri(card_name)?;
    let list = get_list_of_prices_for_card(&slug)?;
    Ok(list)
}

// Function to get the card search URI
fn get_card_search_uri(card_name: &str) -> Result<String, MtgPriceError> {
    // URL encode the card name
    let card_name_utf8 = form_urlencoded::byte_serialize(card_name.as_bytes()).collect::<String>();
    let url = format!(
        "https://api.mtgstocks.com/search/autocomplete/{}",
        card_name_utf8
    );

    let client = reqwest::blocking::Client::new();

    let response = client.get(&url).headers(get_headers()).send()?;

    if response.status().is_success() {
        let response = response.text()?;
        let data: Vec<Value> = serde_json::from_str(&response)?;
        // Filter out objects with "Token" in the name
        let token_objects: Vec<&Value> = data
            .iter()
            .filter(|obj| !obj["name"].as_str().unwrap_or("").contains("Token"))
            .collect();

        if !token_objects.is_empty() {
            return Ok(token_objects[0]["slug"].as_str().unwrap().to_string());
        }
    }

    Err(MtgPriceError::NoCardFound)
}

// Function to get the list of prices for a card
fn get_list_of_prices_for_card(slug: &str) -> Result<Vec<MtgStocksCard>, MtgPriceError> {
    let slug_utf8 = form_urlencoded::byte_serialize(slug.as_bytes()).collect::<String>();
    let url = format!("https://api.mtgstocks.com/prints/{}", slug_utf8);

    let client = reqwest::blocking::Client::new();
    let response = client.get(&url).headers(get_headers()).send()?;

    if response.status().is_success() {
        let response = response.text()?;
        let data: Value = serde_json::from_str(&response)?;
        let mut cardlist = Vec::new();

        // Add the main card data
        cardlist.push(MtgStocksCard {
            set: data["card_set"]["name"].to_string(),
            price: data["latest_price_mkm"]["avg"].as_f64().unwrap(),
        });

        // Add data for all sets
        let mut prices: Vec<MtgStocksCard> = data["sets"]
            .as_array()
            .unwrap()
            .iter()
            .map(|obj| MtgStocksCard {
                set: obj["set_name"].to_string(),
                price: obj["latest_price_mkm"].as_f64().unwrap_or(1000.0),
            })
            .collect();

        prices.extend(cardlist);
        Ok(prices)
    } else {
        Err(MtgPriceError::BadApiResponse(response.text()?))
    }
}

// Function to get the headers for the HTTP requests
fn get_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("accept", HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7"));
    headers.insert(
        "accept-language",
        HeaderValue::from_static("en-US,en;q=0.9,sv-SE;q=0.8,sv;q=0.7,nb;q=0.6"),
    );
    headers.insert("User-Agent", HeaderValue::from_static("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_10_1) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/39.0.2171.95 Safari/537.36"));
    headers
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_card_search_uri() {
        let card_name = "Giant Growth";
        match get_card_search_uri(card_name) {
            Ok(slug) => assert!(!slug.is_empty()),
            Err(err) => match err {
                MtgPriceError::NoCardFound => assert!(false, "No card found"),
                _ => assert!(false, "Unexpected error: {:?}", err),
            },
        }
    }

    #[test]
    fn test_get_list_of_prices_for_card() {
        // let headers = HeaderMap::new();
        // reqwest::Client::builder()
        //     .default_headers(headers)
        //     .build()
        //     .unwrap();

        let slug = "16455-giant-growth"; // Replace with a valid slug
        match get_list_of_prices_for_card(slug) {
            Ok(prices) => assert!(!prices.is_empty()),
            Err(err) => match err {
                MtgPriceError::BadApiResponse(_) => assert!(false, "Bad API response"),
                _ => assert!(false, "Unexpected error: {:?}", err),
            },
        }
    }
}
