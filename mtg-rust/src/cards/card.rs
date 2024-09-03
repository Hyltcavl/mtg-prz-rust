use serde::{Deserialize, Serialize};

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

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct CardName {
    pub almost_raw: String,
    pub cleaned: String,
}

impl CardName {
    pub fn new(raw: String) -> Result<Self, String> {
        let name_without_disclaimers = Self::remove_name_disclaimers(&raw);
        let cleaned_name = Self::clean_name(&name_without_disclaimers);

        if name_without_disclaimers.is_empty() || cleaned_name.is_empty() {
            return Err("Raw and cleaned names cannot be empty".to_string());
        }

        if Self::is_basic_land(&cleaned_name) {
            return Err("Card cannot be a basic land".to_string());
        }

        Ok(CardName {
            almost_raw: raw,
            cleaned: cleaned_name,
        })
    }

    fn remove_name_disclaimers(name: &str) -> String {
        let disclaimers = [
            "(Prerelease)",
            "(Showcase)",
            "(Extended Art)",
            "(Foil)",
            "(Etched Foil)",
            "(Foil Etched)",
            "(Borderless)",
            "(Full art)",
        ];

        let mut cleaned_raw = name.to_string();
        for disclaimer in &disclaimers {
            cleaned_raw = cleaned_raw.replace(disclaimer, "");
        }
        cleaned_raw.trim().to_string()
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

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Vendor {
    Dragonslair,
    Alphaspel,
    Cardmarket,
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
