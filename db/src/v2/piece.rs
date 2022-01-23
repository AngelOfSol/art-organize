use crate::table::TableId;

use chrono::{Local, NaiveDate};
use serde::{Deserialize, Serialize};

pub type PieceId = TableId<Piece>;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Piece {
    pub description: String,
    pub added: NaiveDate,
}

impl Default for Piece {
    fn default() -> Self {
        Self {
            description: String::new(),
            added: Local::today().naive_local(),
        }
    }
}
