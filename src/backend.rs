use std::{
    path::PathBuf,
    str::{from_utf8, FromStr},
};

use anyhow::{anyhow, Context};
use chrono::{DateTime, Local};

use crate::model::{media_type::MediaType, piece::Piece, source_type::SourceType};

pub struct Backend {
    root: PathBuf,
}

fn data_file(mut path: PathBuf) -> PathBuf {
    path.push("ao.db");
    path
}

impl Backend {
    pub async fn from_path(root: PathBuf) -> anyhow::Result<Self> {
        Ok(Self { root })
    }

    pub async fn init_at_path(root: PathBuf) -> anyhow::Result<Self> {
        Ok(Self { root })
    }

    pub async fn add_file(&mut self, file: PathBuf) -> anyhow::Result<()> {
        let file_name = file
            .file_name()
            .and_then(|x| x.to_str())
            .ok_or_else(|| anyhow!("invalid file name: {:?}", file.file_name()))?;
        let new_path = {
            let mut new_path = self.root.clone();
            new_path.push(file_name);
            new_path
        };

        std::fs::copy(file, new_path)?;

        Ok(())
    }

    pub async fn query_pieces(&self) -> anyhow::Result<Vec<()>> {
        Ok(Vec::new())
    }
}
