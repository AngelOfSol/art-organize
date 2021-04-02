use clap::Clap;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clap)]
#[clap(version = "1.0", author = "Angel of Sol")]
pub struct Opts {
    #[clap(subcommand)]
    pub subcmd: Option<SubCommand>,
}

#[derive(Clap, Debug, Serialize, Deserialize)]
pub enum SubCommand {
    /// Creates a new database in the current working directory
    Init {
        path: Option<PathBuf>,
    },

    /// Updates file explorer context menus
    Contextual {
        #[clap(subcommand)]
        subcmd: ContextualSubCommand,
    },
    /// Runs the gui in the local directory.
    Gui,
    /// Resets the configuration to the default values
    ResetConfig,
    Update,
}

#[derive(Clap, Debug, Serialize, Deserialize)]
pub enum ContextualSubCommand {
    /// Installs context menu handlers to the local file explorer
    Install,
    /// Removes context menu handlers from the local file explorer
    Remove,
}
