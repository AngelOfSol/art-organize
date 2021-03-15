use chrono::{DateTime, Local};

use super::{media_type::MediaType, source_type::SourceType};

#[derive(Debug)]
pub struct Piece {
    pub name: String,
    pub source_type: SourceType,
    pub media_type: MediaType,
    pub added: DateTime<Local>,
    pub base_price: Option<i64>,
    pub tip_price: Option<i64>,
}
