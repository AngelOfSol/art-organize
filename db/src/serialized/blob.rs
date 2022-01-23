use std::{fmt::Display, path::PathBuf};

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::table::TableId;
use strum::EnumIter;

pub type BlobId = TableId<Blob>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Blob {
    pub file_name: String,
    pub hash: u64,
    pub blob_type: BlobType,
    pub added: NaiveDate,
}

impl Blob {
    pub fn storage_name(&self, id: BlobId) -> PathBuf {
        format!("[{}] {}", id, self.file_name).parse().unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy, EnumIter, PartialOrd, Ord)]
pub enum BlobType {
    Canon,
    Variant,
    Raw,
    Draft,
}

impl Display for BlobType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                BlobType::Canon => {
                    "Canon"
                }
                BlobType::Variant => {
                    "Variant"
                }
                BlobType::Raw => {
                    "Raw"
                }
                BlobType::Draft => {
                    "Draft"
                }
            }
        )
    }
}
