use std::error::Error;

use regex::Regex;
use reqwest::blocking;
use scraper::{Html, Selector};

#[derive(Debug, PartialEq)]
struct Card {
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

fn fetch_and_parse(url: &str) -> Result<Vec<Card>, Box<dyn Error>> {
    let response = blocking::get(url)?;
    let html_content = response.text()?;

    let parse_document = Html::parse_document(&html_content);
    let table_selector = Selector::parse("tr[id*='product-row-']").unwrap();
    let selected_elements = parse_document.select(&table_selector);

    let mut results = Vec::new();

    for tr_elements in selected_elements {
        let name = tr_elements
            .select(&Selector::parse("a.fancybox")?)
            .next()
            .unwrap()
            .text()
            .map(|txt| {
                txt.trim().replace(
                    "\n                                                                ",
                    " ",
                )
            })
            .collect::<Vec<_>>()
            .join(",")
            .to_string();
        // println!("{}", &name);

        let patterns = [
            Regex::new(r"(?i)\(skadad\)")?,
            Regex::new(r"(?i)\( Skadad \)")?,
            Regex::new(r"(?i)\(Spelad\)")?,
            Regex::new(r"(?i)\[Token\]")?,
        ];
        let mut not_wanted = false;
        patterns.iter().for_each(|pattern| {
            if pattern.is_match(&name) {
                not_wanted = true
            }
        });

        if not_wanted {
            continue;
        }

        let foil = [
            Regex::new(r"(?i)\(Foil\)")?,
            Regex::new(r"(?i)\(Etched Foil\)")?,
            Regex::new(r"(?i)\(Foil Etched\)")?,
        ]
        .into_iter()
        .find(|filter| filter.is_match(&name))
        .is_some();

        let prerelease = Regex::new(r"(?i)\(Prerelease\)")?.is_match(&name);

        let showcase = Regex::new(r"(?i)\(Showcase\)")?.is_match(&name);

        let extended_art = Regex::new(r"(?i)\(Extended Art\)")?.is_match(&name);

        let set = tr_elements
            .select(&Selector::parse("img[title]")?)
            .next()
            .and_then(|attributes| attributes.value().attr("title").map(String::from));

        let price_if_item_is_in_store = tr_elements
            .select(&Selector::parse("td.align-right span.format-bold")?)
            .next()
            .and_then(|element| {
                Some(
                    element
                        .text()
                        .collect::<String>()
                        .replace(" kr", "")
                        .trim()
                        .to_owned(),
                )
            });

        let price_if_item_is_not_in_store = tr_elements
            .select(&Selector::parse("td.align-right span.format-subtle")?)
            .next()
            .and_then(|element| {
                Some(
                    element
                        .text()
                        .collect::<String>()
                        .replace(" kr", "")
                        .trim()
                        .to_owned(),
                )
            });

        let mut price = 0;
        if price_if_item_is_in_store.is_some() {
            price = price_if_item_is_in_store.unwrap().parse().unwrap_or(0);
        } else if price_if_item_is_not_in_store.is_some() {
            let p = price_if_item_is_not_in_store.unwrap();
            price = p.parse().unwrap_or(0);
        }

        let price_if_item_is_tradable = tr_elements
            .select(&Selector::parse("td.align-right")?)
            .nth(2)
            .and_then(|element| {
                Some(
                    element
                        .text()
                        .collect::<String>()
                        .replace(" kr", "")
                        .trim()
                        .to_owned(),
                )
            });

        let trade_in_price = price_if_item_is_tradable
            .as_ref()
            .map(|price_str| {
                price_str.parse::<i32>().unwrap_or_else(|_| {
                    println!(
                        "Unable to convert tradable price to integer, string was: {}",
                        price_str
                    );
                    0
                })
            })
            .unwrap_or(0);

        let stock = tr_elements
            .select(&Selector::parse("td.align-right span.stock")?)
            .next()
            .and_then(|element| Some(element.text().collect::<String>().trim().to_owned()))
            .as_ref()
            .map(|stock_str| {
                stock_str.parse::<i8>().unwrap_or_else(|_| {
                    println!(
                        "Unable to convert stock to integer, string was: {}",
                        stock_str
                    );
                    0
                })
            })
            .unwrap_or(0);

        let max_stock = tr_elements
            .select(&Selector::parse("td.align-right").unwrap())
            .nth(3)
            .and_then(|element| {
                Some(
                    element
                        .text()
                        .collect::<String>()
                        .replace("/", "")
                        .replace("st", "")
                        .trim()
                        .to_owned()
                        .split_whitespace()
                        .map(|s| s.to_string())
                        .collect::<Vec<String>>(),
                )
            })
            .map(|stock_strings| {
                stock_strings
                    .into_iter()
                    .map(|stock_str| {
                        stock_str.parse::<i8>().unwrap_or_else(|_| {
                            println!(
                                "Unable to convert max_stock to integer, string was: {}",
                                stock_str
                            );
                            0
                        })
                    })
                    .collect::<Vec<i8>>()
            })
            .unwrap_or_else(|| vec![0, 0]); // Default to a vector of two zeros if parsing fails

        //     println!("{:?}", max_stock);
        // }

        // println!("{}", price_if_item_is_tradable.unwrap());
        // let price: i32 = price.replace(" kr", "").trim().parse().unwrap();

        // let s = tr_elements
        //     .select(&Selector::parse("td.align-right")?)
        //     .next()
        //     .unwrap()
        //     .text()
        //     .collect::<Vec<_>>()
        //     .join(",")
        //     .to_string();

        // println!("{}", s);

        let card = Card {
            name: name,
            foil: foil,
            image_url: "https://astraeus.dragonslair.se/images/4026/product".to_string(),
            extended_art: extended_art,
            prerelease: prerelease,
            showcase: showcase,
            set: set.unwrap_or("UNKNOWN".to_string()),
            price: price,
            trade_in_price: trade_in_price,
            current_stock: max_stock.first().unwrap().to_owned(),
            max_stock: max_stock.last().unwrap().to_owned(),
        };

        results.push(card);
    }

    Ok(results)
}

fn main() {
    let url = "https://astraeus.dragonslair.se/product/card-singles/magic?name=reaper+king";
    match fetch_and_parse(url) {
        Ok(results) => {
            for result in results {
                println!("{:#?}", result);
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
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
        assert_eq!(result.len(), 4);

        let first_card = Card {
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
        assert_eq!(result.first(), Some(&first_card))
    }
}
