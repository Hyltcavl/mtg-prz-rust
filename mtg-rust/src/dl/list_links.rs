use regex::Regex;
use reqwest::blocking;
use scraper::{Html, Selector};
use std::{any::Any, error::Error, fmt::format};

pub fn request_page(cmc: u32, page_count: u32) -> Option<String> {
    let request_url = format_args!(
        "https://astraeus.dragonslair.se/product/magic/card-singles/store:kungsholmstorg/cmc-{}/{}",
        cmc, page_count,
    )
    .to_string();

    // i need a way to try and get the text, and if i fail, return none, otherways return some witht he String in it.
    let result = match blocking::get(request_url) {
        Ok(response) => response.text(),
        Err(e) => {
            println!("Failed to fetch URL: {} \n with error {}", request_url, e);
            return None;
        }
    };
    result
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    // use mockito::{mock, Matcher};

    #[test]
    fn test_fetch_and_parse() {
        request_page()
        // let html_content =
        //     fs::read_to_string("/workspaces/mtg-prz-rust/product_search_page.html").unwrap();

        // let mut server = mockito::Server::new();
        // let url = server.url();
        // // Create a mock
        // let mock = server
        //     .mock("GET", "/product/card-singles/magic?name=reaper+king")
        //     .with_status(200)
        //     .with_header("content-type", "text/plain")
        //     .with_header("x-api-key", "1234")
        //     .with_body(html_content.clone())
        //     .create();

        // // let url = &mockito::server_url();
        // let result = fetch_and_parse(&format!(
        //     "{}/product/card-singles/magic?name=reaper+king",
        //     url
        // ))
        // .unwrap();

        // mock.assert();
        // assert_eq!(result.len(), 8);
    }
}
