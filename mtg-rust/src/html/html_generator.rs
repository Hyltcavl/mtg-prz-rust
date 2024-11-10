use std::collections::HashMap;
use std::error::Error;
use std::fs;

use crate::cards::card::Vendor;
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

    let mut content = format!(
        r#"
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>MTG Card Price Comparison, {} </title>
        <style>
            table {{ border-collapse: collapse; width: 70%; }}
            th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
            th {{ cursor: pointer; }}
            .card-image-container {{ position: relative; display: inline-block; }}
            .card-image {{ width: 40px; height: auto; cursor: pointer; }}
            .enlarged-image {{
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
            }}
            .card-image-container:hover .enlarged-image {{ display: block; }}
            .pagination {{ margin-top: 20px; text-align: center; }}
        </style>
        <script src="https://cdnjs.cloudflare.com/ajax/libs/tablesort/5.2.1/tablesort.min.js"></script>
    </head>
    <body>
        <h1>MTG Cards with Price Difference, {}</h1>
        <table id="card-table">
            <thead>
                <tr>
                    <th>Image</th>
                    <th>Name</th>
                    <th>Vendor price (SEK)</th>
                    <th>MCM price (SEK)</th>
                    <th>Price Difference</th>
                    <th>Vendor</th>
                </tr>
            </thead>
            <tbody>
    "#,
        current_date, current_date
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
                    <td>{cheapest_vendor_price} SEK</td>
                    <td>{cheapest_mcm_price:.2} SEK</td>
                    <td>{price_diff:.2} SEK</td>
                    <td>{vendor}</td>
                </tr>
            "#
        ));
    }

    content.push_str("</tbody></table>");

    // Add pagination links
    content.push_str("<div class='pagination'>");

    content.push_str(
        r#"
        <script>
            new Tablesort(document.getElementById('card-table'));
        </script>
    </body>
    </html>
    "#,
    );

    content
}

// fn generate_vendor_index_page(vendor: &Vendor, total_cards: usize) -> String {
//     let mut content = format!(
//         r#"
//     <!DOCTYPE html>
//     <html lang="en">
//     <head>
//         <meta charset="UTF-8">
//         <meta name="viewport" content="width=device-width, initial-scale=1.0">
//         <title>{} - MTG-prizes. Total cards: {}</title>
//     </head>
//     <body>
//         <h1>{} - MTG-prizes. Total cards: {}</h1>
//         <ul>
//     "#,
//         vendor, total_cards, vendor, total_cards
//     );

//     content.push_str(
//         "    </ul>\n    <p><a href='../index.html'>Back to main index</a></p>\n</body>\n</html>",
//     );

//     content
// }

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
