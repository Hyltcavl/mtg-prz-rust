use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Vendor {
    Dragonslair,
    Alphaspel,
    Cardmarket,
}
