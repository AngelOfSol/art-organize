use chrono::{Local, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::table::TableId;

pub type TagCategoryId = TableId<TagCategory>;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct TagCategory {
    pub name: String,
    pub color: [u8; 4],
    pub added: NaiveDate,
}

impl TagCategory {
    pub fn raw_color(&self) -> [f32; 4] {
        [
            self.color[0] as f32 / 255.0,
            self.color[1] as f32 / 255.0,
            self.color[2] as f32 / 255.0,
            self.color[3] as f32 / 255.0,
        ]
    }
}

impl Default for TagCategory {
    fn default() -> Self {
        Self {
            name: "New Tag Category".to_string(),
            color: [0; 4],
            added: Local::today().naive_local(),
        }
    }
}
