use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
pub enum SourceType {
    FanCreation,
    Official,
    Commission,
}
