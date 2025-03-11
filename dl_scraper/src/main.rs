use std::{error::Error, time::Instant};
use reqwest::Client;


struct DragonsLairScraper {
    url: String,
    client: Client,
    cmcs_available: Vec<u8>,
}

impl DragonsLairScraper {
    fn new(url: &str, cmcs_available: Option<Vec<u8>>) -> Self {
        DragonsLairScraper {
            url: url.to_string(),
            client: Client::new(),
            cmcs_available: cmcs_available
                .unwrap_or_else(|| vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 15, 16]),
        }
    }

    async fn get_card_urls(&self) -> Result<Vec<String>, Box<dyn Error>> {
        let start_time = Instant::now();
        let semaphore = Arc::new(Semaphore::new(10));
    
        let page_fetches_asyncs = cmcs_available.into_iter().map(|cmc| {
            log::info!("Fetching DL cards with cmc: {}", cmc);
    
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
                log::debug!("cmc: {}, and page_count is {:?}", cmc, page_count);
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
        log::debug!("{:?}", results);
        return results.into_iter().flatten().collect();
    }

    pub async fn get_available_cards(&self) -> Result<Vec<VendorCard>, Box<dyn Error>> {
        
        self.get_card_urls().await

        let mut all_cards = Vec::new();

        for cmc in self.cmcs_available.iter() {
            let request_url = format!(
                "{}/product/magic/card-singles/store:kungsholmstorg/cmc-{}/{}",
                self.url, cmc, 0
            );
            let page_count = self.get_page_count(&request_url).await?;

            for page in 1..=page_count {
                let page_url = format!(
                    "{}/product/magic/card-singles/store:kungsholmstorg/cmc-{}/{}",
                    self.url, cmc, page
                );
                let res = self.client.get(&page_url).send().await?.text().await?;
                let document = Html::parse_document(&res);
                let selector = Selector::parse(".magic-card-class").unwrap(); // Adjust the selector as needed

                let cards: Vec<VendorCard> = document
                    .select(&selector)
                    .map(|element| {
                        // Use the parsing logic from card_parser.rs
                        let name = element.text().collect::<String>().trim().to_string();
                        // ...existing code for parsing other fields...
                        VendorCard {
                            // ...initialize VendorCard fields...
                        }
                    })
                    .collect();

                all_cards.extend(cards);
            }
        }

        Ok(all_cards)
    }

    fn fetch_magic_cards(&self) -> Result<Vec<VendorCard>, Box<dyn Error>> {
        let mut all_cards = Vec::new();

        for cmc in self.cmcs_available.iter() {
            let request_url = format!(
                "{}/product/magic/card-singles/store:kungsholmstorg/cmc-{}/{}",
                self.url, cmc, 0
            );
            let page_count = self.get_page_count(&request_url)?;

            for page in 1..=page_count {
                let page_url = format!(
                    "{}/product/magic/card-singles/store:kungsholmstorg/cmc-{}/{}",
                    self.url, cmc, page
                );
                let res = self.client.get(&page_url).send()?.text()?;
                let document = Html::parse_document(&res);
                let selector = Selector::parse(".magic-card-class").unwrap(); // Adjust the selector as needed

                let cards: Vec<VendorCard> = document
                    .select(&selector)
                    .map(|element| {
                        // Use the parsing logic from card_parser.rs
                        let name = element.text().collect::<String>().trim().to_string();
                        // ...existing code for parsing other fields...
                        VendorCard {
                            // ...initialize VendorCard fields...
                        }
                    })
                    .collect();

                all_cards.extend(cards);
            }
        }

        Ok(all_cards)
    }

    fn get_page_count(&self, url: &str) -> Result<usize, Box<dyn Error>> {
        let res = self.client.get(url).send()?.text()?;
        let document = Html::parse_document(&res);
        let selector = Selector::parse(".page-count-class").unwrap(); // Adjust the selector as needed

        let page_count = document
            .select(&selector)
            .next()
            .and_then(|element| element.text().collect::<Vec<_>>().join("").parse().ok())
            .unwrap_or(1);

        Ok(page_count)
    }
}



// fn main() {
//     let scraper = DragonLairScraper::new("https://astraeus.dragonslair.se", None);
//     match scraper.fetch_magic_cards() {
//         Ok(cards) => {
//             for card in cards {
//                 println!("{:?}", card);
//             }
//         }
//         Err(e) => eprintln!("Error fetching magic cards: {}", e),
//     }
// }


