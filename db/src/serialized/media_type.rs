use std::fmt::Display;

use serde::{Deserialize, Serialize};
use strum::EnumIter;
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy, EnumIter)]
pub enum MediaType {
    Image,
    Text,
}

impl Display for MediaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                MediaType::Image => "Image",
                MediaType::Text => "Text",
            }
        )
    }
}
