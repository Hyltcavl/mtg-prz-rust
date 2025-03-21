use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl PartialEq for SetName {
    fn eq(&self, other: &Self) -> bool {
        self.cleaned == other.cleaned
    }
}
