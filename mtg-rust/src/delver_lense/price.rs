use std::cmp::Ordering;
use std::fmt;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Currency {
    EUR,
    SEK,
}

impl Currency {
    /// Get the exchange rate to EUR for each currency
    fn to_eur_rate(&self) -> f64 {
        match self {
            Currency::EUR => 1.0, // Base currency
            Currency::SEK => 0.1, // Example: 1 SEK = 0.1 EUR
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Price {
    amount: f64,
    currency: Currency,
}

impl Price {
    /// Create a new Price instance
    pub fn new(amount: f64, currency: Currency) -> Self {
        Self { amount, currency }
    }

    /// Convert the price to EUR for comparison
    fn to_eur(&self) -> f64 {
        self.amount * self.currency.to_eur_rate()
    }
}

// Implement PartialOrd and Ord for Price
impl PartialOrd for Price {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.to_eur().partial_cmp(&other.to_eur())
    }
}

// impl Ord for Price {
//     fn cmp(&self, other: &Self) -> Ordering {
//         self.to_eur()
//             .partial_cmp(&other.to_eur())
//             .expect("Comparison failed")
//     }
// }

// Implement PartialEq for Price
impl PartialEq for Price {
    fn eq(&self, other: &Self) -> bool {
        self.to_eur() == other.to_eur()
    }
}

// Implement Display for Price
impl fmt::Display for Price {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2} {:?}", self.amount, self.currency)
    }
}

// Example test case
fn main() {
    let price_eur = Price::new(5.0, Currency::EUR);
    let price_sek = Price::new(60.0, Currency::SEK);

    println!("Price in EUR: {}", price_eur);
    println!("Price in SEK: {}", price_sek);

    // Direct comparison
    if price_eur < price_sek {
        println!("EUR price is less than SEK price.");
    } else if price_eur > price_sek {
        println!("EUR price is greater than SEK price.");
    } else {
        println!("Prices are equal.");
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
