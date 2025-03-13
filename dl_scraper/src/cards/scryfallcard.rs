use serde::{Deserialize, Serialize};
use crate::cards::price::Price;
use super::{cardname::CardName, setname::SetName};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ScryfallCard {
    pub name: CardName,
    pub set: SetName,
    pub image_url: String,
    pub prices: Prices,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Prices {
    pub eur: Option<Price>,
    pub eur_foil: Option<Price>,
}
