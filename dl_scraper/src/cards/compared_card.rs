use serde::{Deserialize, Serialize};

use super::{scryfallcard::ScryfallCard, vendorcard::VendorCard};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ComparedCard {
    pub vendor_card: VendorCard,
    pub scryfall_card: ScryfallCard,
    // Positive here is a cheaper vendor card then the MCM. The amount is the difference in SEK
    pub price_difference_to_cheapest_vendor_card: i32,
}

#[derive(Debug, Deserialize, Serialize)]
struct CurrencyRate {
    amount: f64,
    base: String,
    date: String,
    rates: Rates,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Serialize)]
struct Rates {
    SEK: f64,
}
