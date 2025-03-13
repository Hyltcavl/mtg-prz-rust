use crate::cards::scryfallcard::Prices;
use crate::cards::setname::SetName;
use crate::date_time_as_string;
use crate::utilities::string_manipulators::clean_string;
use crate::utilities::file_management::save_to_file;
use chrono::{format, Local};
use log::{self, info};
use reqwest;
use serde_json::Value;
use std::collections::HashMap;
use std::vec;
use std::{fs, path::Path};

use crate::cards::cardname::CardName;
use crate::cards::scryfallcard::ScryfallCard;
use crate::utilities::file_management::{download_and_save_file, load_from_json_file};

const PRICES_DIR: &str = "scryfall_prices_raw";
const FILE_PREFIX: &str = "scryfall_download";

pub struct ScryfallScraper {
    client: reqwest::Client,
    base_url: String,
    directory_path: String,
}

impl ScryfallScraper {
    pub fn new(base_url: &str, client: reqwest::Client, directory_path: Option<String>) -> Self {
        ScryfallScraper {
            client,
            base_url: base_url.to_string(),
            directory_path: directory_path
                .unwrap_or_else(|| "/workspaces/mtg-prz-rust/scryfall/".to_string()),
        }
    }

    fn setup_http_headers() -> reqwest::header::HeaderMap {
        let mut header_map = reqwest::header::HeaderMap::new();
        header_map.insert(reqwest::header::ACCEPT, "*/*".parse().unwrap());
        header_map.insert(
            reqwest::header::USER_AGENT,
            "application/json".parse().unwrap(),
        );
        header_map
    }

    fn get_prices_directory(&self) -> std::path::PathBuf {
        Path::new(&self.directory_path).join(PRICES_DIR)
    }

    fn create_file_path(&self, timestamp: &str) -> std::path::PathBuf {
        self.get_prices_directory()
            .join(format!("{}_{}.json", FILE_PREFIX, timestamp))
    }

    fn get_existing_scryfall_file(&self) -> Option<String> {
        let current_date = Local::now().format("%Y-%m-%d").to_string();
        let directory = self.get_prices_directory();

        fs::read_dir(directory).ok()?.find_map(|entry| {
            let entry = entry.ok()?;
            let file_name = entry.file_name().into_string().ok()?;
            if file_name.starts_with(&format!("{}_{}", FILE_PREFIX, current_date))
                && file_name.ends_with(".json")
            {
                Some(entry.path().to_str()?.to_string())
            } else {
                None
            }
        })
    }

    pub async fn get_large_scryfall_cards_file(
        &self,
    ) -> Result<String, Box<dyn std::error::Error>> {
        if let Some(existing_file) = self.get_existing_scryfall_file() {
            info!("Using existing Scryfall price file: {}", existing_file);
            return Ok(existing_file);
        }

        let url = format!("{}/bulk-data", self.base_url);
        let response = self
            .client
            .get(url)
            .headers(Self::setup_http_headers())
            .send()
            .await?
            .text()
            .await?;

        let json: Value = serde_json::from_str(&response)?;
        let download_uri = json["data"][2]["download_uri"]
            .as_str()
            .ok_or("Missing download_uri")?;

        let current_time = Local::now().format("%Y-%m-%d_%H:%M:%S").to_string();
        let path = self.create_file_path(&current_time);

        fs::create_dir_all(self.get_prices_directory())?;
        download_and_save_file(download_uri, path.to_str().unwrap()).await?;

        info!("Saved raw scryfall price file to: {}", path.display());
        Ok(path.to_str().unwrap().to_string())
    }

    fn is_not_artseries(&self, obj: &Value) -> bool {
        obj["layout"] != "art_series"
    }

    fn is_not_basic_land(&self, obj: &Value) -> bool {
        !obj["type_line"]
            .as_str()
            .unwrap_or("Not what we are looking for")
            .starts_with("Basic Land")
    }

