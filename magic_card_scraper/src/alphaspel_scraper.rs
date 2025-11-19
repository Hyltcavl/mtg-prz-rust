use std::{collections::HashMap, error::Error};

use futures::{stream, StreamExt};
use log::{debug, error, info, warn};
use regex::Regex;
use scraper::{Html, Selector};

use crate::cards::{
    cardname::CardName, currency::Currency, price::Price, setname::SetName, vendor::Vendor,
    vendorcard::VendorCard,
};

#[derive(Debug)]
pub struct AlphaspelScraper {
    promo_patterns: Vec<Regex>,
    base_url: String,
}

impl AlphaspelScraper {
    pub fn new(base_url: &str) -> Self {
        let patterns = [
            r"(?i)\(Promo\)",
            r"(?i)\(promo\)",
            r"(?i)\(Prerelease\)",
            r"(?i)\(prerelease\)",
        ];

        let promo_patterns = patterns
            .iter()
            .map(|&p| Regex::new(p).map_err(|e| Box::new(e) as Box<dyn Error>))
            .collect::<Result<Vec<Regex>, Box<dyn Error>>>()
            .unwrap();

        Self {
            promo_patterns,
            base_url: base_url.to_string(),
        }
    }

    async fn get_all_card_pages(&self) -> Result<Vec<(String, String)>, Box<dyn Error>> {
        let url = format!("{}/1978-mtg-loskort/", self.base_url);
        info!("Fetching all card pages from: {}", url);
        let client = reqwest::Client::new();
        let mut header_map = reqwest::header::HeaderMap::new();
        header_map.insert(reqwest::header::ACCEPT, "*/*".parse().unwrap());
        header_map.insert(
            reqwest::header::USER_AGENT,
            "application/json".parse().unwrap(),
        );
        let sets_page = client
            .get(url)
            .headers(header_map.clone())
            .send()
            .await?
            .text()
            .await?;

        // info!("{}", sets_page);
        let document = Html::parse_document(&sets_page);
        // let selector = Selector::parse(".nav.nav-list a").unwrap();
        let selector = Selector::parse(".categories.row h4.text-center a").unwrap();

        let sets_links: Vec<(String, String)> = document
            .select(&selector)
            .filter_map(|element| {
                let href = element.value().attr("href")?.to_string();
                let text = element.text().collect::<Vec<_>>().join(" ");
                Some((href, text))
            })
            // .map(|(href, content)| href.to_string(),)
            .collect();

        Ok(sets_links)
    }

    fn get_card_from_html(
        &self,
        card_elements: scraper::ElementRef,
        list_of_sets: Vec<String>,
    ) -> Result<VendorCard, Box<dyn Error>> {
        let in_stock = card_elements
            .select(&Selector::parse(".stock").unwrap())
            .next()
            .ok_or("No stock information found")?
            .text()
            .collect::<String>()
            .trim()
            .to_string();

        let stock = if in_stock == "Slutsåld" {
            return Err("Card is Slutsåld".into());
        } else {
            let cleaned_stock = in_stock.replace("i butiken", "").trim().to_string();
            cleaned_stock
                .parse::<i8>()
                .map_err(|e| format!("Failed to parse stock '{}': {}", cleaned_stock, e))?
        };

        let image_url: String = card_elements
            .select(&Selector::parse("img.img-responsive.center-block").unwrap())
            .filter_map(|element| element.value().attr("src"))
            .map(|href| href.to_string())
            .collect();

        let image_url = format!("{}{}", self.base_url, image_url.replace("\n", "").trim());

        let product_name = card_elements
            .select(&Selector::parse(".product-name").unwrap())
            .next()
            .ok_or("No product name found")?
            .text()
            .collect::<String>();

        let product_name = product_name
            .replace("\n", "")
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ");

        // let promo_patterns = create_regex_patterns(&PROMO_PATTERNS)?;
        let prerelease = self
            .promo_patterns
            .iter()
            .any(|pattern| pattern.is_match(&product_name));

        let alternative_art = product_name.contains("(alternative art)");

        if product_name.to_lowercase().contains("(italiensk)")
            || product_name.to_lowercase().contains("(tysk)")
            || product_name.to_lowercase().contains("(rysk)")
        {
            return Err("Card is not english".into());
        }

        if Regex::new(r"Token")?.is_match(&product_name) {
            return Err("Card is a token".into());
        }

        let set = list_of_sets
            .into_iter()
            .find(|set| product_name.to_lowercase().contains(&set.to_lowercase()));

        let set = if set.is_some() {
            set.unwrap()
        } else {
            return Err(format!("Unable to find what set {} belongs to", &product_name).into());
        };

        let raw_name = &product_name
            .split_once(&set)
            .map(|(_, after)| after)
            .unwrap_or("Error retrieving the name".into())
            .replace(":", "");

        let price = card_elements
            .select(&Selector::parse(".price.text-success").unwrap())
            .next()
            .ok_or("No price found")?
            .text()
            .collect::<String>();

        let price: f64 = Regex::new(r"\d+")?
            .find(&price)
            .ok_or_else(|| format!("No numeric value found in price string: '{}'", price))?
            .as_str()
            .parse()
            .map_err(|e| format!("Failed to parse price '{}': {}", price, e))?;

        let price = Price::new(price, Currency::SEK);

        let foil = Regex::new(r"\(Foil\)")?.is_match(raw_name)
            || Regex::new(r"\(Etched Foil\)")?.is_match(raw_name)
            || Regex::new(r"\(Foil Etched\)")?.is_match(raw_name);

        let name = CardName::new(raw_name.to_owned())?;
        let set = SetName::new(set)?;

        Ok(VendorCard {
            name,
            vendor: Vendor::Alphaspel,
            foil,
            image_url: image_url,
            extended_art: alternative_art,
            prerelease: prerelease,
            showcase: false,
            set,
            price,
            trade_in_price: 0,
            current_stock: stock,
            max_stock: 3,
            collector_number: None,
        })
    }

