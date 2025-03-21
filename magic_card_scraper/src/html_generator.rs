use std::collections::HashMap;
use std::error::Error;
use std::fs;

use crate::cards::cardname::CardName;
use crate::cards::compared_card::ComparedCard;
use crate::cards::currency::Currency;
use crate::cards::price::Price;
use crate::utilities::string_manipulators::date_time_as_string;

// Filter cards based on nice_price_diff
// A positive diff means that the card is atleast that much cheaper on MCM trend than the cheapest vendor price
// A negative diff means that the card is at most that much more expensive on MCM trend than the cheapest vendor price
// This diff is only applied on cards that have a MCM trend price of atleast 15 SEK, anything below must have a positive diff
pub fn filter_nice_price_cards(
    cards: &HashMap<CardName, Vec<ComparedCard>>,
    nice_price_limit: i32,
) -> Vec<&ComparedCard> {
    cards
        .values()
        .flatten()
        .filter(|card| {
            let price_sek = card.vendor_card.price.convert_to(Currency::SEK); // Compare price in SEK
            let price_diff = card.price_difference_to_cheapest_vendor_card;
            if price_sek <= 10.0 {
                price_diff <= 0
            } else if price_sek <= 30.0 {
                price_diff <= 5
            } else {
                price_sek > 30.0 && price_diff <= nice_price_limit
            }
        })
        .collect()
}
// const CARDS_PER_PAGE: usize = 50;
pub fn generate_nice_price_page(
    compared_cards: HashMap<CardName, Vec<ComparedCard>>,
    output_dir: &str,
    html_page_name: &str,
    nice_price_limit: i32,
) -> Result<(), Box<dyn Error>> {
    // Filter cards with positive price difference
    let positive_diff_cards: Vec<&ComparedCard> =
        filter_nice_price_cards(&compared_cards, nice_price_limit);

    let generate_page_content =
        generate_page_content(positive_diff_cards, &date_time_as_string(None, None));

    fs::write(
        format!("{}/{}", output_dir, html_page_name),
        generate_page_content,
    )?;

    Ok(())
}
fn generate_page_content(cards: Vec<&ComparedCard>, current_date: &str) -> String {
    // let mut sorted_cards = cards.to_vec();
    // sorted_cards.sort_by(|a, b| {
    //     a.price_difference_to_cheapest_vendor_card
    //         .cmp(&b.price_difference_to_cheapest_vendor_card)
    // });

    let mut content = format!(
        r#"
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>MTG Card Price Comparison, {} </title>
        <script src="https://cdnjs.cloudflare.com/ajax/libs/tablesort/5.2.1/tablesort.min.js"></script>
        <script src="https://cdnjs.cloudflare.com/ajax/libs/tablesort/5.2.1/sorts/tablesort.number.min.js"></script>
        <style> 
            {}
        </style>
    </head>
    <body>
        <h1>MTG-prizes {}, Total cards: {}</h1>
         <div class="filters">
        <div class="filter-group">
            <label for="vendorFilter">Vendor:</label>
            <select id="vendorFilter">
                <option value="all">All</option>
            </select>
            
            
            <button onclick="resetFilters()">Reset Filters</button>
        </div>
    </div>
        <table id="card-table">
            <thead>
                <tr>
                    <th>Image</th>
                    <th>Name</th>
                    <th data-sort-method="number">Vendor price (SEK)</th>
                    <th data-sort-method="number">MCM price (SEK)</th>
                    <th data-sort-method="number">Price Difference</th>
                    <th>Vendor</th>
                </tr>
            </thead>
            <tbody>
    "#,
        current_date,
        include_str!("../../magic_card_scraper/static/nice_price_cards_page/style.css"),
        current_date,
        cards.len(),
    );

    for card in cards {
        let vendor = &card.vendor_card.vendor;
        let name = &card.vendor_card.name.raw;
        let foil = card.vendor_card.foil;
        let cheapest_mcm_price = if foil {
            card.scryfall_card
                .prices
                .eur_foil
                .unwrap_or(Price::new(0.0, Currency::EUR))
        } else {
            card.scryfall_card
                .prices
                .eur
                .unwrap_or(Price::new(0.0, Currency::EUR))
        };
        let cheapest_vendor_price = &card.vendor_card.price;
        let price_diff = card.price_difference_to_cheapest_vendor_card;
        let image_url = &card.vendor_card.image_url;

        content.push_str(&format!(
            r#"
                <tr>
                    <td>
                        <div class="card-image-container">
                            <img class="card-image" src="{image_url}" alt="{name}">
                            <img class="enlarged-image" src="{image_url}" alt="{name}">
                        </div>
                    </td>
                    <td>{name}</td>
                    <td data-sort={cheapest_vendor_price}>{cheapest_vendor_price} SEK</td>
                    <td data-sort={cheapest_mcm_price:.2}>{cheapest_mcm_price:.2} </td>
                    <td data-sort={price_diff:.2}>{price_diff:.2} SEK</td>
                    <td>{vendor}</td>
                </tr>
            "#
        ));
    }

    content.push_str(generate_html_footer().as_str());

    content
}

