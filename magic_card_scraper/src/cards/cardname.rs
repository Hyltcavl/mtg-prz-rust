use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone)]
pub struct CardName {
    pub raw: String,
    pub almost_raw: String,
    pub cleaned: String,
    double_faced: bool,
}

impl CardName {
    pub fn new(raw: String) -> Result<Self, String> {
        let name_without_disclaimers = Self::remove_things_in_parenthesies_after_name(&raw);
        let double_faced = name_without_disclaimers.clone().contains("//");
        let cleaned_name = Self::clean_name(&name_without_disclaimers);

        if name_without_disclaimers.is_empty() || cleaned_name.is_empty() {
            return Err("Raw and cleaned names cannot be empty".to_string());
        }

        if Self::is_basic_land(&cleaned_name) {
            return Err("Card cannot be a basic land".to_string());
        }

        Ok(CardName {
            raw,
            almost_raw: name_without_disclaimers,
            cleaned: cleaned_name,
            double_faced,
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
        // Check if names contain names only if its a dubble faced card
            || (self.double_faced && (self.cleaned.contains(&other.cleaned) || other.cleaned.contains(&self.cleaned)))
            || (other.double_faced && (self.cleaned.contains(&other.cleaned) || other.cleaned.contains(&self.cleaned)))
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
        serializer.serialize_str(&self.raw)
    }
}
impl<'de> Deserialize<'de> for CardName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(CardName::new(s).unwrap())
    }
}
