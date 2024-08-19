use regex::Regex;
use reqwest::blocking;
use scraper::{Html, Selector};
use std::error::Error;

const UNWANTED_PATTERNS: [&str; 4] = [
    r"(?i)\(skadad\)",
    r"(?i)\( Skadad \)",
    r"(?i)\(Spelad\)",
    r"(?i)\[Token\]",
];

const FOIL_PATTERNS: [&str; 3] = [
    r"(?i)\(Foil\)",
    r"(?i)\(Etched Foil\)",
    r"(?i)\(Foil Etched\)",
];

#[derive(Debug, PartialEq)]
pub struct Card {
    name: String,
    foil: bool,
    image_url: String,
    extended_art: bool,
    prerelease: bool,
    showcase: bool,
    set: String,
    price: i32,
    trade_in_price: i32,
    current_stock: i8,
    max_stock: i8,
}

fn create_regex_patterns(patterns: &[&str]) -> Result<Vec<Regex>, Box<dyn Error>> {
    patterns
        .iter()
        .map(|&p| Regex::new(p).map_err(|e| Box::new(e) as Box<dyn Error>)) // Convert regex::Error to Box<dyn Error>
        .collect()
}
fn parse_price(price_str: Option<&str>) -> i32 {
    price_str
        .and_then(|s| s.replace(" kr", "").trim().parse::<i32>().ok())
        .unwrap_or(0)
}

fn get_price(tr_elements: scraper::ElementRef) -> Result<i32, Box<dyn Error>> {
    let price_if_item_is_in_store = tr_elements
        .select(&Selector::parse("td.align-right span.format-bold")?)
        .next()
        .map(|element| parse_price(Some(element.text().collect::<String>().as_str())));
    let price_if_item_is_not_in_store = tr_elements
        .select(&Selector::parse("td.align-right span.format-subtle")?)
        .next()
        .map(|element| parse_price(Some(element.text().collect::<String>().as_str())));
    let price = price_if_item_is_in_store
        .or(price_if_item_is_not_in_store)
        .unwrap_or(0);
    Ok(price)
}

pub fn fetch_and_parse(url: &str) -> Result<Vec<Card>, Box<dyn Error>> {
    let response = blocking::get(url)?;
    let html_content = response.text()?;
    let parse_document = Html::parse_document(&html_content);
    let table_selector = Selector::parse("tr[id*='product-row-']")?;
    let selected_elements = parse_document.select(&table_selector);

    let unwanted_patterns = create_regex_patterns(&UNWANTED_PATTERNS)?;
    let foil_patterns = create_regex_patterns(&FOIL_PATTERNS)?;

    let mut results = Vec::new();

    for tr_elements in selected_elements {
        let name = tr_elements
            .select(&Selector::parse("a.fancybox")?)
            .next()
            .map(|el| {
                el.text()
                    .collect::<String>()
                    .replace(
                        "\n                                                                ",
                        " ",
                    )
                    .trim()
                    .to_string()
            })
            .unwrap_or_default();

        if unwanted_patterns
            .iter()
            .any(|pattern| pattern.is_match(&name))
        {
            continue; // Skip unwanted cards
        }

        let foil = foil_patterns.iter().any(|pattern| pattern.is_match(&name));
        let prerelease = Regex::new(r"(?i)\(Prerelease\)")?.is_match(&name);
        let showcase = Regex::new(r"(?i)\(Showcase\)")?.is_match(&name);
        let extended_art = Regex::new(r"(?i)\(Extended Art\)")?.is_match(&name);

        let mut set = tr_elements
            .select(&Selector::parse("img[title]")?)
            .next()
            .and_then(|attributes| attributes.value().attr("title").map(String::from))
            .unwrap_or("UNKNOWN".to_string());

        let other_set = tr_elements
            .select(&Selector::parse("td.align-right a")?)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or("UNKNOWN".to_string());
        // .and_then(|attributes| attributes.value().attr("title").map(String::from))
        // .unwrap_or("UNKNOWN".to_string());

        if set == "UNKNOWN" {
            set = other_set;
        }

        let price = get_price(tr_elements)?;

        let trade_in_price = tr_elements
            .select(&Selector::parse("td.align-right")?)
            .nth(2)
            .and_then(|element| {
                Some(parse_price(Some(
                    element.text().collect::<String>().as_str(),
                )))
            });

        let stock = tr_elements
            .select(&Selector::parse("td.align-right")?)
            .nth(3)
            .and_then(|element| {
                let stock_str = element.text().collect::<String>();
                let stock_numbers: Vec<i8> = stock_str
                    .replace("/", "")
                    .replace("st", "")
                    .trim()
                    .split_whitespace()
                    .filter_map(|s| s.parse::<i8>().ok())
                    .collect();
                Some(stock_numbers)
            })
            .unwrap_or_else(|| vec![0, 0]); // Default to a vector of two zeros if parsing fails

        let card = Card {
            name,
            foil,
            image_url: "https://astraeus.dragonslair.se/images/4026/product".to_string(),
            extended_art,
            prerelease,
            showcase,
            set,
            price,
            trade_in_price: trade_in_price.unwrap_or(0),
            current_stock: stock.first().unwrap_or(&0).to_owned(),
            max_stock: stock.last().unwrap_or(&0).to_owned(),
        };

        results.push(card);
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    // use mockito::{mock, Matcher};

    #[test]
    fn test_fetch_and_parse() {
        let html_content =
            fs::read_to_string("/workspaces/mtg-prz-rust/product_search_page.html").unwrap();

        let mut server = mockito::Server::new();
        let url = server.url();
        // Create a mock
        let mock = server
            .mock("GET", "/product/card-singles/magic?name=reaper+king")
            .with_status(200)
            .with_header("content-type", "text/plain")
            .with_header("x-api-key", "1234")
            .with_body(html_content.clone())
            .create();

        // let url = &mockito::server_url();
        let result = fetch_and_parse(&format!(
            "{}/product/card-singles/magic?name=reaper+king",
            url
        ))
        .unwrap();

        mock.assert();
        assert_eq!(result.len(), 8);

        let regular_card = Card {
            name: "Reaper King".to_string(),
            foil: false,
            image_url: "https://astraeus.dragonslair.se/images/4026/product".to_string(),
            extended_art: false,
            prerelease: false,
            showcase: false,
            set: "Shadowmoor".to_string(),
            price: 100,
            trade_in_price: 50,
            current_stock: 1,
            max_stock: 2,
        };
        assert_eq!(result.first().unwrap(), &regular_card);
        assert_eq!(result.get(1).unwrap().foil, true);
        assert_eq!(result.get(2).unwrap().foil, true);
        assert_eq!(result.get(3).unwrap().foil, true);
        assert_eq!(result.get(4).unwrap().prerelease, true);
        assert_eq!(result.get(5).unwrap().showcase, true);
        assert_eq!(result.get(6).unwrap().extended_art, true);
        assert_eq!(
            result.get(7).unwrap().set,
            "Mystery Booster Retail Edition Foils"
        );
    }
}
