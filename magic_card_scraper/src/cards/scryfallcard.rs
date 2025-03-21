use super::{cardname::CardName, collector_number::CollectorNumber, setname::SetName};
use crate::cards::price::Price;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ScryfallCard {
    pub name: CardName,
    pub set: SetName,
    pub image_url: String,
    pub prices: Prices,
    pub collector_number: Option<CollectorNumber>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Prices {
    pub eur: Option<Price>,
    pub eur_foil: Option<Price>,
}
