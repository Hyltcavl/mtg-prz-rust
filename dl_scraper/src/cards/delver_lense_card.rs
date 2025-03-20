use serde::{Deserialize, Serialize};

#[allow(non_snake_case)]
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct DelverLenseCard {
    pub Name: String,
    pub Foil: String,
    pub Edition: String,
    pub Price: String,
    pub Quantity: String,
    pub Color: String,
    pub Rarity: String,
}
