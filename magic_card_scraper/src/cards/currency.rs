use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum Currency {
    EUR,
    SEK,
}

impl Currency {
    /// Get the exchange rate based on currency
    /// Example: 1 SEK = 0.87 EUR (1 EUR = 11.50 SEK)
    pub fn exchange_rate(&self) -> f64 {
        match self {
            Currency::EUR => 11.0304,
            Currency::SEK => 0.090658791,
        }
    }
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Currency::EUR => write!(f, "EUR"),
            Currency::SEK => write!(f, "SEK"),
        }
    }
}
impl FromStr for Currency {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "SEK" => Ok(Currency::SEK),
            "EUR" => Ok(Currency::EUR),
            _ => Err(()),
        }
    }
}
