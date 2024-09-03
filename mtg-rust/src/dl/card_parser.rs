use regex::Regex;
use reqwest;
use scraper::{Html, Selector};
use std::error::Error;
use tokio::time::Instant;

use crate::cards::card::{CardName, SetName, Vendor, VendorCard};

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

pub async fn fetch_and_parse(url: &str) -> Result<Vec<VendorCard>, Box<dyn Error>> {
    let start = Instant::now();
    let response = reqwest::get(url).await?;
    log::debug!("fetching {} took {:?} sec", url, start.elapsed().as_secs());
    let html_content = response.text().await?;
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
            .unwrap_or(
                // if it doesn't have a name, with a link to image such as: https://astraeus.dragonslair.se/product/card-singles/magic/store:kungsholmstorg/sort:recent/magic-warhammer-40-000
                tr_elements
                    .select(&Selector::parse("td.wrap")?)
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
                    .unwrap(),
            );

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

        // TODO: ignore cards that are only basic, so plain, mountain, forest etc
        let card_name = match CardName::new(name) {
            Ok(card_name) => card_name,
            Err(e) => {
                log::error!("Error parsing card name: {}", e);
                continue;
            }
        };

        let image_url = tr_elements
            .select(&Selector::parse("a.fancybox")?)
            .next()
            .and_then(|el| el.value().attr("href"))
            .map(|href| format!("https://astraeus.dragonslair.se{}", href))
            .unwrap_or(
                "https://upload.wikimedia.org/wikipedia/en/a/aa/Magic_the_gathering-card_back.jpg"
                    .to_string(),
            );

        let mut set = tr_elements
            .select(&Selector::parse("img[title]")?)
            .next()
            .and_then(|attributes| attributes.value().attr("title").map(String::from))
            .unwrap_or("UNKNOWN".to_string())
            .replace(
                "\n                                                                ",
                " ",
            )
            .to_owned();

        let other_set = tr_elements
            .select(&Selector::parse("td.align-right a")?)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or("UNKNOWN".to_string())
            .replace(
                "\n                                                                ",
                " ",
            )
            .to_owned();

        if set == "UNKNOWN" {
            set = other_set;
        }

        let set_name = match SetName::new(set) {
            Ok(set_name) => set_name,
            Err(e) => {
                log::error!("Error parsing set name: {}", e);
                continue;
            }
        };

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

        let card = VendorCard {
            vendor: Vendor::Dragonslair,
            name: card_name,
            foil,
            image_url: image_url,
            extended_art,
            prerelease,
            showcase,
            set: set_name,
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

    use crate::test::helpers::reaper_king_vendor_card;

    use super::*;
    use tokio;
    #[tokio::test]
    async fn test_fetch_and_parse() {
        let html_content =
            fs::read_to_string("/workspaces/mtg-prz-rust/product_search_page.html").unwrap();

        let mut server = std::thread::spawn(|| mockito::Server::new())
            .join()
            .unwrap();
        let url = server.url();
        // Create a mock
        let mock = server
            .mock("GET", "/product/card-singles/magic?name=reaper+king")
            .with_status(200)
            .with_header("content-type", "text/plain")
            .with_header("x-api-key", "1234")
            .with_body(html_content.clone())
            .create();

        let result = fetch_and_parse(&format!(
            "{}/product/card-singles/magic?name=reaper+king",
            url
        ))
        .await
        .unwrap();

        mock.assert();
        assert_eq!(result.len(), 9);

        let reaper_king_vendor_card = reaper_king_vendor_card();

        assert_eq!(result.first().unwrap(), &reaper_king_vendor_card);
        assert_eq!(result.get(1).unwrap().foil, true);
        assert_eq!(result.get(2).unwrap().foil, true);
        assert_eq!(result.get(3).unwrap().foil, true);
        assert_eq!(result.get(4).unwrap().prerelease, true);
        assert_eq!(result.get(5).unwrap().showcase, true);
        assert_eq!(result.get(6).unwrap().extended_art, true);
        assert_eq!(
            result.get(7).unwrap().set.cleaned,
            "mystery booster retail edition foils"
        );
        assert_eq!(
            result.get(8).unwrap().image_url,
            "https://upload.wikimedia.org/wikipedia/en/a/aa/Magic_the_gathering-card_back.jpg"
        );
    }
}