    pub async fn scrape_cards(&self) -> Result<HashMap<CardName, Vec<VendorCard>>, Box<dyn Error>> {
        let pages_and_set_names = self.get_all_card_pages().await?;
        let (pages, set_names): (Vec<_>, Vec<_>) = pages_and_set_names.into_iter().unzip();
        info!("Found {} alphaspel set pages", pages.len());

        //Get all pages to call
        let links_to_call = stream::iter(pages)
            .map(|set_href| {
                let link = format!(
                    "{}{}?order_by=stock_a&ordering=desc&page=1",
                    self.base_url, set_href
                );
                debug!("Processing link {}", &link);
                async move {
                    match reqwest::get(&link).await {
                        Ok(response) => match response.text().await {
                            Ok(set_initial_page) => {
                                let document = Html::parse_document(&set_initial_page);
                                let selector = Selector::parse("ul.pagination li").unwrap();

                                let mut max_page = 1;
                                for element in document.select(&selector) {
                                    if let Ok(num) =
                                        element.text().collect::<String>().trim().parse::<u32>()
                                    {
                                        if num > max_page {
                                            max_page = num;
                                        }
                                    }
                                }
                                (set_href, max_page)
                            }
                            Err(e) => {
                                error!("Error reading response text for {}: {}", link, e);
                                (set_href, 1)
                            }
                        },
                        Err(e) => {
                            error!("Error fetching page {}: {}", link, e);
                            (set_href, 1)
                        }
                    }
                }
            })
            .buffered(40)
            .collect::<Vec<_>>()
            .await;
        let cards: Vec<VendorCard> = stream::iter(links_to_call)
            .map(|(set_href, max_page_count)| {
                let set_names_clone = set_names.clone();

                async move {
                    let mut cards = Vec::new();
                    // let value = set_names.clone();

                    for page_count in 1..=max_page_count as i32 {
                        let link = format!(
                            "{}{}?order_by=stock_a&ordering=desc&page={}",
                            self.base_url, set_href, page_count
                        );
                        info!("Fetching cards from {}", &link);
                        match reqwest::get(&link).await {
                            Ok(response) => match response.text().await {
                                Ok(set_page) => {
                                    let document = Html::parse_document(&set_page);
                                    let product_selector =
                                        &Selector::parse(".products.row .product").unwrap();
                                    let products = document.select(product_selector);

                                    for product in products {
                                        match self
                                            .get_card_from_html(product, set_names_clone.clone())
                                        {
                                            Ok(card) => cards.push(card),
                                            Err(e) => warn!("Error parsing card: {}", e),
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("Error reading response text for {}: {}", link, e)
                                }
                            },
                            Err(e) => error!("Error fetching page {}: {}", link, e),
                        }
                    }
                    cards
                }
            })
            .buffered(40)
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .flatten()
            .collect();

        let mut grouped_cards = HashMap::new();
        for card in &cards {
            grouped_cards
                .entry(card.name.clone())
                .or_insert_with(Vec::new)
                .push(card.clone())
        }
        Ok(grouped_cards)
    }
}

#[cfg(test)]
mod tests {
    use crate::{cards::vendorcard::VendorCard, test::alphaspel::alphaspel_page_set_endings};

    use super::*;
    use tokio;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    //     #[tokio::test]
    // async fn test_get_all_card_pages_live() {
    //     init();

    //     let url = "https://alphaspel.se";

    //     let scraper = AlphaspelScraper::new(&url);
    //     let result = scraper.get_all_card_pages().await.unwrap();
    //     let (pages, set_names): (Vec<_>, Vec<_>) = result.into_iter().unzip();
    //     info!("Found {} alphaspel set pages", pages.len());
    //     // assert_eq!(result, alphaspel_page_set_endings())
    // }

    #[tokio::test]
    async fn test_get_all_card_pages() {
        init();

        let html_content = include_str!("test/alphaspel_starter_page.html",);

        let mut server = std::thread::spawn(|| mockito::Server::new())
            .join()
            .unwrap();
        let url = server.url();
        // Create a mock
        let mock = server
            .mock("GET", "/1978-mtg-loskort/")
            .with_status(200)
            .with_header("content-type", "text/plain")
            .with_body(html_content)
            .create();

        let scraper = AlphaspelScraper::new(&url);
        let result = scraper.get_all_card_pages().await.unwrap();
        mock.assert();
        assert_eq!(result, alphaspel_page_set_endings())
    }

    #[test]
    fn test_card_parser() {
        init();

        let card_html = include_str!("test/alphaspel_cards_page.html");

        let document = Html::parse_document(&card_html);
        let selector = Selector::parse(".products.row div.product").unwrap();
        let products = document.select(&selector);

        let alpha_scraper = AlphaspelScraper::new("fake");

        let mut cards: Vec<VendorCard> = Vec::new();
        for product in products {
            match alpha_scraper.get_card_from_html(
                product,
                ["Bloomburrow".to_string(), "10th Edition".to_string()].to_vec(),
            ) {
                Ok(card) => cards.push(card),
                Err(e) => {
                    error!("Error parsing card: {}", e);
                }
            }
        }
        let first_card = VendorCard {
            name: CardName::new("Loxodon Mystic".to_owned()).unwrap(),
            vendor: Vendor::Alphaspel,
            foil:false,
            image_url: "fake/media/products/thumbs/e29e66d6-1d3b-4824-9ae4-a8607aec5648.250x250_q50_fill.png".to_owned(),
            extended_art: false,
            prerelease: false,
            showcase: false,
            set: SetName::new("10th Edition".to_owned()).unwrap(),
            price: Price::new(5.0, Currency::SEK),
            trade_in_price: 0,
            current_stock: 9,
            max_stock: 3,
            collector_number: None,
        };

        assert_eq!(cards.len(), 51);
        assert_eq!(cards[0], first_card);
        assert_eq!(cards[49].extended_art, true);
        assert_eq!(cards.last().unwrap().prerelease, true);
        assert_eq!(cards.last().unwrap().foil, true);
        assert_eq!(
            cards.last().unwrap().name.almost_raw,
            "Whiskervale Forerunner"
        );
    }

    #[tokio::test]
    async fn test_scraper_cards() {
        init();

        //Given all calls mocked
        let mut server = std::thread::spawn(|| mockito::Server::new())
            .join()
            .unwrap();
        let url = server.url();

        let starterpage_short = include_str!("test/alphaspel_starter_page_short.html",);

        // Create a mock
        let mock = server
            .mock("GET", "/1978-mtg-loskort/")
            .with_status(200)
            .with_header("content-type", "text/plain")
            .with_body(starterpage_short)
            .create();

        let card_page = include_str!("test/alphaspel_cards_page_no_extra_pages.html",);
        //Call to see amount of pages
        let mock2 = server
            .mock(
                "GET",
                "/2441-10th-edition/?order_by=stock_a&ordering=desc&page=1",
            )
            .with_status(200)
            .with_header("content-type", "text/plain")
            .with_body(card_page)
            .create();

        //Call to get cards
        let mock3 = server
            .mock(
                "GET",
                "/2441-10th-edition/?order_by=stock_a&ordering=desc&page=1",
            )
            .with_status(200)
            .with_header("content-type", "text/plain")
            .with_body(card_page)
            .create();

        //When scrape_cards is called
        let scraper = AlphaspelScraper::new(&url);

        //Then we should have a Hashmap of 54 cards
        let result = scraper.scrape_cards().await.unwrap();
        mock.assert();
        mock2.assert();
        mock3.assert();
        assert_eq!(result.len(), 51);
    }
}
