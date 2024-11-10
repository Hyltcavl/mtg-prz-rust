use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Error, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use url::form_urlencoded;

use super::string_manipulators::clean_string;

#[derive(Debug)]
pub enum MtgPriceError {
    RequestError(reqwest::Error),
    // NoCardFound,
    // BadApiResponse(String),
    OtherError(serde_json::Error),
}

impl From<reqwest::Error> for MtgPriceError {
    fn from(err: reqwest::Error) -> Self {
        MtgPriceError::RequestError(err)
    }
}

impl From<serde_json::Error> for MtgPriceError {
    fn from(err: serde_json::Error) -> Self {
        MtgPriceError::OtherError(err)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MtgStocksCard {
    pub set: String,
    pub price: f64,
}

pub struct MtgPriceFetcher {
    client: Client,
    cache: Arc<Mutex<HashMap<String, Vec<MtgStocksCard>>>>,
}

impl MtgPriceFetcher {
    pub fn new(client: Client) -> Self {
        MtgPriceFetcher {
            client,
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn get_live_card_prices(
        &self,
        card_name: &str,
        base_url: &str,
    ) -> Result<Vec<MtgStocksCard>, Box<dyn std::error::Error>> {
        log::info!("Fetching live prices for card: {}", card_name);

        // Check cache first
        let cached_prices = {
            let cache = self.cache.lock().await;
            cache.get(card_name).cloned()
        };

        if let Some(prices) = cached_prices {
            log::debug!("Returning cached prices for card: {:?}", prices);
            return Ok(prices);
        }
        let slug = self.get_card_search_uri(card_name).await?;
        let list = self.get_list_of_prices_for_card(&slug, base_url).await?;

        // Update cache
        {
            let mut cache = self.cache.lock().await;
            cache.insert(card_name.to_string(), list.clone());
        }

        log::debug!("Fetched live prices for card: {:?}", list);
        Ok(list)
    }

    async fn get_card_search_uri(
        &self,
        card_name: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let card_name_utf8 =
            form_urlencoded::byte_serialize(card_name.as_bytes()).collect::<String>();
        let url = format!(
            "https://api.mtgstocks.com/search/autocomplete/{}",
            card_name_utf8
        );

        let response = self
            .client
            .get(&url)
            .headers(self.get_headers())
            .send()
            .await?;

        if response.status().is_success() {
            let response = response.text().await?;
            let data: Vec<Value> = serde_json::from_str(&response)?;
            let token_objects: Vec<&Value> = data
                .iter()
                .filter(|obj| !obj["name"].as_str().unwrap_or("").contains("Token"))
                .collect();

            if !token_objects.is_empty() {
                return Ok(token_objects[0]["slug"].as_str().unwrap().to_string());
            }
        }

        Err("Card not found on MTGStocks".into())
    }

    async fn get_list_of_prices_for_card(
        &self,
        slug: &str,
        base_url: &str,
    ) -> Result<Vec<MtgStocksCard>, Box<dyn std::error::Error>> {
        let slug_utf8 = form_urlencoded::byte_serialize(slug.as_bytes()).collect::<String>();
        let url = format!("{}/{}", base_url, slug_utf8);

        let response = self
            .client
            .get(&url)
            .headers(self.get_headers())
            .send()
            .await?;

        if response.status().is_success() {
            let response = response.text().await?;
            let data: Value = serde_json::from_str(&response)?;
            let mut cardlist = Vec::new();

            // Add the main card data
            cardlist.push(MtgStocksCard {
                set: clean_string(&data["card_set"]["name"].to_string()),
                price: data["latest_price_mkm"]["avg"].as_f64().unwrap_or(100.0),
            });

            // Add data for all sets
            let mut prices: Vec<MtgStocksCard> = data["sets"]
                .as_array()
                .unwrap()
                .iter()
                .map(|obj| MtgStocksCard {
                    set: clean_string(&obj["set_name"].to_string()),
                    price: obj["latest_price_mkm"].as_f64().unwrap_or(1000.0),
                })
                .collect();

            prices.extend(cardlist);
            Ok(prices)
        } else {
            Err(response.text().await?.into())
        }
    }

    fn get_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("accept", HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7"));
        headers.insert(
            "accept-language",
            HeaderValue::from_static("en-US,en;q=0.9,sv-SE;q=0.8,sv;q=0.7,nb;q=0.6"),
        );
        headers.insert("User-Agent", HeaderValue::from_static("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_10_1) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/39.0.2171.95 Safari/537.36"));
        headers.insert("cache-control", HeaderValue::from_static("max-age=0"));
        headers.insert("priority", HeaderValue::from_static("u=0, i"));
        headers.insert(
            "sec-ch-ua",
            HeaderValue::from_static(
                "\"Chromium\";v=\"123\", \"Not;A=Brand\";v=\"24\", \"Google Chrome\";v=\"123\"",
            ),
        );
        headers.insert("sec-ch-ua-mobile", HeaderValue::from_static("?0"));
        headers.insert(
            "sec-ch-ua-platform",
            HeaderValue::from_static("\"Windows\""),
        );
        headers.insert("sec-fetch-dest", HeaderValue::from_static("document"));
        headers.insert("sec-fetch-mode", HeaderValue::from_static("navigate"));
        headers.insert("sec-fetch-site", HeaderValue::from_static("none"));
        headers.insert("sec-fetch-user", HeaderValue::from_static("?1"));
        headers.insert("upgrade-insecure-requests", HeaderValue::from_static("1"));
        headers.insert("credentials", HeaderValue::from_static("omit"));
        headers
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito;
    use tokio;

    #[tokio::test]
    async fn test_get_card_search_uri() {
        let client = Client::new();
        let fetcher = MtgPriceFetcher::new(client);
        let card_name = "Giant Growth";
        match fetcher.get_card_search_uri(card_name).await {
            Ok(slug) => assert!(!slug.is_empty()),
            Err(err) => panic!("Unexpected error: {:?}", err),
        }
    }

    #[tokio::test]
    async fn test_get_list_of_prices_for_card() {
        let mut server = std::thread::spawn(|| mockito::Server::new())
            .join()
            .unwrap();
        let mock_url = server.url();

        // Create a mock response
        let mock_response = r#"
        {
            "card_set": {"name": "Alpha"},
            "latest_price_mkm": {"avg": 10.5},
            "sets": [
                {"set_name": "Beta", "latest_price_mkm": 11.0},
                {"set_name": "Unlimited", "latest_price_mkm": 9.5}
            ]
        }"#;

        let mock = server
            .mock("GET", "/prints/16455-giant-growth")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response)
            .create();

        let client = reqwest::Client::builder().build().unwrap();

        let fetcher = MtgPriceFetcher::new(client);

        // Override the base URL for testing
        let test_url = format!("{}/prints", mock_url);
        let prices = fetcher
            .get_list_of_prices_for_card("16455-giant-growth", &test_url)
            .await;

        match prices {
            Ok(prices) => {
                assert!(!prices.is_empty());
                assert_eq!(prices.len(), 3); // One for Alpha and two from the sets array
                assert_eq!(prices[0].set, "Beta");
                assert_eq!(prices[0].price, 11.0);
                assert_eq!(prices[1].set, "Unlimited");
                assert_eq!(prices[1].price, 9.5);
            }
            Err(err) => panic!("Unexpected error: {:?}", err),
        }

        mock.assert();
    }
}
