use crate::cards::card::{CardName, SetName, VendorCard};
use crate::tradable_cars::delver_lense_card::TradeableCard;
use crate::tradable_cars::price::Price;
use rusqlite::types::ToSql;
use rusqlite::{params, Connection, Result};

struct Database {
    conn: Connection,
}

impl Database {
    fn new() -> Result<Self> {
        let conn = Connection::open("/workspaces/mtg-prz-rust/db/sqlite.db")?;
        let db = Database { conn };

        db.create_tables()?;

        Ok(db)
    }

    fn create_tables(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS tradable_cards (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                set_raw TEXT NOT NULL,
                foil BOOLEAN NOT NULL,
                tradeable_vendor TEXT NOT NULL,
                trade_in_price_amount REAL NOT NULL,
                trade_in_price_currency TEXT NOT NULL,
                mcm_price_amount REAL NOT NULL,
                mcm_price_currency TEXT NOT NULL,
                cards_to_trade INTEGER NOT NULL,
                card_ammount_requested_by_vendor INTEGER NOT NULL,
                image_url TEXT NOT NULL,
                color TEXT NOT NULL,
                rarity TEXT NOT NULL,
                UNIQUE(name, tradeable_vendor)
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS vendor_cards (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                vendor TEXT NOT NULL,
                foil BOOLEAN NOT NULL,
                image_url TEXT NOT NULL,
                extended_art BOOLEAN NOT NULL,
                prerelease BOOLEAN NOT NULL,
                showcase BOOLEAN NOT NULL,
                set_raw TEXT NOT NULL,
                price INTEGER NOT NULL,
                trade_in_price INTEGER NOT NULL,
                current_stock INTEGER NOT NULL,
                max_stock INTEGER NOT NULL,
                UNIQUE(name, vendor)
            )",
            [],
        )?;
        Ok(())
    }

    fn create_tradable_card(&self, card: &TradeableCard) -> Result<i64> {
        let sql = "INSERT OR REPLACE INTO tradable_cards (
            name, set_raw, foil, tradeable_vendor,
            trade_in_price_amount, trade_in_price_currency,
            mcm_price_amount, mcm_price_currency,
            cards_to_trade, card_ammount_requested_by_vendor,
            image_url, color, rarity
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)";

        let params: &[&dyn ToSql] = &[
            &card.name.almost_raw,
            &card.set.raw,
            &card.foil,
            &card.tradeable_vendor.to_string(),
            &card.trade_in_price.amount,
            &card.trade_in_price.currency.to_string(),
            &card.mcm_price.amount,
            &card.mcm_price.currency.to_string(),
            &card.cards_to_trade,
            &card.card_ammount_requested_by_vendor,
            &card.image_url,
            &card.color,
            &card.rarity.to_string(),
        ];

