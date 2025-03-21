use serde::{Deserialize, Serialize};

use super::{
    cardname::CardName, colour::Colour, price::Price, rarity::Rarity, setname::SetName,
    vendor::Vendor,
};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct TradeableCard {
    pub name: CardName,
    pub set: SetName,
    pub foil: bool,
    pub prerelease: bool,
    pub tradeable_vendor: Vendor,
    pub trade_in_price: Price,
    pub mcm_price: Price,
    pub cards_to_trade: i8,
    pub card_ammount_requested_by_vendor: i8,
    #[serde(default = "image_url_default")]
    pub image_url: String,
    pub color: Colour,
    pub rarity: Rarity,
}

fn image_url_default() -> String {
    "https://upload.wikimedia.org/wikipedia/en/a/aa/Magic_the_gathering-card_back.jpg".to_string()
}
