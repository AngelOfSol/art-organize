use std::{fmt::Display, sync::Arc};

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::table::{MaybeEntry, TableId};
use strum::EnumIter;

pub type BlobId = TableId<Blob>;
pub type MaybeBlob = MaybeEntry<Blob>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Blob {
    pub file_name: String,
    pub hash: u64,
    #[serde(with = "raw_data")]
    pub data: Arc<Vec<u8>>,
    pub blob_type: BlobType,
    pub added: NaiveDate,
}

mod raw_data {
    use std::sync::Arc;

    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(data: &Arc<Vec<u8>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        data.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Arc<Vec<u8>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let data = Vec::deserialize(deserializer)?;
        Ok(Arc::new(data))
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy, EnumIter)]
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
