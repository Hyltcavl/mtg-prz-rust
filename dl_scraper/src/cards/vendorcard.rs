use serde::{Deserialize, Serialize};

use super::{
    cardname::CardName, collector_number::CollectorNumber, price::Price, setname::SetName,
    vendor::Vendor,
};

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
    pub price: Price,
    pub trade_in_price: i32,
    pub current_stock: i8,
    pub max_stock: i8,
    pub collector_number: Option<CollectorNumber>,
}
