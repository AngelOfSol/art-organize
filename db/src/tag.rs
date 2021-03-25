use crate::table::TableId;
use chrono::{Local, NaiveDate};
use serde::{Deserialize, Serialize};

pub type TagId = TableId<Tag>;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Tag {
    pub name: String,
    pub description: String,
    pub added: NaiveDate,
    pub links: Vec<String>,
}
impl Default for Tag {
    fn default() -> Self {
        Self {
            name: "New Tag".to_string(),
            description: String::new(),
            added: Local::today().naive_local(),
            links: Vec::new(),
        }
    }
}
