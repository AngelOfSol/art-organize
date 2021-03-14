use crate::project_dir::PROJECT;
use serde::{Deserialize, Serialize};
use std::{io::Result, path::PathBuf};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub data_dirs: Vec<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            data_dirs: Vec::new(),
        }
    }
}
fn get_file() -> PathBuf {
    let mut config_file = PROJECT.config_dir().to_path_buf();
    config_file.push("config.toml");

    config_file
}

impl Config {
    pub fn load() -> Option<Self> {
        toml::from_str(&std::fs::read_to_string(get_file()).ok()?).ok()
    }

    pub fn save(&self) -> Result<()> {
        if !PROJECT.config_dir().exists() {
            std::fs::create_dir_all(PROJECT.config_dir())?;
        }

        std::fs::write(get_file(), &toml::to_string_pretty(self).unwrap())
    }
}
