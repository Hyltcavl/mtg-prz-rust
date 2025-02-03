use std::collections::HashMap;
use std::error::Error;
use std::fs;

use crate::cards::card::Vendor;
use crate::delver_lense::show_tradable_cards::generate_html_footer;
use crate::utils::compare_prices::ComparedCard;
use crate::utils::file_management::load_from_json_file;
use crate::utils::string_manipulators::date_time_as_string;

// const CARDS_PER_PAGE: usize = 50;
pub fn generate_html_from_json(
    json_file_path: &str,
    output_dir: &str,
) -> Result<(), Box<dyn Error>> {
    // Read the JSON file
    let cards = load_from_json_file::<Vec<ComparedCard>>(json_file_path)?;

    // Filter cards with positive price difference
    let positive_diff_cards: Vec<&ComparedCard> = cards
        .iter()
        .filter(|card| card.price_difference_to_cheapest_vendor_card > 0)
        .collect();

    // Create output directory if it doesn't exist
    fs::create_dir_all(output_dir)?;

    let mut vendor_cards: HashMap<Vendor, Vec<&ComparedCard>> = HashMap::new();
    for card in &positive_diff_cards {
        let cheapest_vendor_card = card.vendor_cards.iter().min_by_key(|c| c.price).unwrap();
        vendor_cards
            .entry(cheapest_vendor_card.vendor.clone())
            .or_default()
            .push(card);
    }

    let current_date = date_time_as_string(None, None);

    // Generate pages for each vendor
    for (vendor, cards) in &vendor_cards {
        // Genergate vendor directories if they don't exist yet
        let vendor_directory = format!("{}/{}", output_dir, vendor.to_string().to_lowercase());
        fs::create_dir_all(&vendor_directory)?;

        let page_content = generate_page_content(cards, &current_date);
        let file_name = format!("{}/prices.html", vendor_directory);
        fs::write(&file_name, page_content)?;

        // Generate vendor index page
        // let vendor_index = generate_vendor_index_page(vendor, cards.len());
        // fs::write(format!("{}/index.html", vendor_directory), vendor_index)?;
    }

    // Generate main index page
    let index_content = generate_main_index_page(
        &vendor_cards,
        cards.len(),
        positive_diff_cards.len(),
        &current_date,
    );
    fs::write(format!("{}/index.html", output_dir), index_content)?;

    // Calculate number of pages
    // let num_pages = (positive_diff_cards.len() + CARDS_PER_PAGE - 1) / CARDS_PER_PAGE;

    // // Generate individual pages
    // for (page_num, chunk) in positive_diff_cards.chunks(CARDS_PER_PAGE).enumerate() {
    //     let page_content = generate_page_content(chunk, page_num + 1, num_pages);
    //     let file_name = format!("{}/page_{}.html", output_dir, page_num + 1);
    //     fs::write(&file_name, page_content)?;
    // }

    // // Generate index page
    // let index_content = generate_index_page(num_pages, cards.len(), positive_diff_cards.len());
    // fs::write(format!("{}/index.html", output_dir), index_content)?;

    Ok(())
}
fn generate_page_content(cards: &[&ComparedCard], current_date: &str) -> String {
    let mut sorted_cards = cards.to_vec();
    sorted_cards.sort_by(|a, b| {
        a.cheapest_set_price_mcm_sek
            .cmp(&b.cheapest_set_price_mcm_sek)
    });

    let style_and_import = r#"
    <style>
        table { 
            border-collapse: collapse; 
            width: 70%; 
            margin: 0 auto; 
        }
        th, td { 
            border: 1px solid #ddd; 
            padding: 8px; 
            text-align: left; 
        }
        th { 
            cursor: pointer;
            position: sticky;
            top: 0;
            background: white;
            z-index: 10;
            box-shadow: 0 2px 2px -1px rgba(0, 0, 0, 0.1);
        }
        th[role="columnheader"]:not(.no-sort):after {
            content: '';
            float: right;
            margin-top: 7px;
            border-width: 0 4px 4px;
            border-style: solid;
            border-color: #404040 transparent;
            visibility: hidden;
            opacity: 0;
            user-select: none;
        }
        th[aria-sort="ascending"]:not(.no-sort):after {
            border-bottom: none;
            border-width: 4px 4px 0;
        }
        th[aria-sort]:not(.no-sort):after {
            visibility: visible;
            opacity: 0.4;
        }
        th[role="columnheader"]:not(.no-sort):hover:after {
            visibility: visible;
            opacity: 1;
        }

        .card-image-container { 
            position: relative; 
            display: inline-block; 
        }
        .card-image { 
            width: 40px; 
            height: auto; 
            cursor: pointer; 
        }
        .enlarged-image {
            display: none;
            position: fixed;
            top: 50%;
            left: 50%;
            transform: translate(-50%, -50%);
            max-width: 80vw;
            max-height: 80vh;
            width: auto;
            height: auto;
            z-index: 1000;
            box-shadow: 0 0 10px rgba(0,0,0,0.5);
        }
        .card-image-container:hover .enlarged-image { 
            display: block; 
        }
        .pagination { 
            margin-top: 20px; 
            text-align: center; 
        }
        h1 {
            text-align: center;
            margin: 20px 0;
        }
        .filters {
            width: 70%;
            margin: 20px auto;
            padding: 10px;
            background: #f5f5f5;
            border-radius: 5px;
        }
        .filter-group {
            margin: 10px 0;
            display: flex;
            gap: 10px;
            align-items: center;
        }
        select, button {
            padding: 5px;
            border-radius: 4px;
            border: 1px solid #ddd;
        }
        button {
            background: #4CAF50;
            color: white;
            border: none;
            padding: 6px 12px;
            cursor: pointer;
        }
        button:hover {
            background: #45a049;
        }
        .hidden {
            display: none;
        }
    </style>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/tablesort/5.2.1/tablesort.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/tablesort/5.2.1/sorts/tablesort.number.min.js"></script>"#.to_string();

    let mut content = format!(
        r#"
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>MTG Card Price Comparison, {} </title>
        {}
    </head>
    <body>
        <h1>MTG Cards with Price Difference, {}</h1>
         <div class="filters">
        <div class="filter-group">
            <label for="rarityFilter">Rarity:</label>
            <select id="rarityFilter">
                <option value="all">All</option>
            </select>
            
            <label for="colorFilter">Color:</label>
            <select id="colorFilter">
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
        current_date, style_and_import, current_date
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

fn generate_main_index_page(
    vendor_cards: &HashMap<Vendor, Vec<&ComparedCard>>,
    total_cards: usize,
    total_with_diff: usize,
    current_date: &str,
) -> String {
    let mut content = format!(
        r#"
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>MTG-prizes</title>
    </head>
    <body>
        <h1>MTG-prizes {current_date}, Total cards: {total_cards}, Total nice price cards: {total_with_diff}</h1>
        <ul>
    "#
    );

    for (vendor, cards) in vendor_cards {
        content.push_str(&format!(
            "        <li><a href='{}/prices.html'>{} ({} cards)</a></li>\n",
            vendor.to_string().to_lowercase(),
            vendor,
            cards.len()
        ));
    }

    content.push_str("    </ul>\n</body>\n</html>");

    content
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_generate_html_from_json_creates_file() {
        let json_file_path = "/workspaces/mtg-prz-rust/mtg-rust/src/test/test_compared_cards.json";
        let output_dir = "/workspaces/mtg-prz-rust/test_output";

        // Ensure the output directory is clean
        // if Path::new(output_dir).exists() {
        //     fs::remove_dir_all(output_dir).unwrap();
        // }

        // Call the function
        generate_html_from_json(json_file_path, output_dir).unwrap();

        // Check that the output directory and index file were created
        assert!(Path::new(output_dir).exists());
        // assert!(Path::new(&format!("{}/prices.html", output_dir)).exists());

        // Clean up
        // fs::remove_dir_all(output_dir).unwrap();
    }
}
