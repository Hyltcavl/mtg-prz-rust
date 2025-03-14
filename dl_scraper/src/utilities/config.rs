use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub dragonslair: bool,
    pub scryfall: bool,
    pub alpha: bool,
    pub nice_price_diff: i32,
    pub external_price_check: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            dragonslair: true,
            scryfall: true,
            alpha: true,
            nice_price_diff: 0,
            external_price_check: true,
        }
    }
}

impl Config {
    pub fn new() -> Self {
        let mut config = Config::default();
        config.update_from_env();
        config
    }

    fn update_from_env(&mut self) {
        if let Ok(dragonslair) = env::var("DL") {
            self.dragonslair = dragonslair == "1";
        }
        if let Ok(scryfall) = env::var("SF") {
            self.scryfall = scryfall == "1";
        }
        if let Ok(alpha) = env::var("AS") {
            self.alpha = alpha == "1";
        }
        if let Ok(nice_price_diff) = env::var("NICE_PRICE_DIFF") {
            self.nice_price_diff = nice_price_diff.parse().unwrap_or(0);
        }
        if let Ok(external_price_check) = env::var("EXTERNAL_PRICE_CHECK") {
            self.external_price_check = external_price_check == "1";
        }
    }
}

lazy_static::lazy_static! {
    pub static ref CONFIG: Config = Config::new();
}
