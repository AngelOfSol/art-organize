use crate::table::TableId;

use chrono::{Local, NaiveDate};
use serde::{Deserialize, Serialize};

pub type PieceId = TableId<Piece>;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Piece {
    pub external_id: Option<String>,
    pub description: String,
    pub added: NaiveDate,
    pub base_price: Option<i64>,
    pub tip_price: Option<i64>,
}

impl Default for Piece {
    fn default() -> Self {
        Self {
            external_id: None,
            description: String::new(),
            added: Local::today().naive_local(),
            base_price: None,
            tip_price: None,
        }
    }
}
