use crate::table::TableId;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

pub type TagId = TableId<Tag>;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Tag {
    pub name: String,
    pub description: String,
    pub added: DateTime<Local>,
    pub links: Vec<String>,
}
impl Default for Tag {
    fn default() -> Self {
        Self {
            name: "New Tag".to_string(),
            description: String::new(),
            added: Local::now(),
            links: Vec::new(),
        }
    }
}
