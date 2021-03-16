use serde::{Deserialize, Serialize};
use undo::Command;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
pub enum SourceType {
    FanCreation,
    Official,
    Commission,
}
