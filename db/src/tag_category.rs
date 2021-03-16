use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

use crate::table::TableId;

pub type TagCategoryId = TableId<TagCategory>;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TagCategory {
    pub name: String,
    pub color: [u8; 4],
    pub added: DateTime<Local>,
}

impl Default for TagCategory {
    fn default() -> Self {
        Self {
            name: "New Tag Category".to_string(),
            color: [0; 4],
            added: Local::now(),
        }
    }
}
