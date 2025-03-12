use serde::{Deserialize, Serialize};

use super::{cardname::CardName, magicrarity::MagicRarity, setname::SetName};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct PersonalCard {
    pub name: CardName,
    pub set: SetName,
    pub foil: bool,
    pub price: f64,
    pub count: i8,
    pub color: String,
    pub rarity: MagicRarity,
}
