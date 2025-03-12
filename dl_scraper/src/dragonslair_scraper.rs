use crate::cards::card_parser::fetch_and_parse;
use crate::cards::cardname::CardName;
use crate::cards::vendorcard::VendorCard;
use crate::utilities::string_manipulators::date_time_as_string;
use futures::stream::{self, StreamExt};
use log::info;
use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::{error::Error, time::Instant};

pub struct DragonslairScraper {
    url: String,
    client: Client,
    cmcs_available: Vec<u8>,
}

// fn get_page_count(&self, url: &str) -> Result<usize, Box<dyn Error>> {
//     let res = self.client.get(url).send()?.text()?;
//     let document = Html::parse_document(&res);
//     let selector = Selector::parse(".page-count-class").unwrap(); // Adjust the selector as needed

//     let page_count = document
//         .select(&selector)
//         .next()
//         .and_then(|element| element.text().collect::<Vec<_>>().join("").parse().ok())
//         .unwrap_or(1);

//     Ok(page_count)
// }

impl DragonslairScraper {
    pub fn new(url: &str, cmcs_available: Option<Vec<u8>>, client: Client) -> Self {
        DragonslairScraper {
            url: url.to_string(),
            client: client,
            cmcs_available: cmcs_available
                .unwrap_or_else(|| vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 15, 16]),
        }
    }

    /// Get's the page count of the CMC pages. Defaults to 1 if no additional pages exist.
    async fn get_page_count(&self, url: &str) -> Option<u32> {
        let response = self.client.get(url).send().await.ok()?;
        let html_content = response.text().await.ok()?;

        let parse_document = Html::parse_document(&html_content);
        let table_selector = Selector::parse("div.container.align-center.pagination a").ok()?;
        let selected_elements: Vec<_> = parse_document.select(&table_selector).collect();

        if selected_elements.len() >= 3 {
            let last_page_link = &selected_elements[selected_elements.len() - 3];
            last_page_link.inner_html().parse::<u32>().ok()
        } else {
            Some(1) // Assuming there's at least one page if no pagination is found
        }
    }

    fn generate_card_urls(&self, page_count: u32, cmc: u8) -> Vec<String> {
        (1..=page_count)
            .map(|count| {
                format!(
                    "{}/product/magic/card-singles/store:kungsholmstorg/cmc-{}/{}",
                    self.url, cmc, count
                )
            })
            .collect::<Vec<String>>()
    }

    async fn get_card_urls(&self) -> Result<Vec<String>, Box<dyn Error>> {
        let start_time = Instant::now();

        let card_urls = stream::iter(&self.cmcs_available)
            .map(|cmc| {
                let request_url = format!(
                    "{}/product/magic/card-singles/store:kungsholmstorg/cmc-{}/{}",
                    self.url, cmc, 0
                );
                async move {
                    match self.get_page_count(&request_url).await {
                        Some(page_count) => self.generate_card_urls(page_count, *cmc),
                        None => {
                            log::error!("Failed to get page count on request {:?}", request_url);
                            Vec::new()
                        }
                    }
                }
            })
            .buffered(20) // Limit to 20 concurrent requests
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .flatten()
            .collect::<Vec<String>>();

        info!(
            "Fetching urls to fetch took {:?} sec",
            start_time.elapsed().as_secs()
        );
        Ok(card_urls)
    }

    pub async fn get_available_cards(
        &self,
    ) -> Result<HashMap<CardName, Vec<VendorCard>>, Box<dyn Error>> {
        let card_urls = self.get_card_urls().await?;
        let cards = self.fetch_cards(card_urls).await?;
        Ok(self.group_cards(&cards))
    }

    fn group_cards(&self, cards: &Vec<VendorCard>) -> HashMap<CardName, Vec<VendorCard>> {
        let mut grouped_cards: HashMap<CardName, Vec<VendorCard>> = HashMap::new();
        for card in cards {
            grouped_cards
                .entry(card.name.clone())
                .or_insert_with(Vec::new)
                .push(card.clone());
        }
        grouped_cards
    }

    async fn fetch_cards(&self, urls: Vec<String>) -> Result<Vec<VendorCard>, Box<dyn Error>> {
        let card_urls = stream::iter(urls)
            .map(|url| async move {
                match fetch_and_parse(&url).await {
                    Ok(cards) => cards,
                    Err(e) => {
                        log::error!("Error fetching cards from {}: {}", &url, e);
                        Vec::new()
                    }
                }
            })
            .buffered(20) // Limit to 20 concurrent requests
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .flatten()
            .collect::<Vec<VendorCard>>();
        Ok(card_urls)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::to_string;
    use std::fs;
    use tokio;

    #[tokio::test]
    async fn test_get_page_count() {
        let html_content = include_str!("test/get_pages_page.html").to_string();

        let mut server = std::thread::spawn(|| mockito::Server::new())
            .join()
            .unwrap();
        let url = server.url();
        let mock = server
            .mock(
                "GET",
                "/product/magic/card-singles/store:kungsholmstorg/cmc-0/1",
            )
            .with_status(200)
            .with_header("content-type", "text/plain")
            .with_header("x-api-key", "1234")
            .with_body(html_content)
            .create();
        let url = format!(
            "{}/product/magic/card-singles/store:kungsholmstorg/cmc-0/1",
            url
        );

        let res = DragonslairScraper::new(&url, None, reqwest::Client::new())
            .get_page_count(&url)
            .await
            .unwrap();

        mock.assert();
        assert_eq!(res, 51);
    }

    #[tokio::test]
    async fn test_get_page_count_no_pages() {
        let html_content = fs::read_to_string(
            "/workspaces/mtg-prz-rust/mtg-rust/src/test/get_pages_no_pages.html",
        )
        .unwrap();
        let mut server = std::thread::spawn(|| mockito::Server::new())
            .join()
            .unwrap();
        let url = server.url();
        let mock = server
            .mock(
                "GET",
                "/product/magic/card-singles/store:kungsholmstorg/cmc-15/1",
            )
            .with_status(200)
            .with_header("content-type", "text/plain")
            .with_header("x-api-key", "1234")
            .with_body(html_content)
            .create();
        let url = format!(
            "{}/product/magic/card-singles/store:kungsholmstorg/cmc-15/1",
            url
        );
        let res = DragonslairScraper::new(&url, None, reqwest::Client::new())
            .get_page_count(&url)
            .await
            .unwrap();

        mock.assert();
        assert_eq!(res, 1);
    }
}
