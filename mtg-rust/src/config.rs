use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub dl: bool,
    pub scryfall: bool,
    pub alpha: bool,
    pub nice_price_diff: i32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            dl: true,
            scryfall: true,
            alpha: true,
            nice_price_diff: 0,
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
        if let Ok(dl) = env::var("DL") {
            self.dl = dl == "1";
        }
        if let Ok(scryfall) = env::var("SCRYFALL") {
            self.scryfall = scryfall == "1";
        }
        if let Ok(alpha) = env::var("ALPHASPEL") {
            self.alpha = alpha == "1";
        }
        if let Ok(nice_price_diff) = env::var("NICE_PRICE_DIFF") {
            self.nice_price_diff = nice_price_diff.parse().unwrap_or(0);
        }
    }
}

lazy_static::lazy_static! {
    pub static ref CONFIG: Config = Config::new();
}
