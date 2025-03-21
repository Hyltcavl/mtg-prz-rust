// use rusqlite::types::FromSql;
// use rusqlite::types::FromSqlError;
// use rusqlite::types::FromSqlResult;
// use rusqlite::types::ValueRef;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Hash)]
pub enum Vendor {
    Dragonslair,
    Alphaspel,
    Cardmarket,
}

impl fmt::Display for Vendor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Vendor::Dragonslair => write!(f, "Dragonslair"),
            Vendor::Alphaspel => write!(f, "Alphaspel"),
            Vendor::Cardmarket => write!(f, "Cardmarket"),
        }
    }
}

// impl FromSql for Vendor {
//     fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
//         match value.as_str()? {
//             "Dragonslair" => Ok(Vendor::Dragonslair),
//             "Alphaspel" => Ok(Vendor::Alphaspel),
//             "Cardmarket" => Ok(Vendor::Cardmarket),
//             _ => Err(FromSqlError::InvalidType),
//         }
//     }
// }
