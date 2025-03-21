use crate::cards::{currency::Currency, tradable_card::TradeableCard};

pub fn generate_html_header() -> String {
    format!(
        r#"
    <!DOCTYPE html>
<html lang="en">

<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Tradable cards</title>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/tablesort/5.2.1/tablesort.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/tablesort/5.2.1/sorts/tablesort.number.min.js"></script>
    <style> 
            {}
    </style>
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
            
            <label for="minPriceFilter">Min Trade-in Price:</label>
            <input type="number" id="minPriceFilter" step="0.01" placeholder="0.00">
            
            <label for="minDiffFilter">Min % Diff:</label>
            <input type="number" id="minDiffFilter" step="0.01" placeholder="0.00">
            
            <button onclick="resetFilters()">Reset Filters</button>
            <button onclick="filterValueTrades()">Show Value Trades</button>
        </div>
        <input type="checkbox" id="valueTradeFilter" class="hidden">
    </div>

    <table id="card-table">
        <thead>
            <tr>
                <th class="no-sort">Image</th>
                <th>Name/Set</th>
                <th data-sort-method="number">Trade-in price</th>
                <th data-sort-method="number">MCM price</th>
                <th data-sort-method="number">Vendor requested amnt</th>
                <th data-sort-method="number">Tradable cards amnt</th>
                <th data-sort-method="string">Color</th>
                <th data-sort-method="string">Rarity</th>
                <th data-sort-method="number">% Diff</th>
            </tr>
        </thead>
        <tbody>
    "#,
        include_str!(
            "/workspaces/mtg-prz-rust/magic_card_scraper/static/tradable_cards_page/styles.css"
        )
    )
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
        include_str!(
            "/workspaces/mtg-prz-rust/magic_card_scraper/static/tradable_cards_page/filters.js"
        )
    )
}

pub fn generate_card_row(card: &TradeableCard) -> String {
    let image_url = card.image_url.clone();
    let name = card.name.almost_raw.clone();
    let set_name = card.set.raw.clone();
    let foil_text = if card.foil { " (Foil)" } else { "" };
    let trade_in_price_sek = card.trade_in_price.convert_to(Currency::SEK);

    let mcm_price_sek = card.mcm_price.convert_to(Currency::SEK);

    let vendor_stock = card.card_ammount_requested_by_vendor;
    let tradable_stock = card.cards_to_trade;
    let color = card.color.clone();
    let rarity = card.rarity;
    let percentual_difference = if mcm_price_sek > 0.0 && trade_in_price_sek > mcm_price_sek {
        ((trade_in_price_sek - mcm_price_sek) / mcm_price_sek) * 100.0
    } else {
        0.0
    };
    let is_value_trade = percentual_difference >= 50.0;
    format!(
        r#"
        <tr data-value-trade="{is_value_trade}">
            <td>
                <div class="card-image-container ">
                    <img class="card-image" src="{image_url}" alt="{name}">
                    <img class="enlarged-image" src="{image_url}" alt="{name}">
                </div>
            </td>
            <td>{name}{foil_text}/{set_name} </td>
            <td data-sort={trade_in_price_sek:.2}>{trade_in_price_sek:.2} SEK</td>
            <td data-sort={mcm_price_sek:.2}>{mcm_price_sek:.2} SEK</td>
            <td data-sort={vendor_stock:.2}>{vendor_stock:.2}</td>
            <td data-sort={tradable_stock}>{tradable_stock}</td>
            <td>{color:?}</td>
            <td>{rarity:?}</td>
            <td data-sort={percentual_difference:.2}>{percentual_difference:.2}%</td>
        </tr>
        "#
    )
}

pub fn generate_page_content(cards: &[TradeableCard]) -> String {
    let mut content = generate_html_header();

    for card in cards {
        content.push_str(&generate_card_row(card));
    }

    content.push_str(&generate_html_footer());
    content
}
