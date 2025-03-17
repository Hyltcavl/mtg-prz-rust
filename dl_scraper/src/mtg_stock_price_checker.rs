use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use url::form_urlencoded;

use crate::cards::cardname::CardName;
use crate::cards::currency::Currency;
use crate::cards::price::Price;
use crate::cards::setname::SetName;

#[derive(Clone)]
pub struct MtgPriceFetcher {
    client: Client,
    base_url: String,
    cache: Arc<Mutex<HashMap<CardName, Vec<MtgStocksCard>>>>,
    // cache: Arc<RwLock<HashMap<CardName, Price>>>,
}

impl MtgPriceFetcher {
    pub fn new(client: Client, base_url: String) -> Self {
        MtgPriceFetcher {
            client,
            base_url,
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn get_live_card_price(
        &self,
        card_name: CardName,
        card_set: SetName,
    ) -> Result<Price, Box<dyn std::error::Error>> {
        log::info!("Fetching live prices for card: {}", card_name.almost_raw);

        // // Check cache first
        let cached_prices: Option<Price> = {
            let cache = self.cache.lock().await;
            let prices = cache.get(&card_name).cloned();
            match prices {
                Some(prices) => {
                    let price = prices.iter().find(|card| card.set == card_set);
                    match price {
                        Some(card) => return Ok(card.price),
                        None => None,
                    }
                }
                None => None,
            }
        };

        if let Some(price) = cached_prices {
            log::debug!("Returning cached prices for card: {:?}", price);
            return Ok(price);
        }

        let slug = self.get_card_search_uri(&card_name.almost_raw).await?;
        let list = self.get_list_of_prices_for_card(&slug).await?;
        log::debug!("Fetched live prices for card: {:?}", list);

        let price = list.iter().find(|card| card.set == card_set);

        // Update cache
        let mut cache = self.cache.lock().await;
        cache.insert(card_name.clone(), list.clone());

        match price {
            Some(card) => Ok(card.price),
            None => Err("Card not found in the set".into()),
        }
    }

    // pub async fn get_live_card_prices(
    //     &self,
    //     card_name: &str,
    //     base_url: &str,
    // ) -> Result<Vec<MtgStocksCard>, Box<dyn std::error::Error>> {
    //     log::info!("Fetching live prices for card: {}", card_name);

    //     // Check cache first
    //     let cached_prices = {
    //         let cache = self.cache.lock().await;
    //         cache.get(card_name).cloned()
    //     };

    //     if let Some(prices) = cached_prices {
    //         log::debug!("Returning cached prices for card: {:?}", prices);
    //         return Ok(prices);
    //     }
    //     let slug = self.get_card_search_uri(card_name).await?;
    //     let list = self.get_list_of_prices_for_card(&slug).await?;

    //     // Update cache
    //     {
    //         let mut cache = self.cache.lock().await;
    //         cache.insert(card_name.to_string(), list.clone());
    //     }

    //     log::debug!("Fetched live prices for card: {:?}", list);
    //     Ok(list)
    // }

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

        // https://api.mtgstocks.com/search/autocomplete/16455-giant-growth

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
    ) -> Result<Vec<MtgStocksCard>, Box<dyn std::error::Error>> {
        let slug_utf8 = form_urlencoded::byte_serialize(slug.as_bytes()).collect::<String>();
        let url = format!("{}/prints/{}", self.base_url, slug_utf8);

        let response = self
            .client
            .get(&url)
            .headers(self.get_headers())
            .send()
            .await?;

        if response.status().is_success() {
            let response = response.text().await?;
            let data: Value = serde_json::from_str(&response)?;
            // let mut cardlist = Vec::new();

            // Add the main card data
            // cardlist.push(MtgStocksCard {
            //     set: SetName::new(data["card_set"]["name"].to_string()).unwrap(),
            //     price: Price::new(
            //         match data["latest_price_mkm"]["avg"].as_f64() {
            //             Some(price) => price,
            //             None => {
            //                 log::warn!("latest_price_mkm avg not found, using default value of 1000.0");
            //                 1000.0
            //             }
            //         },
            //         Currency::EUR,
            //     ),
            // });

            // Add data for all sets
            let prices: Vec<MtgStocksCard> = data["sets"]
                .as_array()
                .unwrap()
                .iter()
                .map(|obj| MtgStocksCard {
                    set: SetName::new(obj["set_name"].to_string()).unwrap(),
                    price: Price::new(
                        match obj["latest_price_mkm"].as_f64() {
                            Some(price) => price,
                            None => {
                                log::warn!(
                                    "latest_price_mkm avg not found for card: {} in set: {}, using default value of 0.0", slug, obj["set_name"]
                                );
                                0.0
                            }
                        },
                        Currency::EUR,
                    ),
                })
                .collect();

            // prices.extend(cardlist);
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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MtgStocksCard {
    set: SetName,
    pub price: Price,
}

#[cfg(test)]
mod tests {
    use crate::utilities::constants::MTG_STOCKS_BASE_URL;

    use super::*;
    use mockito;
    use tokio;

    #[tokio::test]
    async fn test_get_card_search_uri() {
        let client = Client::new();
        let fetcher = MtgPriceFetcher::new(client, MTG_STOCKS_BASE_URL.to_string());
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
                {"set_name": "Unlimited", "latest_price_mkm": 9.5},
                {"set_name": "Alpha Edition", "latest_price_mkm": 0.11}
            ]
        }"#;

        let mock = server
            .mock("GET", "/prints/16455-giant-growth")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response)
            .create();

        let client = reqwest::Client::builder().build().unwrap();

        // Override the base URL for testing
        // let test_url = format!("{}/prints", mock_url);
        let fetcher = MtgPriceFetcher::new(client, mock_url);

        let prices = fetcher
            .get_list_of_prices_for_card("16455-giant-growth")
            .await;

        mock.assert();

        match prices {
            Ok(prices) => {
                assert!(!prices.is_empty());
                assert_eq!(prices.len(), 3); // One for Alpha and two from the sets array
                assert_eq!(prices[0].set, SetName::new("Beta".to_string()).unwrap());
                assert_eq!(prices[0].price, Price::new(11.0, Currency::EUR));
                assert_eq!(
                    prices[1].set,
                    SetName::new("Unlimited".to_string()).unwrap()
                );
                assert_eq!(prices[1].price, Price::new(9.5, Currency::EUR));
            }
            Err(err) => panic!("Unexpected error: {:?}", err),
        }
    }
}
