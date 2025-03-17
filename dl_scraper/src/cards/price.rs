use std::cmp::Ordering;
use std::fmt;

// use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ValueRef};
use serde::{Deserialize, Serialize};

use super::currency::Currency;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Price {
    pub amount: f64,
    pub currency: Currency,
}

// impl FromSql for Currency {
//     fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
//         match value.as_str()? {
//             "EUR" => Ok(Currency::EUR),
//             "SEK" => Ok(Currency::SEK),
//             _ => Err(FromSqlError::InvalidType),
//         }
//     }
// }

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

    /// Convert the price to the specified currency
    pub fn convert_to(&self, target_currency: Currency) -> f64 {
        if self.currency == target_currency {
            self.amount
        } else {
            let amount_in_eur = self.to_eur();
            match target_currency {
                Currency::EUR => amount_in_eur,
                Currency::SEK => amount_in_eur * Currency::EUR.exchange_rate(),
            }
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

    #[test]
    fn test_price_conversion() {
        let price_eur = Price::new(5.0, Currency::EUR);
        let price_sek = Price::new(60.0, Currency::SEK);

        assert_eq!(price_eur.convert_to(Currency::EUR), 5.0);
        assert_eq!(price_eur.convert_to(Currency::SEK), 55.152);
        assert_eq!(price_sek.convert_to(Currency::SEK), 60.0);
        assert_eq!(price_sek.convert_to(Currency::EUR), 5.43952746);
    }
}
