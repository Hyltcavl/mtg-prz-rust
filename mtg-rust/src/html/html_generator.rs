use std::error::Error;
use std::fs;

use crate::utils::compare_prices::ComparedCard;
use crate::utils::file_management::load_from_json_file;

const CARDS_PER_PAGE: usize = 50;

pub fn generate_html_from_json(
    json_file_path: &str,
    output_dir: &str,
) -> Result<(), Box<dyn Error>> {
    // Read the JSON file
    // let json_content = fs::read_to_string(json_file_path)?;
    // let cards: Vec<Value> = serde_json::from_str(&json_content)?;
    let cards = load_from_json_file::<Vec<ComparedCard>>(json_file_path)?;

    // Filter cards with positive price difference
    let positive_diff_cards: Vec<&ComparedCard> = cards
        .iter()
        .filter(|card| card.price_difference_to_cheapest_vendor_card > 0)
        .collect();

    // Create output directory if it doesn't exist
    fs::create_dir_all(output_dir)?;

    // Calculate number of pages
    let num_pages = (positive_diff_cards.len() + CARDS_PER_PAGE - 1) / CARDS_PER_PAGE;

    // Generate individual pages
    for (page_num, chunk) in positive_diff_cards.chunks(CARDS_PER_PAGE).enumerate() {
        let page_content = generate_page_content(chunk, page_num + 1, num_pages);
        let file_name = format!("{}/page_{}.html", output_dir, page_num + 1);
        fs::write(&file_name, page_content)?;
    }

    // Generate index page
    let index_content = generate_index_page(num_pages, cards.len(), positive_diff_cards.len());
    fs::write(format!("{}/index.html", output_dir), index_content)?;

    Ok(())
}
fn generate_page_content(cards: &[&ComparedCard], page_num: usize, total_pages: usize) -> String {
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
        <title>MTG Card Price Comparison - Page {}</title>
        <style>
            table {{ border-collapse: collapse; width: 100%; }}
            th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
            th {{ cursor: pointer; }}
            .card-image-container {{ position: relative; display: inline-block; }}
            .card-image {{ width: 30px; height: auto; cursor: pointer; }}
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
        <h1>MTG Cards with Price Difference - Page {}</h1>
        <table id="card-table">
            <thead>
                <tr>
                    <th>Image</th>
                    <th>Name</th>
                    <th>Foil</th>
                    <th>Vendor price (SEK)</th>
                    <th>MCM price (SEK)</th>
                    <th>Price Difference</th>
                </tr>
            </thead>
            <tbody>
    "#,
        page_num, page_num
    );

    for card in sorted_cards {
        let name = &card.name;
        let foil = card.foil;
        let cheapest_mcm_price = card.cheapest_set_price_mcm_sek;
        let cheapest_vendor_price = &card.vendor_cards[0].price;
        let price_diff = card.price_difference_to_cheapest_vendor_card;
        let price_diff_percentage = card.price_diff_to_cheapest_percentage_vendor_card;
        let image_url = &card.vendor_cards[0].image_url;

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
                    <td>{foil}</td>
                    <td>{cheapest_vendor_price} SEK</td>
                    <td>{cheapest_mcm_price:.2} SEK</td>
                    <td>{price_diff:.2} SEK</td>
                </tr>
            "#
        ));
    }

    content.push_str("</tbody></table>");

    // Add pagination links
    content.push_str("<div class='pagination'>");
    if page_num > 1 {
        content.push_str(&format!(
            "<a href='page_{}.html'>Previous</a> ",
            page_num - 1
        ));
    }
    content.push_str(&format!("Page {} of {} ", page_num, total_pages));
    if page_num < total_pages {
        content.push_str(&format!("<a href='page_{}.html'>Next</a>", page_num + 1));
    }
    content.push_str("</div>");

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

fn generate_index_page(total_pages: usize, total_cards: usize, total_with_diff: usize) -> String {
    let mut content = format!(
        r#"
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>Nice prices. Total cards: {total_cards}, Total nice price cards: {total_with_diff} </title>
    </head>
    <body>
        <h1>Nice prices. Total cards: {total_cards}, Total nice price cards: {total_with_diff}</h1>
        <ul>
    "#
    );

    for i in 1..=total_pages {
        content.push_str(&format!(
            "        <li><a href='page_{}.html'>Page {}</a></li>\n",
            i, i
        ));
    }

    content.push_str("    </ul>\n</body>\n</html>");

    content
}
