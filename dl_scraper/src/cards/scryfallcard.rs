use serde::{Deserialize, Serialize};

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
    pub eur: Option<f64>,
    pub eur_foil: Option<f64>,
}
