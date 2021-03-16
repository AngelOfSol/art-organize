use crate::table::TableId;

use super::{media_type::MediaType, source_type::SourceType};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

pub type PieceId = TableId<Piece>;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Piece {
    pub name: String,
    pub source_type: SourceType,
    pub media_type: MediaType,
    pub added: DateTime<Local>,
    pub base_price: Option<i64>,
    pub tip_price: Option<i64>,
}

impl Default for Piece {
    fn default() -> Self {
        Self {
            name: "New Piece".to_string(),
            source_type: SourceType::Commission,
            media_type: MediaType::Image,
            added: chrono::Local::now(),
            base_price: None,
            tip_price: None,
        }
    }
}
