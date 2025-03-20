use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Copy)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Mythic,
}

impl fmt::Display for Rarity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Rarity::Common => write!(f, "Common"),
            Rarity::Uncommon => write!(f, "Uncommon"),
            Rarity::Rare => write!(f, "Rare"),
            Rarity::Mythic => write!(f, "Mythic"),
        }
    }
}
