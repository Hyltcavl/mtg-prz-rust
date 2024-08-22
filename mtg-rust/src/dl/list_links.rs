use reqwest::blocking;
use scraper::{Html, Selector};

pub fn get_page_count(url: &str) -> Option<u32> {
    let response = blocking::get(url).ok()?;
    let html_content = response.text().ok()?;

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

pub fn request_page(cmc: u32, page_count: u32) -> Option<String> {
    let request_url = format_args!(
        "https://astraeus.dragonslair.se/product/magic/card-singles/store:kungsholmstorg/cmc-{}/{}",
        cmc, page_count,
    )
    .to_string();

    // i need a way to try and get the text, and if i fail, return none, otherways return some witht he String in it.
    let result = match blocking::get(&request_url) {
        Ok(response) => response.text(),
        Err(e) => {
            println!("Failed to fetch URL: {} \n with error: {}", &request_url, e);
            return None;
        }
    };

    match result {
        Ok(result) => Some(result),
        Err(e) => {
            println!("Failed get response text with error: {}", e);
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    // use mockito::{mock, Matcher};

    #[test]
    fn test_fetch_and_parse() {
        let res = request_page(0, 0);
        println!("{}", res.unwrap())
    }

    #[test]
    fn test_get_page_count() {
        let html_content =
            fs::read_to_string("/workspaces/mtg-prz-rust/get_pages_page.html").unwrap();
        let mut server = mockito::Server::new();
        let url = server.url();
        let mock = server
            .mock(
                "GET",
                "/product/magic/card-singles/store:kungsholmstorg/cmc-0/1",
            )
            .with_status(200)
            .with_header("content-type", "text/plain")
            .with_header("x-api-key", "1234")
            .with_body(html_content.clone())
            .create();
        let url = format!(
            "{}/product/magic/card-singles/store:kungsholmstorg/cmc-0/1",
            url
        );

        let res = get_page_count(&url);
        mock.assert();
        assert_eq!(res.unwrap(), 51);
    }

    #[test]
    fn test_get_page_count_no_pages() {
        let html_content =
            fs::read_to_string("/workspaces/mtg-prz-rust/get_pages_no_pages.html").unwrap();
        let mut server = mockito::Server::new();
        let url = server.url();
        let mock = server
            .mock(
                "GET",
                "/product/magic/card-singles/store:kungsholmstorg/cmc-15/1",
            )
            .with_status(200)
            .with_header("content-type", "text/plain")
            .with_header("x-api-key", "1234")
            .with_body(html_content.clone())
            .create();
        let url = format!(
            "{}/product/magic/card-singles/store:kungsholmstorg/cmc-15/1",
            url
        );

        let res = get_page_count(&url);
        mock.assert();
        assert_eq!(res.unwrap(), 1);
    }
}
