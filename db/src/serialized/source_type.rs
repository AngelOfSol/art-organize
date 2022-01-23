use std::fmt::Display;

use serde::{Deserialize, Serialize};
use strum::EnumIter;
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy, EnumIter)]
pub enum SourceType {
    FanCreation,
    Official,
    Commission,
}

impl Display for SourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SourceType::FanCreation => "Fan Creation",
                SourceType::Official => "Official",
                SourceType::Commission => "Commission",
            }
        )
    }
}
