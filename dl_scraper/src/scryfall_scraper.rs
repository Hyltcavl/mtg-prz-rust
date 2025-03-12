use chrono::{format, Local};
use log::{self, info};
use reqwest;
use serde_json::Value;
use std::collections::HashMap;
use std::vec;
use std::{fs, path::Path};

use crate::cards::cardname::CardName;
use crate::cards::scryfallcard::ScryfallCard;
use crate::utilities::file_management::download_and_save_file;

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

        let html_content =
            include_str!("/workspaces/mtg-prz-rust/dl_scraper/src/test/scryfall_card_resp.json");
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
}
