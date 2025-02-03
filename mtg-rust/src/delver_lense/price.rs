use std::cmp::Ordering;
use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum Currency {
    EUR,
    SEK,
}

impl Currency {
    /// Get the exchange rate based on currency
    /// Example: 1 SEK = 0.87 EUR (1 EUR = 11.50 SEK)
    fn exchange_rate(&self) -> f64 {
        match self {
            Currency::EUR => 11.50,
            Currency::SEK => 0.087,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Price {
    pub amount: f64,
    pub currency: Currency,
}

impl Price {
    /// Create a new Price instance
    pub fn new(amount: f64, currency: Currency) -> Self {
        Self { amount, currency }
    }

    /// Convert the price to EUR for comparison
    fn to_eur(&self) -> f64 {
        match self.currency {
            Currency::EUR => self.amount,
            Currency::SEK => self.amount * self.currency.exchange_rate(),
        }
    }
}

// Implement PartialOrd and Ord for Price
impl PartialOrd for Price {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.to_eur().partial_cmp(&other.to_eur())
    }
}

// Implement PartialEq for Price
impl PartialEq for Price {
    fn eq(&self, other: &Self) -> bool {
        self.to_eur() == other.to_eur()
    }
}

// Implement Display for Price
impl fmt::Display for Price {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let amount_in_sek = match self.currency {
            Currency::EUR => self.amount * self.currency.exchange_rate(),
            Currency::SEK => self.amount,
        };
        write!(f, "{:.2} {:?}", amount_in_sek, Currency::SEK)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reading_delver_lense_cards() {
        let price_eur = Price::new(5.0, Currency::EUR);
        let price_sek = Price::new(60.0, Currency::SEK);
        let price_sek2 = Price::new(60.0, Currency::SEK);

        assert_eq!(price_eur < price_sek, true);
        assert_eq!(price_eur > price_sek, false);
        assert_eq!(price_sek2 == price_sek, true);
    }
}
