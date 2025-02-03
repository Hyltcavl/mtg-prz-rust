use std::fs;

use crate::utils::file_management::load_from_json_file;

use super::delver_lense_card::TradeableCard;

fn generate_html_header() -> String {
    r#"
    <!DOCTYPE html>
<html lang="en">

<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Tradable cards</title>
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
    <script src="https://cdnjs.cloudflare.com/ajax/libs/tablesort/5.2.1/sorts/tablesort.number.min.js"></script>
</head>

<body>
    <h1>Tradable cards</h1>
    
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
                <th class="no-sort">Image</th>
                <th>Name</th>
                <th data-sort-method="number">Trade-in price</th>
                <th data-sort-method="number">MCM price</th>
                <th data-sort-method="number">Vendor requested amnt</th>
                <th data-sort-method="number">Tradable cards amnt</th>
                <th>Color</th>
                <th data-sort-method="string">Rarity</th>
            </tr>
        </thead>
        <tbody>
        "#.to_string()
}

pub fn generate_html_footer() -> String {
    r#"
        </tbody></table>
        <div class='pagination'></div>
        <script>
        // Initialize Tablesort
        new Tablesort(document.getElementById('card-table'), {
            descending: true
        });

        // Function to populate filter dropdowns dynamically
        function populateFilters() {
            const rows = document.querySelectorAll('#card-table tbody tr');
            const raritySet = new Set();
            const colorSet = new Set();
            
            // Collect unique values
            rows.forEach(row => {
                const rarity = row.querySelector('td:nth-child(8)').textContent.trim();
                const color = row.querySelector('td:nth-child(7)').textContent.trim();
                
                raritySet.add(rarity);
                colorSet.add(color);
            });

            // Populate rarity filter
            const rarityFilter = document.getElementById('rarityFilter');
            rarityFilter.innerHTML = '<option value="all">All</option>';
            Array.from(raritySet).sort().forEach(rarity => {
                rarityFilter.innerHTML += `<option value="${rarity}">${rarity}</option>`;
            });

            // Populate color filter
            const colorFilter = document.getElementById('colorFilter');
            colorFilter.innerHTML = '<option value="all">All</option>';
            Array.from(colorSet).sort().forEach(color => {
                colorFilter.innerHTML += `<option value="${color}">${color}</option>`;
            });
        }

        // Filter function
        function applyFilters() {
            const rarityFilter = document.getElementById('rarityFilter').value;
            const colorFilter = document.getElementById('colorFilter').value;
            const rows = document.querySelectorAll('#card-table tbody tr');

            rows.forEach(row => {
                const rarity = row.querySelector('td:nth-child(8)').textContent.trim();
                const color = row.querySelector('td:nth-child(7)').textContent.trim();
                
                const rarityMatch = rarityFilter === 'all' || rarity === rarityFilter;
                const colorMatch = colorFilter === 'all' || color === colorFilter;

                if (rarityMatch && colorMatch) {
                    row.classList.remove('hidden');
                } else {
                    row.classList.add('hidden');
                }
            });
        }

        // Reset filters
        function resetFilters() {
            document.getElementById('rarityFilter').value = 'all';
            document.getElementById('colorFilter').value = 'all';
            const rows = document.querySelectorAll('#card-table tbody tr');
            rows.forEach(row => row.classList.remove('hidden'));
        }

        // Initialize filters
        populateFilters();

        // Add event listeners to filters
        document.getElementById('rarityFilter').addEventListener('change', applyFilters);
        document.getElementById('colorFilter').addEventListener('change', applyFilters);
    </script>
    </body>
    </html>
    "#
    .to_string()
}

fn generate_card_row(card: &TradeableCard) -> String {
    let image_url = card.image_url.clone();
    let name = card.name.almost_raw.clone();
    let foil_text = if card.foil { " (Foil)" } else { "" };
    let trade_in_price = card.trade_in_price;
    let trade_in_price_p = card.trade_in_price.amount;

    let mcm_price = card.mcm_price;
    let mcm_price_p = card.mcm_price.amount;

    let vendor_stock = card.card_ammount_requested_by_vendor;
    let tradable_stock = card.cards_to_trade;
    let color = card.color.clone();
    let rarity = card.rarity;
    format!(
        r#"
        <tr>
            <td>
                <div class="card-image-container ">
                    <img class="card-image" src="{image_url}" alt="{name}">
                    <img class="enlarged-image" src="{image_url}" alt="{name}">
                </div>
            </td>
            <td>{name} {foil_text}</td>
            <td data-sort={trade_in_price_p}>{trade_in_price} </td>
            <td data-sort={mcm_price_p}>{mcm_price:.2} </td>
            <td data-sort={vendor_stock:.2}>{vendor_stock:.2} </td>
            <td data-sort={tradable_stock}>{tradable_stock}</td>
            <td>{color:?}</td>
            <td>{rarity:?}</td>

        </tr>
        "#
    )
}

fn generate_page_content(cards: &[TradeableCard]) -> String {
    let mut content = generate_html_header();

    for card in cards {
        content.push_str(&generate_card_row(card));
    }

    content.push_str(&generate_html_footer());
    content
}

fn generate_html_from_cards(cards: &[TradeableCard]) -> String {
    let mut html = String::from(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Card Trade Overview</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .card { 
            border: 1px solid #ddd; 
            padding: 15px; 
            margin: 10px 0; 
            border-radius: 5px;
            background-color: #f9f9f9;
        }
        .profit { color: green; }
        .loss { color: red; }
        .header { 
            background-color: #333;
            color: white;
            padding: 10px;
            margin-bottom: 20px;
        }
        .price-comparison {
            display: flex;
            gap: 20px;
            margin: 10px 0;
        }
    </style>
</head>
<body>
    <div class="header">
        <h1>Card Trade Overview</h1>
    </div>
"#,
    );

    for card in cards {
        html.push_str(&format!(
            r#"<div class="card">
    <h2>{} {}</h2>
    <p><strong>Set:</strong> {}</p>
    <p><strong>Vendor:</strong> {}</p>
    <p><strong>Foil:</strong> {}</p>
    <div class="price-comparison">
        <div>
            <strong>Trade-in Price:</strong> {:.2} {:?}
        </div>
        <div>
            <strong>MCM Price:</strong> {:.2} {:?}
        </div>
    </div>
    <p><strong>Cards to Trade:</strong> {}</p>
    <p><strong>Requested by Vendor:</strong> {}</p>
</div>"#,
            card.name.almost_raw,
            if card.foil { "(Foil)" } else { "" },
            card.set.raw,
            card.tradeable_vendor,
            card.foil,
            card.trade_in_price.amount,
            card.trade_in_price.currency,
            card.mcm_price.amount,
            card.mcm_price.currency,
            card.cards_to_trade,
            card.card_ammount_requested_by_vendor
        ));
    }

    html.push_str("</body></html>");
    html
}

// Example usage function
pub fn create_tradable_card_html_page(
    tradable_cards: &Vec<TradeableCard>,
) -> Result<(), Box<dyn std::error::Error>> {
    let html = generate_page_content(&tradable_cards);

    fs::write("/workspaces/mtg-prz-rust/cards.html", html)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_html_page_for_tradable_cards() {
        let mut tradable_cards: Vec<TradeableCard> = load_from_json_file::<Vec<TradeableCard>>(
            "/workspaces/mtg-prz-rust/tradable_cards_03_02_2025-10-44.json",
        )
        .unwrap();
        tradable_cards.sort_by(|a, b| a.cards_to_trade.cmp(&b.cards_to_trade));

        create_tradable_card_html_page(&tradable_cards).unwrap();
    }
}
