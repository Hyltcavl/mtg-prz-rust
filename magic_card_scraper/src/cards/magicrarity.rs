// use rusqlite::types::FromSql;
// use rusqlite::types::FromSqlError;
// use rusqlite::types::FromSqlResult;
// use rusqlite::types::ValueRef;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Copy)]
pub enum MagicRarity {
    Common,
    Uncommon,
    Rare,
    Mythic,
}

// impl FromSql for MagicRarity {
//     fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
//         match value.as_str()? {
//             "Common" => Ok(MagicRarity::Common),
//             "Uncommon" => Ok(MagicRarity::Uncommon),
//             "Rare" => Ok(MagicRarity::Rare),
//             "Mythic" => Ok(MagicRarity::Mythic),
//             _ => Err(FromSqlError::InvalidType),
//         }
//     }
// }

impl fmt::Display for MagicRarity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MagicRarity::Common => write!(f, "Common"),
            MagicRarity::Uncommon => write!(f, "Uncommon"),
            MagicRarity::Rare => write!(f, "Rare"),
            MagicRarity::Mythic => write!(f, "Mythic"),
        }
    }
}