    fn is_not_token(&self, obj: &Value) -> bool {
        obj["layout"] != "token"
    }
    pub fn convert_to_domain_version(
        &self,
        path: &str,
    ) -> Result<HashMap<CardName, Vec<ScryfallCard>>, Box<dyn std::error::Error>> {
        let mut scryfall_card_list = Vec::new();

        let cards: serde_json::Value = load_from_json_file(&path)?;
        if let Value::Array(cards_array) = cards {
            for obj in cards_array {
                if self.is_not_token(&obj)
                    && self.is_not_basic_land(&obj)
                    && self.is_not_artseries(&obj)
                {
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

        save_to_file(&parsed_file_path, &grouped_cards)?;

        Ok(grouped_cards)
    }

    async fn get_scryfall_cards() -> HashMap<CardName, Vec<ScryfallCard>> {
        HashMap::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use env_logger;
    use mockito;
    use serde_json::json;
    use tempfile::tempdir;

    struct TestContext {
        server: mockito::ServerGuard,
        temp_dir: tempfile::TempDir,
        scraper: ScryfallScraper,
    }

    impl TestContext {
        fn new() -> Self {
            let _ = env_logger::builder().is_test(true).try_init();
            let server = std::thread::spawn(|| mockito::Server::new())
                .join()
                .unwrap();
            let temp_dir = tempdir().unwrap();

            let scraper = ScryfallScraper::new(
                &server.url(),
                reqwest::Client::new(),
                Some(temp_dir.path().to_str().unwrap().to_string()),
            );

            TestContext {
                server,
                temp_dir,
                scraper,
            }
        }

        fn setup_prices_directory(&self) -> std::path::PathBuf {
            let prices_dir = self.temp_dir.path().join(PRICES_DIR);
            fs::create_dir_all(&prices_dir).unwrap();
            prices_dir
        }

        fn create_mock_file(&self, content: &str) -> std::path::PathBuf {
            let prices_dir = self.setup_prices_directory();
            let current_date = Local::now().format("%Y-%m-%d").to_string();
            let file_name = format!("{}_{}.json", FILE_PREFIX, current_date);
            let file_path = prices_dir.join(file_name);
            fs::write(&file_path, content).unwrap();
            file_path
        }
    }

    #[tokio::test]
    async fn test_new_download_when_no_file_exists() {
        let mut ctx = TestContext::new();

        // Setup mocks
        let mock = ctx
            .server
            .mock("GET", "/bulk-data")
            .with_status(200)
            .with_body(
                json!({
                    "data": [{}, {}, {"download_uri": ctx.server.url() + "/all-cards"}]
                })
                .to_string(),
            )
            .create();

        let html_content = include_str!("test/scryfall_card_resp.json");
        let _cards_mock = ctx
            .server
            .mock("GET", "/all-cards")
            .with_status(200)
            .with_body(html_content)
            .create();

        // Execute and verify
        let result = ctx.scraper.get_large_scryfall_cards_file().await.unwrap();
        assert!(fs::metadata(&result).is_ok());
        assert!(!fs::read_to_string(&result).unwrap().is_empty());
        mock.assert();
    }

    #[tokio::test]
    async fn test_reuse_existing_file() {
        let mut ctx = TestContext::new();

        // Setup existing file
        let file_path = ctx.create_mock_file("test content");

        // Setup mock that should never be called
        let mock = ctx.server.mock("GET", "/bulk-data").expect(0).create();

        // Execute and verify
        let result = ctx.scraper.get_large_scryfall_cards_file().await.unwrap();
        assert_eq!(result, file_path.to_str().unwrap());
        mock.assert();
    }

    #[tokio::test]
    async fn test_should_create_domain_version_of_scryfall_cards_list() {
        let mut ctx = TestContext::new();

        // Setup existing file

        let file_path = ctx.create_mock_file(include_str!("test/scryfall_card_resp.json"));

        let parsed_cards = ctx
            .scraper
            .convert_to_domain_version(file_path.to_str().unwrap())
            .unwrap();

        // Execute and verify
        // assert_eq!(result, file_path.to_str().unwrap());

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
