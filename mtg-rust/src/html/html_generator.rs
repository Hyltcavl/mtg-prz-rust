use std::error::Error;
use std::fs;

use crate::config::CONFIG;
use crate::utils::compare_prices::ComparedCard;
use crate::utils::file_management::load_from_json_file;
use crate::utils::string_manipulators::date_time_as_string;

// Filter cards based on nice_price_diff
// A positive diff means that the card is atleast that much cheaper on MCM trend than the cheapest vendor price
// A negative diff means that the card is at most that much more expensive on MCM trend than the cheapest vendor price
// This diff is only applied on cards that have a MCM trend price of atleast 15 SEK, anything below must have a positive diff
pub fn filter_nice_price_cards(cards: &Vec<ComparedCard>) -> Vec<&ComparedCard> {
    cards
        .iter()
        .filter(|card| {
            if card.cheapest_set_price_mcm_sek <= 10 {
                card.price_difference_to_cheapest_vendor_card > -1
            } else if card.cheapest_set_price_mcm_sek <= 30 {
                card.price_difference_to_cheapest_vendor_card > -5
            } else {
                card.cheapest_set_price_mcm_sek >= 30
                    && card.price_difference_to_cheapest_vendor_card >= CONFIG.nice_price_diff
            }
        })
        .collect()
}
// const CARDS_PER_PAGE: usize = 50;
pub fn generate_html_from_json(
    json_file_path: &str,
    output_dir: &str,
) -> Result<(), Box<dyn Error>> {
    // Read the JSON file
    let cards = load_from_json_file::<Vec<ComparedCard>>(json_file_path)?;

    // Filter cards with positive price difference
    let positive_diff_cards: Vec<&ComparedCard> = filter_nice_price_cards(&cards);

    let current_date = date_time_as_string(None, None);

    let generate_page_content =
        generate_page_content(&positive_diff_cards, &current_date, cards.len());

    fs::write(format!("{}/index.html", output_dir), generate_page_content)?;

    Ok(())
}
fn generate_page_content(
    cards: &[&ComparedCard],
    current_date: &str,
    total_with_diff: usize,
) -> String {
    let mut sorted_cards = cards.to_vec();
    sorted_cards.sort_by(|a, b| {
        a.cheapest_set_price_mcm_sek
            .cmp(&b.cheapest_set_price_mcm_sek)
    });

    let mut content = format!(
        r#"
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>MTG Card Price Comparison, {} </title>
        <link rel="stylesheet" href="/mtg-rust/static/nice_price_cards_page/style.css">
        <script src="https://cdnjs.cloudflare.com/ajax/libs/tablesort/5.2.1/tablesort.min.js"></script>
        <script src="https://cdnjs.cloudflare.com/ajax/libs/tablesort/5.2.1/sorts/tablesort.number.min.js"></script>
    </head>
    <body>
        <h1>MTG-prizes {}, Total cards: {}, Total nice price cards: {}</h1>
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
        current_date,
        cards.len(),
        total_with_diff
    );

    for card in sorted_cards {
        let cheapest_vendor_card = &card
            .vendor_cards
            .clone()
            .into_iter()
            .min_by(|x, y| x.price.cmp(&y.price))
            .unwrap();
        let vendor = &cheapest_vendor_card.vendor;
        let name = &card.name;
        let foil = card.foil;
        let foil_text = if foil { "(foil)" } else { "" };
        let cheapest_mcm_price = card.cheapest_set_price_mcm_sek;
        let cheapest_vendor_price = &cheapest_vendor_card.price;
        let price_diff = card.price_difference_to_cheapest_vendor_card;
        let image_url = &cheapest_vendor_card.image_url;

        content.push_str(&format!(
            r#"
                <tr>
                    <td>
                        <div class="card-image-container">
                            <img class="card-image" src="{image_url}" alt="{name}">
                            <img class="enlarged-image" src="{image_url}" alt="{name}">
                        </div>
                    </td>
                    <td>{name} {foil_text}</td>
                    <td data-sort={cheapest_vendor_price}>{cheapest_vendor_price} SEK</td>
                    <td data-sort={cheapest_mcm_price:.2}>{cheapest_mcm_price:.2} SEK</td>
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
    r#"
        </tbody></table>
        <div class='pagination'></div>
        <script src="/mtg-rust/static/nice_price_cards_page/filter.js"></script>
    </body>
    </html>
    "#
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::path::Path;

    #[test]
    fn test_generate_html_from_json_creates_file() {
        let json_file_path = "/workspaces/mtg-prz-rust/mtg-rust/src/test/test_compared_cards.json";
        let output_dir = "/workspaces/mtg-prz-rust";

        // Call the function
        generate_html_from_json(json_file_path, output_dir).unwrap();

        // Check that the output directory and index file were created
        assert!(Path::new(output_dir).exists());
    }

    #[test]
    fn test_filter_nice_price_cards_default_config() {
        let json_file_path = "/workspaces/mtg-prz-rust/mtg-rust/src/test/test_compared_cards.json";
        let cards = load_from_json_file::<Vec<ComparedCard>>(json_file_path).unwrap();
        let nice_price_cards = filter_nice_price_cards(&cards);
        assert_eq!(nice_price_cards.len(), 7);
    }

    #[test]
    fn test_filter_nice_price_cards_custom_config() {
        let json_file_path = "/workspaces/mtg-prz-rust/mtg-rust/src/test/test_compared_cards.json";
        let cards = load_from_json_file::<Vec<ComparedCard>>(json_file_path).unwrap();

        // Temporarily change the CONFIG for this test
        env::set_var("NICE_PRICE_DIFF", "-20");
        let nice_price_cards = filter_nice_price_cards(&cards);
        assert_eq!(nice_price_cards.len(), 8);
    }
}