pub fn generate_html_footer() -> String {
    format!(
        r#"
        </tbody></table>
        <div class='pagination'></div>
        <script>
        {}
        </script>
    </body>
    </html>
    "#,
        include_str!("../../magic_card_scraper/static/nice_price_cards_page/filter.js")
    )
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use crate::utilities::file_management::load_from_json_file;

    use super::*;
    use std::path::Path;

    struct TestContext {
        temp_dir: tempfile::TempDir,
    }

    impl TestContext {
        fn new() -> Self {
            let _ = env_logger::builder().is_test(true).try_init();

            let temp_dir = tempdir().unwrap();

            TestContext { temp_dir }
        }

        // fn setup_prices_directory(&self) -> std::path::PathBuf {
        //     let prices_dir = self.temp_dir.path().join(RAW_CARDS_DIR);
        //     fs::create_dir_all(&prices_dir).unwrap();
        //     prices_dir
        // }

        // fn create_mock_file(&self, content: &str) -> std::path::PathBuf {
        //     let prices_dir = self.setup_prices_directory();
        //     let current_date = Local::now().format("%Y-%m-%d").to_string();
        //     let file_name = format!("{}_{}.json", RAW_FILE_PREFIX, current_date);
        //     let file_path = prices_dir.join(file_name);
        //     fs::write(&file_path, content).unwrap();
        //     file_path
        // }
    }

    // #[test]
    // fn temp() {
    //     // let ctx = TestContext::new();
    //     // // let json_file_path = "/workspaces/mtg-prz-rust/mtg-rust/src/test/test_compared_cards.json";
    //     // // let output_dir = "/workspaces/mtg-prz-rust";
    //     let json_file_path =
    //         "/workspaces/mtg-prz-rust/dl_scraper/src/test/test_compared_cards.json";
    //     let cards = load_from_json_file::<Vec<ComparedCard>>(json_file_path).unwrap();

    //     let mut grouped_cards: HashMap<CardName, Vec<ComparedCard>> = HashMap::new();
    //     for card in cards {
    //         grouped_cards
    //             .entry(card.vendor_card.name.clone())
    //             .or_insert_with(Vec::new)
    //             .push(card.clone());
    //     }

    //     save_to_file(
    //         "/workspaces/mtg-prz-rust/dl_scraper/src/test/grouped_compared_cards.json",
    //         &grouped_cards,
    //     )
    //     .unwrap();
    // }

    #[test]
    fn test_generate_html_from_json_creates_file() {
        let ctx = TestContext::new();
        // let json_file_path = "/workspaces/mtg-prz-rust/mtg-rust/src/test/test_compared_cards.json";
        // let output_dir = "/workspaces/mtg-prz-rust";
        let json_file_path =
            "/workspaces/mtg-prz-rust/dl_scraper/src/test/test_grouped_compared_cards.json";
        let cards =
            load_from_json_file::<HashMap<CardName, Vec<ComparedCard>>>(json_file_path).unwrap();
        let html_page_name = "index.html";

        //Temporary dir
        let temp_dir = ctx.temp_dir.path().to_str().unwrap();

        // Call the function
        generate_nice_price_page(cards, temp_dir, html_page_name, 0).unwrap();

        // Check that the output directory and index file were created
        // assert!(Path::new(output_dir).exists());
        let index_file_path = Path::new(temp_dir).join(html_page_name);
        assert!(index_file_path.exists());
        assert!(!fs::read_to_string(&index_file_path).unwrap().is_empty());
        // assert!(fs::metadata(&result).is_ok());
        // assert!(!fs::read_to_string(&result).unwrap().is_empty());
    }

    #[test]
    fn test_filter_nice_price_cards_default_config() {
        let json_file_path =
            "/workspaces/mtg-prz-rust/dl_scraper/src/test/test_grouped_compared_cards.json";
        // let cards = load_from_json_file::<Vec<ComparedCard>>(json_file_path).unwrap();
        let cards =
            load_from_json_file::<HashMap<CardName, Vec<ComparedCard>>>(json_file_path).unwrap();
        let nice_price_cards = filter_nice_price_cards(&cards, 0);
        // assert_eq!(cards["Mist-Syndicate Naga"][0].vendor_card.price, 30.0);
        // assert_eq!(cards["Mist-Syndicate Naga"][0].price_difference_to_cheapest_vendor_card, 2);
        assert_eq!(nice_price_cards.len(), 4);
    }

    // #[test]
    // fn test_filter_nice_price_cards_custom_config() {
    //     let json_file_path = "/workspaces/mtg-prz-rust/mtg-rust/src/test/test_compared_cards.json";
    //     let cards = load_from_json_file::<Vec<ComparedCard>>(json_file_path).unwrap();

    //     // Temporarily change the CONFIG for this test
    //     env::set_var("NICE_PRICE_DIFF", "-20");
    //     let nice_price_cards = filter_nice_price_cards(&cards);
    //     assert_eq!(nice_price_cards.len(), 8);
    // }
}