        self.conn.execute(sql, params)?;
        Ok(self.conn.last_insert_rowid())
    }

    fn get_tradable_card_by_name(&self, name: &str) -> Result<Option<TradeableCard>> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM tradable_cards WHERE name = ?1")?;

        let card_result = stmt.query_row(params![name], |row| {
            Ok(TradeableCard {
                name: CardName::new(row.get(1)?).unwrap(),
                set: SetName::new(row.get(2)?).unwrap(),
                foil: row.get(3)?,
                tradeable_vendor: row.get(4)?,
                trade_in_price: Price {
                    amount: row.get(5)?,
                    currency: row.get(6)?,
                },
                mcm_price: Price {
                    amount: row.get(7)?,
                    currency: row.get(8)?,
                },
                cards_to_trade: row.get(9)?,
                card_ammount_requested_by_vendor: row.get(10)?,
                image_url: row.get(11)?,
                color: row.get(12)?,
                rarity: row.get(13)?,
            })
        });

        match card_result {
            Ok(card) => Ok(Some(card)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    fn create_vendor_card(&self, card: &VendorCard) -> Result<i64> {
        let sql = "INSERT OR REPLACE INTO vendor_cards (
            name, vendor, foil, image_url, extended_art, prerelease, showcase, set_raw, price, trade_in_price, current_stock, max_stock
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)";

        let params: &[&dyn ToSql] = &[
            &card.name.almost_raw,
            &card.vendor.to_string(),
            &card.foil,
            &card.image_url,
            &card.extended_art,
            &card.prerelease,
            &card.showcase,
            &card.set.raw,
            &card.price,
            &card.trade_in_price,
            &card.current_stock,
            &card.max_stock,
        ];

        self.conn.execute(sql, params)?;
        Ok(self.conn.last_insert_rowid())
    }

    fn get_vendor_card_by_name(&self, name: &str) -> Result<Option<VendorCard>> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM vendor_cards WHERE name = ?1")?;

        let card_result = stmt.query_row(params![name], |row| {
            Ok(VendorCard {
                name: CardName::new(row.get(1)?).unwrap(),
                vendor: row.get(2)?,
                foil: row.get(3)?,
                image_url: row.get(4)?,
                extended_art: row.get(5)?,
                prerelease: row.get(6)?,
                showcase: row.get(7)?,
                set: SetName::new(row.get(8)?).unwrap(),
                price: row.get(9)?,
                trade_in_price: row.get(10)?,
                current_stock: row.get(11)?,
                max_stock: row.get(12)?,
            })
        });

        match card_result {
            Ok(card) => Ok(Some(card)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    fn bulk_upload_tradable_cards(&self, cards: &[TradeableCard]) -> Result<()> {
        for card in cards {
            self.create_tradable_card(card)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        cards::card::{CardName, SetName, Vendor},
        tradable_cars::{
            delver_lense_card::{MagicRarity, TradeableCard},
            price::{Currency, Price},
        },
    };

    use super::*;

    #[test]
    fn test_database_operations() -> Result<()> {
        let db = Database::new()?;

        // Create example tradable card
        let tradable_card = TradeableCard {
            name: CardName::new("Chulane, Teller of Tales".to_string()).unwrap(),
            set: SetName::new("Throne of Eldraine".to_string()).unwrap(),
            foil: true,
            tradeable_vendor: Vendor::Dragonslair,
            trade_in_price: Price {
                amount: 70.0,
                currency: Currency::SEK,
            },
            mcm_price: Price {
                amount: 2.47,
                currency: Currency::EUR,
            },
            cards_to_trade: 1,
            card_ammount_requested_by_vendor: 1,
            image_url: "https://astraeus.dragonslair.se/images/56968/product".to_string(),
            color: "White/Blue/Green".to_string(),
            rarity: MagicRarity::Mythic,
        };

        // Save the tradable card
        db.create_tradable_card(&tradable_card)?;

        // Retrieve the tradable card
        match db.get_tradable_card_by_name("Chulane, Teller of Tales")? {
            Some(retrieved_card) => println!("Retrieved tradable card: {:?}", retrieved_card),
            None => println!("Tradable card not found"),
        }

        // Create example vendor card
        let vendor_card = VendorCard {
            name: CardName::new("Chulane, Teller of Tales".to_string()).unwrap(),
            vendor: Vendor::Dragonslair,
            foil: true,
            image_url: "https://astraeus.dragonslair.se/images/56968/product".to_string(),
            extended_art: false,
            prerelease: false,
            showcase: false,
            set: SetName::new("Throne of Eldraine".to_string()).unwrap(),
            price: 70,
            trade_in_price: 60,
            current_stock: 5,
            max_stock: 10,
        };

        // Save the vendor card
        db.create_vendor_card(&vendor_card)?;

        // Retrieve the vendor card
        match db.get_vendor_card_by_name("Chulane, Teller of Tales")? {
            Some(retrieved_card) => println!("Retrieved vendor card: {:?}", retrieved_card),
            None => println!("Vendor card not found"),
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests2 {
    use super::*;
    use crate::utils::file_management::load_from_json_file;
    use std::collections::HashMap;
    use tokio;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[tokio::test]
    async fn test_fetch_card() {
        init();
        let file_path = "/workspaces/mtg-prz-rust/mtg-rust/tradable_cards_03_02_2025-12-43.json";
        let vendor_cards: Vec<TradeableCard> =
            load_from_json_file::<Vec<TradeableCard>>(file_path).unwrap();
        let db = Database::new().unwrap();

        db.bulk_upload_tradable_cards(&vendor_cards).unwrap();
    }
}
