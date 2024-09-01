use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Vendor {
    dragonslair,
    alphaspel,
    cardmarket,
}
