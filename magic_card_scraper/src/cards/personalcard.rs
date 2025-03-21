use serde::{Deserialize, Serialize};

use super::{cardname::CardName, colour::Colour, price::Price, rarity::Rarity, setname::SetName};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct PersonalCard {
    pub name: CardName,
    pub set: SetName,
    pub foil: bool,
    pub price: Price,
    pub count: i8,
    pub color: Colour,
    pub rarity: Rarity,
}
