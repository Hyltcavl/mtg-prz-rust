use serde::{Deserialize, Serialize};

use super::{cardname::CardName, setname::SetName, vendor::Vendor};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct VendorCard {
    pub name: CardName,
    pub vendor: Vendor,
    pub foil: bool,
    pub image_url: String,
    pub extended_art: bool,
    pub prerelease: bool,
    pub showcase: bool,
    pub set: SetName,
    pub price: i32,
    pub trade_in_price: i32,
    pub current_stock: i8,
    pub max_stock: i8,
}

// implementation for collector number
// pub fn new(raw: String, collector_number: Option<String>) -> Result<Self, String> {
//     let collector_number = if let Some(ref num) = collector_number {
//         let is_valid = num.len() >= 5
//             && num.len() <= 10
//             && num.chars().all(|c| c.is_alphanumeric() || c == '-');
//         if !is_valid {
//             log::error!("Collector number must be between 5 and 10 characters long and only contain letters, numbers, and dashes");
//             None
//         } else {
//             Some(num.clone())
//         }
//     } else {
//         None
//     };
