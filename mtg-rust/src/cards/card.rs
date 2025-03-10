use std::{
    cmp::Ordering,
    fmt,
    hash::{Hash, Hasher},
};

use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ValueRef};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::tradable_cars::delver_lense_card::MagicRarity;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct SetName {
    pub raw: String,
    pub cleaned: String,
}

impl SetName {
    pub fn new(raw: String) -> Result<Self, String> {
        let raw = raw.replace("'", "").replace("\"", "");
        let cleaned = Self::clean_set_name(&raw);

        if raw.is_empty() || cleaned.is_empty() {
            return Err("Raw and cleaned names cannot be empty".to_string());
        }

        Ok(SetName { raw, cleaned })
    }

    fn clean_set_name(name: &str) -> String {
        name.chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect::<String>()
            .trim()
            .to_string()
            .to_lowercase()
    }
}

#[derive(Debug, Clone)]
pub struct CardName {
    pub almost_raw: String,
    pub cleaned: String,
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

impl CardName {
    pub fn new(raw: String) -> Result<Self, String> {
        let name_without_disclaimers = Self::remove_things_in_parenthesies_after_name(&raw);
        let cleaned_name = Self::clean_name(&name_without_disclaimers);

        if name_without_disclaimers.is_empty() || cleaned_name.is_empty() {
            return Err("Raw and cleaned names cannot be empty".to_string());
        }

        if Self::is_basic_land(&cleaned_name) {
            return Err("Card cannot be a basic land".to_string());
        }

        Ok(CardName {
            almost_raw: name_without_disclaimers,
            cleaned: cleaned_name,
        })
    }

    // TODO: meybe use this instead of remove name disclaimers
    fn remove_things_in_parenthesies_after_name(str: &str) -> String {
        str.find("(")
            .map_or_else(|| str.to_string(), |start| str[..start].trim().to_string())
    }

    fn clean_name(name: &str) -> String {
        name.to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || c == &' ')
            .collect::<String>()
            .replace("Æ", "ae")
            .trim()
            .to_string()
    }

    fn is_basic_land(name: &str) -> bool {
        matches!(name, "mountain" | "island" | "plains" | "swamp" | "forest")
    }
}

impl PartialEq for CardName {
    fn eq(&self, other: &Self) -> bool {
        self.cleaned == other.cleaned
    }
}

impl Eq for CardName {}

impl PartialOrd for CardName {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cleaned.cmp(&other.cleaned))
    }
}

impl Ord for CardName {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cleaned.cmp(&other.cleaned)
    }
}
impl Hash for CardName {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.cleaned.hash(state);
    }
}

impl Serialize for CardName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize CardName as a string (you can choose which field to use)
        serializer.serialize_str(&self.almost_raw)
    }
}
impl<'de> Deserialize<'de> for CardName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        // You might want to implement the cleaning logic here
        Ok(CardName {
            almost_raw: s.clone(),
            cleaned: Self::clean_name(&s), // This is a simplification
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Hash)]
pub enum Vendor {
    Dragonslair,
    Alphaspel,
    Cardmarket,
}

impl fmt::Display for Vendor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Vendor::Dragonslair => write!(f, "Dragonslair"),
            Vendor::Alphaspel => write!(f, "Alphaspel"),
            Vendor::Cardmarket => write!(f, "Cardmarket"),
        }
    }
}

impl FromSql for Vendor {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value.as_str()? {
            "Dragonslair" => Ok(Vendor::Dragonslair),
            "Alphaspel" => Ok(Vendor::Alphaspel),
            "Cardmarket" => Ok(Vendor::Cardmarket),
            _ => Err(FromSqlError::InvalidType),
        }
    }
}

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

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Prices {
    pub eur: Option<f64>,
    pub eur_foil: Option<f64>,
}
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ScryfallCard {
    pub name: CardName,
    pub set: SetName,
    pub image_url: String,
    pub prices: Prices,
}

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
